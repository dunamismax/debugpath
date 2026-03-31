import { afterAll, beforeAll, expect, test } from 'bun:test';
import { randomUUID } from 'node:crypto';

import { appShellResponseSchema, investigationDetailResponseSchema } from '@debugpath/contracts';

import { migrateDatabase } from '../../../../db/scripts/migrate';
import { seedDevelopmentDatabase } from '../../../../db/scripts/seed';
import { type Database, closeDatabase, createDatabase } from '../../src/db/client';
import { app } from '../../src/index';

const baseDatabaseUrl =
  Bun.env.DATABASE_URL ?? 'postgresql://debugpath:debugpath@localhost:5433/debugpath';
const adminDatabaseUrl = (() => {
  const url = new URL(baseDatabaseUrl);
  url.pathname = '/postgres';
  return url.toString();
})();
const testDatabaseName = `debugpath_auth_${randomUUID().replaceAll('-', '')}`;
const testDatabaseUrl = (() => {
  const url = new URL(baseDatabaseUrl);
  url.pathname = `/${testDatabaseName}`;
  return url.toString();
})();

let adminDatabase: Database;
let database: Database;

const assertSafeIdentifier = (value: string) => {
  if (!/^[a-z0-9_]+$/i.test(value)) {
    throw new Error(`Unsafe SQL identifier: ${value}`);
  }
};

const formRequest = (url: string, values: Record<string, string>, cookie?: string) => {
  const formData = new FormData();
  for (const [key, value] of Object.entries(values)) {
    formData.set(key, value);
  }

  const headers = new Headers();
  if (cookie) {
    headers.set('cookie', cookie);
  }

  return new Request(url, {
    method: 'POST',
    body: formData,
    headers,
  });
};

const cookieFrom = (response: Response) => {
  const header = response.headers.get('set-cookie');
  expect(header).toBeString();

  if (!header) {
    throw new Error('Expected a session cookie in the response.');
  }

  return header.split(';', 1)[0] ?? '';
};

beforeAll(async () => {
  assertSafeIdentifier(testDatabaseName);

  adminDatabase = createDatabase(adminDatabaseUrl, 1);
  await adminDatabase.unsafe(`drop database if exists "${testDatabaseName}" with (force)`);
  await adminDatabase.unsafe(`create database "${testDatabaseName}"`);

  await migrateDatabase({ databaseUrl: testDatabaseUrl, quiet: true });
  await seedDevelopmentDatabase({ databaseUrl: testDatabaseUrl, quiet: true });

  Bun.env.DATABASE_URL = testDatabaseUrl;
  await closeDatabase();
  database = createDatabase(testDatabaseUrl, 1);
});

afterAll(async () => {
  await closeDatabase();

  if (database) {
    await database.end({ timeout: 5 });
  }

  if (adminDatabase) {
    await adminDatabase.unsafe(`drop database if exists "${testDatabaseName}" with (force)`);
    await adminDatabase.end({ timeout: 5 });
  }
});

test('registration creates a personal workspace and a live session-backed shell', async () => {
  const registerResponse = await app.handle(
    formRequest('http://debugpath.local/api/v1/auth/register', {
      displayName: 'Avery Debugger',
      email: 'avery@debugpath.dev',
      password: 'avery-debugpath-pass',
      redirectTo: '/app',
    })
  );

  expect(registerResponse.status).toBe(302);
  expect(registerResponse.headers.get('location')).toBe('/app');

  const sessionCookie = cookieFrom(registerResponse);
  const shellResponse = await app.handle(
    new Request('http://debugpath.local/api/v1/app-shell', {
      headers: {
        cookie: sessionCookie,
      },
    })
  );

  expect(shellResponse.status).toBe(200);

  const shell = appShellResponseSchema.parse(await shellResponse.json()).data;
  expect(shell.currentUser.email).toBe('avery@debugpath.dev');
  expect(shell.currentWorkspace.slug).toBe('avery-debugger-workspace');
  expect(shell.workspaces).toHaveLength(1);
  expect(shell.investigations).toHaveLength(0);
});

test('seeded owner can sign in and run the investigation CRUD surface', async () => {
  const loginResponse = await app.handle(
    formRequest('http://debugpath.local/api/v1/auth/login', {
      email: 'owner@debugpath.dev',
      password: 'debugpath-dev-password',
      redirectTo: '/app',
    })
  );

  expect(loginResponse.status).toBe(302);
  const sessionCookie = cookieFrom(loginResponse);

  const createResponse = await app.handle(
    formRequest(
      'http://debugpath.local/api/v1/investigations',
      {
        title: 'Payments 502 on debugpath.dev',
        summary: 'New regression in the production payments lane.',
        severity: 'critical',
        redirectTo: '/app',
      },
      sessionCookie
    )
  );

  expect(createResponse.status).toBe(302);
  expect(createResponse.headers.get('location')).toBe('/app');

  const shellResponse = await app.handle(
    new Request('http://debugpath.local/api/v1/app-shell', {
      headers: {
        cookie: sessionCookie,
      },
    })
  );
  const shell = appShellResponseSchema.parse(await shellResponse.json()).data;
  const createdInvestigation = shell.investigations.find(
    (investigation) => investigation.title === 'Payments 502 on debugpath.dev'
  );

  expect(createdInvestigation).toBeDefined();
  expect(shell.investigations.length).toBeGreaterThanOrEqual(2);

  if (!createdInvestigation) {
    throw new Error('Expected the created investigation to be present in the app shell.');
  }

  const createdInvestigationId = createdInvestigation.id;

  const updateResponse = await app.handle(
    formRequest(
      `http://debugpath.local/api/v1/investigations/${createdInvestigationId}`,
      {
        title: 'Payments 502 on debugpath.dev',
        summary: 'Confirmed against the debugpath.dev production edge and database retries.',
        severity: 'critical',
        status: 'resolved',
        redirectTo: `/app/investigations/${createdInvestigationId}`,
      },
      sessionCookie
    )
  );

  expect(updateResponse.status).toBe(302);
  expect(updateResponse.headers.get('location')).toBe(
    `/app/investigations/${createdInvestigationId}?saved=1`
  );

  const detailResponse = await app.handle(
    new Request(`http://debugpath.local/api/v1/investigations/${createdInvestigationId}`, {
      headers: {
        cookie: sessionCookie,
      },
    })
  );
  const detail = investigationDetailResponseSchema.parse(await detailResponse.json()).data;
  expect(detail.status).toBe('resolved');
  expect(detail.summary).toContain('production edge');

  const archiveResponse = await app.handle(
    formRequest(
      `http://debugpath.local/api/v1/investigations/${createdInvestigationId}/archive`,
      {
        redirectTo: '/app',
      },
      sessionCookie
    )
  );

  expect(archiveResponse.status).toBe(302);
  expect(archiveResponse.headers.get('location')).toBe('/app?archived=1');

  const archivedDetailResponse = await app.handle(
    new Request(`http://debugpath.local/api/v1/investigations/${createdInvestigationId}`, {
      headers: {
        cookie: sessionCookie,
      },
    })
  );
  const archivedDetail = investigationDetailResponseSchema.parse(
    await archivedDetailResponse.json()
  ).data;
  expect(archivedDetail.status).toBe('archived');
  expect(archivedDetail.archivedAt).toBeString();
});

test('users cannot read or mutate investigations from another workspace', async () => {
  const outsiderRegister = await app.handle(
    formRequest('http://debugpath.local/api/v1/auth/register', {
      displayName: 'Taylor Outsider',
      email: 'taylor@debugpath.dev',
      password: 'taylor-debugpath-pass',
      redirectTo: '/app',
    })
  );
  const outsiderCookie = cookieFrom(outsiderRegister);

  await app.handle(
    formRequest(
      'http://debugpath.local/api/v1/investigations',
      {
        title: 'Private outsider incident',
        summary: 'Should stay in Taylor workspace only.',
        severity: 'medium',
        redirectTo: '/app',
      },
      outsiderCookie
    )
  );

  const outsiderShellResponse = await app.handle(
    new Request('http://debugpath.local/api/v1/app-shell', {
      headers: {
        cookie: outsiderCookie,
      },
    })
  );
  const outsiderShell = appShellResponseSchema.parse(await outsiderShellResponse.json()).data;
  const outsiderInvestigation = outsiderShell.investigations.find(
    (investigation) => investigation.title === 'Private outsider incident'
  );

  expect(outsiderInvestigation).toBeDefined();

  if (!outsiderInvestigation) {
    throw new Error('Expected the outsider investigation to be present in the app shell.');
  }

  const outsiderInvestigationId = outsiderInvestigation.id;

  const ownerLogin = await app.handle(
    formRequest('http://debugpath.local/api/v1/auth/login', {
      email: 'owner@debugpath.dev',
      password: 'debugpath-dev-password',
      redirectTo: '/app',
    })
  );
  const ownerCookie = cookieFrom(ownerLogin);

  const readAttempt = await app.handle(
    new Request(`http://debugpath.local/api/v1/investigations/${outsiderInvestigationId}`, {
      headers: {
        cookie: ownerCookie,
      },
    })
  );
  expect(readAttempt.status).toBe(404);

  const archiveAttempt = await app.handle(
    formRequest(
      `http://debugpath.local/api/v1/investigations/${outsiderInvestigationId}/archive`,
      {
        redirectTo: '/app',
      },
      ownerCookie
    )
  );
  expect(archiveAttempt.status).toBe(302);
  expect(archiveAttempt.headers.get('location')).toBe('/app?error=not_found');

  const row = await database<{ status: string }[]>`
    select status::text as status
    from investigations
    where id = ${outsiderInvestigationId}
    limit 1
  `;
  expect(row[0]?.status).toBe('active');
});
