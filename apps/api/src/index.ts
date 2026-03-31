import {
  type CreateInvestigationInput,
  type LoginInput,
  type RegisterInput,
  type UpdateInvestigationInput,
  apiBasePath,
  apiVersion,
  appShellSchema,
  authStatusSchema,
  createInvestigationInputSchema,
  createOkEnvelope,
  healthPayloadSchema,
  investigationDetailSchema,
  loginInputSchema,
  registerInputSchema,
  updateInvestigationInputSchema,
} from '@debugpath/contracts';
import { Elysia } from 'elysia';

import {
  clearSessionCookie,
  createSessionCookie,
  ensurePersonalWorkspaceForUser,
  getSessionCookieName,
  resolveAuthContext,
} from './auth';
import { getDatabase, withTransaction } from './db/client';
import {
  type InvestigationSeverityRecord,
  archiveInvestigationByIdForUser,
  createAuditEvent,
  createInvestigationForWorkspace,
  createUser,
  deleteUserSessionByTokenHash,
  findInvestigationByIdForUser,
  findUserByEmail,
  listInvestigationsByWorkspaceForUser,
  updateInvestigationByIdForUser,
  updateUserLastLoginAt,
} from './db/repositories';
import {
  errorResponse,
  jsonResponse,
  parseCookies,
  redirectResponse,
  safeRedirectPath,
} from './lib/http';
import { sha256Hex, slugify } from './lib/security';

const startedAt = Date.now();
const port = Number(Bun.env.PORT ?? 3000);
const hostname = Bun.env.HOST ?? '0.0.0.0';

const buildHealthResponse = () =>
  createOkEnvelope(healthPayloadSchema, {
    service: 'api',
    version: apiVersion,
    environment: Bun.env.APP_ENV ?? 'development',
    uptimeSeconds: Math.floor((Date.now() - startedAt) / 1000),
    timestamp: new Date().toISOString(),
  });

const toTrimmedString = (value: FormDataEntryValue | null) => {
  if (typeof value !== 'string') {
    return '';
  }

  return value.trim();
};

const toOptionalString = (value: FormDataEntryValue | null) => {
  const trimmed = toTrimmedString(value);
  return trimmed ? trimmed : null;
};

const toSeverity = (value: FormDataEntryValue | null): InvestigationSeverityRecord => {
  const trimmed = toTrimmedString(value);
  if (!trimmed) {
    return null;
  }

  if (trimmed === 'low' || trimmed === 'medium' || trimmed === 'high' || trimmed === 'critical') {
    return trimmed;
  }

  return null;
};

const getRequestIpAddress = (request: Request) =>
  request.headers
    .get('x-forwarded-for')
    ?.split(',')
    .map((value) => value.trim())
    .find(Boolean) ?? null;

const toIsoString = (value: string | Date | null) =>
  value instanceof Date ? value.toISOString() : value;

const serializeInvestigation = (investigation: {
  archivedAt: string | Date | null;
  createdAt: string | Date;
  createdByUserId: string;
  id: string;
  severity: InvestigationSeverityRecord;
  slug: string;
  status: 'draft' | 'active' | 'resolved' | 'archived';
  summary: string | null;
  title: string;
  updatedAt: string | Date;
  workspaceId: string;
}) => ({
  archivedAt: toIsoString(investigation.archivedAt),
  createdAt: toIsoString(investigation.createdAt) ?? new Date(0).toISOString(),
  createdByUserId: investigation.createdByUserId,
  id: investigation.id,
  severity: investigation.severity,
  slug: investigation.slug,
  status: investigation.status,
  summary: investigation.summary,
  title: investigation.title,
  updatedAt: toIsoString(investigation.updatedAt) ?? new Date(0).toISOString(),
  workspaceId: investigation.workspaceId,
});

const requireApiAuth = async (request: Request) => {
  const auth = await resolveAuthContext(request);
  if (!auth || !auth.currentWorkspace) {
    return null;
  }

  return auth;
};

const createInvestigationSlug = (title: string) => `${slugify(title)}-${Date.now().toString(36)}`;

const unauthorizedRedirect = (request: Request, fallback = '/login?error=auth_required') => {
  const url = new URL(request.url);
  return redirectResponse(
    `${fallback}&redirectTo=${encodeURIComponent(url.pathname + url.search)}`
  );
};

export const app = new Elysia()
  .get('/health', () => buildHealthResponse())
  .group(apiBasePath, (group) =>
    group
      .get('/health', () => buildHealthResponse())
      .get('/auth/status', async ({ request }) => {
        const auth = await resolveAuthContext(request);

        return createOkEnvelope(authStatusSchema, {
          authenticated: Boolean(auth),
          user: auth?.user ?? null,
          currentWorkspace: auth?.currentWorkspace
            ? {
                id: auth.currentWorkspace.id,
                slug: auth.currentWorkspace.slug,
                name: auth.currentWorkspace.name,
                role: auth.currentWorkspace.role,
              }
            : null,
        });
      })
      .post('/auth/register', async ({ request }) => {
        const form = await request.formData();
        const redirectTo = safeRedirectPath(toTrimmedString(form.get('redirectTo')), '/app');
        const db = getDatabase();

        let input: RegisterInput;
        try {
          input = registerInputSchema.parse({
            displayName: toTrimmedString(form.get('displayName')),
            email: toTrimmedString(form.get('email')).toLowerCase(),
            password: toTrimmedString(form.get('password')),
          });
        } catch {
          return redirectResponse('/login?error=invalid_registration');
        }

        const existingUser = await findUserByEmail(db, input.email);
        if (existingUser) {
          return redirectResponse('/login?error=account_exists');
        }

        const createdUser = await withTransaction(db, async (tx) => {
          const user = await createUser(tx, {
            displayName: input.displayName,
            email: input.email,
            passwordHash: await Bun.password.hash(input.password),
          });

          const workspace = await ensurePersonalWorkspaceForUser(
            {
              id: user.id,
              email: user.email,
              displayName: user.displayName,
            },
            tx
          );

          await createAuditEvent(tx, {
            action: 'auth.registered',
            actorUserId: user.id,
            ipAddress: getRequestIpAddress(request),
            metadata: {
              email: user.email,
            },
            targetId: user.id,
            targetType: 'user',
            userAgent: request.headers.get('user-agent'),
            workspaceId: workspace.id,
          });

          return user;
        });

        const sessionCookie = await createSessionCookie(createdUser.id, db);
        return redirectResponse(redirectTo, {
          'set-cookie': sessionCookie,
        });
      })
      .post('/auth/login', async ({ request }) => {
        const form = await request.formData();
        const redirectTo = safeRedirectPath(toTrimmedString(form.get('redirectTo')), '/app');
        const db = getDatabase();

        let input: LoginInput;
        try {
          input = loginInputSchema.parse({
            email: toTrimmedString(form.get('email')).toLowerCase(),
            password: toTrimmedString(form.get('password')),
          });
        } catch {
          return redirectResponse('/login?error=invalid_credentials');
        }

        const user = await findUserByEmail(db, input.email);
        const passwordMatches = user?.passwordHash
          ? await Bun.password.verify(input.password, user.passwordHash)
          : false;

        if (!user || !passwordMatches) {
          return redirectResponse('/login?error=invalid_credentials');
        }

        await withTransaction(db, async (tx) => {
          await updateUserLastLoginAt(tx, user.id);
          const workspace = await ensurePersonalWorkspaceForUser(
            {
              id: user.id,
              email: user.email,
              displayName: user.displayName,
            },
            tx
          );
          await createAuditEvent(tx, {
            action: 'auth.logged_in',
            actorUserId: user.id,
            ipAddress: getRequestIpAddress(request),
            metadata: {
              email: user.email,
            },
            targetId: user.id,
            targetType: 'user',
            userAgent: request.headers.get('user-agent'),
            workspaceId: workspace.id,
          });
        });

        const sessionCookie = await createSessionCookie(user.id, db);
        return redirectResponse(redirectTo, {
          'set-cookie': sessionCookie,
        });
      })
      .post('/auth/logout', async ({ request }) => {
        const form = await request.formData();
        const redirectTo = safeRedirectPath(
          toTrimmedString(form.get('redirectTo')),
          '/login?logged_out=1'
        );
        const db = getDatabase();
        const auth = await resolveAuthContext(request, db);
        const rawToken = parseCookies(request.headers.get('cookie')).get(getSessionCookieName());

        if (rawToken) {
          await deleteUserSessionByTokenHash(db, sha256Hex(rawToken));
        }

        if (auth) {
          await createAuditEvent(db, {
            action: 'auth.logged_out',
            actorUserId: auth.user.id,
            ipAddress: getRequestIpAddress(request),
            metadata: {
              email: auth.user.email,
            },
            targetId: auth.user.id,
            targetType: 'user',
            userAgent: request.headers.get('user-agent'),
            workspaceId: auth.currentWorkspace?.id ?? null,
          });
        }

        return redirectResponse(redirectTo, {
          'set-cookie': clearSessionCookie(),
        });
      })
      .get('/app-shell', async ({ request }) => {
        const auth = await requireApiAuth(request);
        if (!auth || !auth.currentWorkspace) {
          return errorResponse(401, 'unauthorized', 'Authentication is required.');
        }

        const investigations = await listInvestigationsByWorkspaceForUser(
          getDatabase(),
          auth.currentWorkspace.id,
          auth.user.id
        );

        return jsonResponse(
          createOkEnvelope(appShellSchema, {
            currentUser: auth.user,
            currentWorkspace: {
              id: auth.currentWorkspace.id,
              slug: auth.currentWorkspace.slug,
              name: auth.currentWorkspace.name,
              role: auth.currentWorkspace.role,
            },
            investigations: investigations.map((investigation) =>
              serializeInvestigation({
                ...investigation,
                createdByUserId: investigation.createdByUserId,
              })
            ),
            workspaces: auth.workspaces.map((workspace) => ({
              id: workspace.id,
              slug: workspace.slug,
              name: workspace.name,
              role: workspace.role,
            })),
          })
        );
      })
      .get('/investigations/:id', async ({ params, request }) => {
        const auth = await requireApiAuth(request);
        if (!auth) {
          return errorResponse(401, 'unauthorized', 'Authentication is required.');
        }

        const investigation = await findInvestigationByIdForUser(
          getDatabase(),
          params.id,
          auth.user.id
        );
        if (!investigation) {
          return errorResponse(404, 'not_found', 'Investigation not found.');
        }

        return jsonResponse(
          createOkEnvelope(investigationDetailSchema, serializeInvestigation(investigation))
        );
      })
      .post('/investigations', async ({ request }) => {
        const auth = await resolveAuthContext(request);
        if (!auth || !auth.currentWorkspace) {
          return unauthorizedRedirect(request);
        }

        const form = await request.formData();
        const redirectTo = safeRedirectPath(toTrimmedString(form.get('redirectTo')), '/app');
        const workspace = auth.currentWorkspace;

        let input: CreateInvestigationInput;
        try {
          input = createInvestigationInputSchema.parse({
            severity: toSeverity(form.get('severity')),
            summary: toOptionalString(form.get('summary')),
            title: toTrimmedString(form.get('title')),
          });
        } catch {
          return redirectResponse('/app?error=invalid_investigation');
        }

        const db = getDatabase();
        const investigation = await withTransaction(db, async (tx) => {
          const created = await createInvestigationForWorkspace(tx, {
            createdByUserId: auth.user.id,
            severity: input.severity ?? 'high',
            slug: createInvestigationSlug(input.title),
            status: 'active',
            summary: input.summary ?? null,
            title: input.title,
            workspaceId: workspace.id,
          });

          if (!created) {
            return null;
          }

          await createAuditEvent(tx, {
            action: 'investigation.created',
            actorUserId: auth.user.id,
            ipAddress: getRequestIpAddress(request),
            targetId: created.id,
            targetType: 'investigation',
            userAgent: request.headers.get('user-agent'),
            workspaceId: workspace.id,
          });

          return created;
        });

        if (!investigation) {
          return redirectResponse('/app?error=forbidden');
        }

        return redirectResponse(
          safeRedirectPath(redirectTo, `/app/investigations/${investigation.id}`)
        );
      })
      .post('/investigations/:id', async ({ params, request }) => {
        const auth = await resolveAuthContext(request);
        if (!auth || !auth.currentWorkspace) {
          return unauthorizedRedirect(request);
        }

        const form = await request.formData();
        const redirectTo = safeRedirectPath(
          toTrimmedString(form.get('redirectTo')),
          `/app/investigations/${params.id}`
        );

        let input: UpdateInvestigationInput;
        try {
          input = updateInvestigationInputSchema.parse({
            severity: toSeverity(form.get('severity')),
            status: toTrimmedString(form.get('status')) || 'active',
            summary: toOptionalString(form.get('summary')),
            title: toTrimmedString(form.get('title')),
          });
        } catch {
          return redirectResponse(`${redirectTo}?error=invalid_investigation`);
        }

        const db = getDatabase();
        const investigation = await withTransaction(db, async (tx) => {
          const updated = await updateInvestigationByIdForUser(tx, params.id, auth.user.id, {
            archivedAt: input.status === 'archived' ? new Date().toISOString() : null,
            severity: input.severity ?? null,
            status: input.status ?? 'active',
            summary: input.summary ?? null,
            title: input.title,
          });

          if (!updated) {
            return null;
          }

          await createAuditEvent(tx, {
            action:
              updated.status === 'archived' ? 'investigation.archived' : 'investigation.updated',
            actorUserId: auth.user.id,
            ipAddress: getRequestIpAddress(request),
            targetId: updated.id,
            targetType: 'investigation',
            userAgent: request.headers.get('user-agent'),
            workspaceId: updated.workspaceId,
          });

          return updated;
        });

        if (!investigation) {
          return redirectResponse('/app?error=not_found');
        }

        return redirectResponse(`${redirectTo}?saved=1`);
      })
      .post('/investigations/:id/archive', async ({ params, request }) => {
        const auth = await resolveAuthContext(request);
        if (!auth || !auth.currentWorkspace) {
          return unauthorizedRedirect(request);
        }

        const form = await request.formData();
        const redirectTo = safeRedirectPath(toTrimmedString(form.get('redirectTo')), '/app');
        const db = getDatabase();

        const investigation = await withTransaction(db, async (tx) => {
          const archived = await archiveInvestigationByIdForUser(tx, params.id, auth.user.id);
          if (!archived) {
            return null;
          }

          await createAuditEvent(tx, {
            action: 'investigation.archived',
            actorUserId: auth.user.id,
            ipAddress: getRequestIpAddress(request),
            targetId: archived.id,
            targetType: 'investigation',
            userAgent: request.headers.get('user-agent'),
            workspaceId: archived.workspaceId,
          });

          return archived;
        });

        if (!investigation) {
          return redirectResponse('/app?error=not_found');
        }

        return redirectResponse(`${redirectTo}?archived=1`);
      })
  );

if (import.meta.main) {
  app.listen({
    port,
    hostname,
  });

  console.log(`debugpath api listening on http://${hostname}:${port}${apiBasePath}/health`);
}
