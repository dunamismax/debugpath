import { afterAll, beforeAll, expect, test } from 'bun:test';
import { randomUUID } from 'node:crypto';

import { migrateDatabase } from '../../../../db/scripts/migrate';
import { seedDevelopmentDatabase } from '../../../../db/scripts/seed';
import { type Database, createDatabase } from '../../src/db/client';
import { listInvestigationsByWorkspace } from '../../src/db/repositories';

const baseDatabaseUrl =
  Bun.env.DATABASE_URL ?? 'postgresql://debugpath:debugpath@localhost:5433/debugpath';
const adminDatabaseUrl = (() => {
  const url = new URL(baseDatabaseUrl);
  url.pathname = '/postgres';
  return url.toString();
})();
const testDatabaseName = `debugpath_test_${randomUUID().replaceAll('-', '')}`;
const testDatabaseUrl = (() => {
  const url = new URL(baseDatabaseUrl);
  url.pathname = `/${testDatabaseName}`;
  return url.toString();
})();

let adminDatabase: Database;
let database: Database;
let seededWorkspaceId = '';
let seededOwnerId = '';

const assertSafeIdentifier = (value: string) => {
  if (!/^[a-z0-9_]+$/i.test(value)) {
    throw new Error(`Unsafe SQL identifier: ${value}`);
  }
};

beforeAll(async () => {
  assertSafeIdentifier(testDatabaseName);

  adminDatabase = createDatabase(adminDatabaseUrl, 1);
  await adminDatabase.unsafe(`drop database if exists "${testDatabaseName}" with (force)`);
  await adminDatabase.unsafe(`create database "${testDatabaseName}"`);

  await migrateDatabase({ databaseUrl: testDatabaseUrl, quiet: true });
  await seedDevelopmentDatabase({ databaseUrl: testDatabaseUrl, quiet: true });

  database = createDatabase(testDatabaseUrl, 1);

  const workspaceRows = await database<{ id: string }[]>`
    select id
    from workspaces
    where slug = ${'debugpath-lab'}
    limit 1
  `;
  seededWorkspaceId = workspaceRows[0]?.id ?? '';

  const ownerRows = await database<{ id: string }[]>`
    select id
    from users
    where email = ${'owner@debugpath.dev'}
    limit 1
  `;
  seededOwnerId = ownerRows[0]?.id ?? '';
});

afterAll(async () => {
  if (database) {
    await database.end({ timeout: 5 });
  }

  if (adminDatabase) {
    await adminDatabase.unsafe(`drop database if exists "${testDatabaseName}" with (force)`);
    await adminDatabase.end({ timeout: 5 });
  }
});

test('migrations create the schema tables through the current phase and rerun without drift', async () => {
  const rerun = await migrateDatabase({ databaseUrl: testDatabaseUrl, quiet: true });
  expect(rerun.applied).toHaveLength(0);
  expect(rerun.skipped).toContain('0001_initial_schema.sql');

  const tableRows = await database<{ tableName: string }[]>`
    select table_name as "tableName"
    from information_schema.tables
    where table_schema = 'public'
    order by table_name asc
  `;
  const tableNames = tableRows.map((row) => row.tableName);

  expect(tableNames).toEqual(
    expect.arrayContaining([
      'artifacts',
      'audit_events',
      'bundle_artifacts',
      'bundle_notes',
      'bundle_share_links',
      'bundles',
      'ingestion_jobs',
      'investigations',
      'notes',
      'schema_migrations',
      'user_sessions',
      'users',
      'workspace_memberships',
      'workspaces',
    ])
  );
});

test('seed is idempotent and repositories can read the seeded investigation graph', async () => {
  const secondSeed = await seedDevelopmentDatabase({ databaseUrl: testDatabaseUrl, quiet: true });

  expect(secondSeed.workspaceId).toBe(seededWorkspaceId);
  expect(secondSeed.ownerEmail).toBe('owner@debugpath.dev');

  const investigations = await listInvestigationsByWorkspace(database, seededWorkspaceId);
  expect(investigations).toHaveLength(1);
  expect(investigations[0]?.slug).toBe('checkout-outage-debugpath-dev');
  expect(investigations[0]?.status).toBe('active');

  const counts = await database<
    { noteCount: number; artifactCount: number; shareCount: number; bundleCount: number }[]
  >`
    select
      (select count(*)::int from notes) as "noteCount",
      (select count(*)::int from artifacts) as "artifactCount",
      (select count(*)::int from bundle_share_links) as "shareCount",
      (select count(*)::int from bundles) as "bundleCount"
  `;

  expect(counts[0]?.noteCount).toBe(1);
  expect(counts[0]?.artifactCount).toBe(1);
  expect(counts[0]?.bundleCount).toBe(1);
  expect(counts[0]?.shareCount).toBe(1);
});

test('relational constraints reject orphaned investigations', async () => {
  let error: unknown = null;

  try {
    await database`
      insert into investigations (workspace_id, created_by_user_id, slug, title, status)
      values (${randomUUID()}, ${seededOwnerId}, ${'orphaned-investigation'}, ${'Orphaned'}, ${'draft'})
    `;
  } catch (caughtError) {
    error = caughtError;
  }

  expect(error).toBeDefined();
});
