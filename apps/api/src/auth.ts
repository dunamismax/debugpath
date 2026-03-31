import type { SessionUser } from '@debugpath/contracts';

import { getDatabase } from './db/client';
import {
  type AccessibleWorkspaceRecord,
  createUserSession,
  deleteExpiredUserSessions,
  findAuthenticatedSessionByTokenHash,
  findDefaultWorkspaceForUser,
  findWorkspaceBySlug,
  listWorkspacesForUser,
  touchUserSession,
  upsertWorkspace,
  upsertWorkspaceMembership,
} from './db/repositories';
import { parseCookies, serializeCookie } from './lib/http';
import { createSessionToken, sha256Hex, slugify } from './lib/security';

const sessionCookieName = Bun.env.SESSION_COOKIE_NAME ?? 'debugpath_session';
const sessionTtlSeconds = Number(Bun.env.SESSION_TTL_SECONDS ?? 60 * 60 * 24 * 30);
const useSecureCookies = () => (Bun.env.APP_ENV ?? 'development') === 'production';

export interface AuthContext {
  sessionId: string;
  user: SessionUser;
  currentWorkspace: AccessibleWorkspaceRecord | null;
  workspaces: AccessibleWorkspaceRecord[];
}

const personalWorkspaceNameFor = (user: SessionUser) =>
  user.displayName?.trim() ? `${user.displayName} workspace` : `${user.email} workspace`;

const personalWorkspaceSlugBaseFor = (user: SessionUser) => {
  const source = user.displayName?.trim() || user.email.split('@')[0] || 'debugpath';
  return `${slugify(source)}-workspace`;
};

export const ensurePersonalWorkspaceForUser = async (
  user: SessionUser,
  db = getDatabase()
): Promise<AccessibleWorkspaceRecord> => {
  const existingWorkspace = await findDefaultWorkspaceForUser(db, user.id);
  if (existingWorkspace) {
    return existingWorkspace;
  }

  const baseSlug = personalWorkspaceSlugBaseFor(user);
  let candidate = baseSlug;
  let suffix = 0;

  while (await findWorkspaceBySlug(db, candidate)) {
    suffix += 1;
    candidate = `${baseSlug}-${suffix}`;
  }

  const workspace = await upsertWorkspace(db, {
    name: personalWorkspaceNameFor(user),
    ownerUserId: user.id,
    slug: candidate,
  });

  await upsertWorkspaceMembership(db, {
    role: 'owner',
    userId: user.id,
    workspaceId: workspace.id,
  });

  return {
    ...workspace,
    role: 'owner',
  };
};

export const resolveAuthContext = async (request: Request, db = getDatabase()) => {
  const cookies = parseCookies(request.headers.get('cookie'));
  const rawToken = cookies.get(sessionCookieName);

  if (!rawToken) {
    return null;
  }

  await deleteExpiredUserSessions(db);

  const session = await findAuthenticatedSessionByTokenHash(db, sha256Hex(rawToken));
  if (!session) {
    return null;
  }

  await touchUserSession(db, session.id);

  const user = {
    id: session.userId,
    email: session.email,
    displayName: session.displayName,
  } satisfies SessionUser;

  const currentWorkspace = await ensurePersonalWorkspaceForUser(user, db);
  const workspaces = await listWorkspacesForUser(db, user.id);

  return {
    sessionId: session.id,
    user,
    currentWorkspace,
    workspaces,
  } satisfies AuthContext;
};

export const createSessionCookie = async (userId: string, db = getDatabase()) => {
  const rawToken = createSessionToken();
  const tokenHash = sha256Hex(rawToken);
  const expiresAt = new Date(Date.now() + sessionTtlSeconds * 1000).toISOString();

  await deleteExpiredUserSessions(db);
  await createUserSession(db, {
    userId,
    tokenHash,
    expiresAt,
  });

  return serializeCookie({
    name: sessionCookieName,
    value: rawToken,
    maxAge: sessionTtlSeconds,
    secure: useSecureCookies(),
  });
};

export const clearSessionCookie = () =>
  serializeCookie({
    name: sessionCookieName,
    value: '',
    maxAge: 0,
    secure: useSecureCookies(),
  });

export const getSessionCookieName = () => sessionCookieName;
