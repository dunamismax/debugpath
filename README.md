# DebugPath · debugpath.dev

> Production target: <https://debugpath.dev>
>
> Self-hostable investigation workspace for turning messy production evidence into one coherent debug surface.

DebugPath is a browser-first debug artifact workspace for production investigations. It is being built as a Bun monorepo with an Astro web app, an Elysia API, shared Zod contracts, PostgreSQL metadata, MinIO-backed artifact storage, and Caddy for local integration parity.

## Current status

This repo now has the Phase 3 investigation shell underway:

- `apps/web` serves the public overview, sign-in flow, investigation list, and investigation edit shell as Astro-owned pages.
- `apps/api` now includes session-backed auth routes, workspace-aware investigation CRUD routes, audit-event writes for account and investigation actions, and the PostgreSQL access layer.
- `db/migrations/0001_initial_schema.sql` creates the initial relational model for users, workspaces, investigations, artifacts, notes, ingestion jobs, bundles, and share links.
- `db/migrations/0002_auth_sessions_and_audit.sql` adds password-backed auth, server-side sessions, and audit-event storage.
- `db/scripts/migrate.ts` applies deterministic SQL migrations with checksum tracking.
- `db/scripts/seed.ts` creates a rerunnable local seed graph rooted in `debugpath.dev` sample data and a seed login for the local shell.
- `apps/api/test/integration/auth.integration.test.ts` verifies registration, sign-in, investigation CRUD, and negative authorization checks against PostgreSQL.

Phase 3 is not fully done yet. The app shell is real, but broader audit coverage, browser automation, and the next artifact-ingestion tranche still remain.

## Vue admission rule

Astro owns routes, layouts, page data loading, and first-rendered investigation shells.
Vue is **not** part of the current app shell.
Only introduce Vue for a specific DebugPath workflow when plain Astro plus HTML becomes awkward because of sustained client-side state, such as:

- dense timeline filtering
- inspector state that spans multiple panes
- drag-and-drop upload state that becomes complex
- bundle composition with substantial multi-select interactions

If that threshold is not met, stay with Astro.

## Repo layout

```text
apps/
  api/
  web/
packages/
  contracts/
db/
  migrations/
  scripts/
  seeds/
ops/
  docker/
compose.yaml
Caddyfile
```

## Local development

1. Install dependencies:

   ```bash
   bun install
   ```

2. Optional: copy `.env.example`, `apps/api/.env.example`, or `apps/web/.env.example` into local `.env` files if you need to override defaults.

3. Start the local stack:

   ```bash
   bun run dev
   ```

   That command:

   - starts or refreshes PostgreSQL on `localhost:5433`, MinIO, and Caddy via Docker Compose
   - runs the Elysia API on `http://localhost:3000`
   - runs the Astro app on `http://localhost:4321`
   - exposes the integrated entrypoint at `http://localhost:8080`

   `bun run dev` auto-detects a host LAN IP for the Caddy upstreams on this macOS setup. If another host needs different routing, override `DEBUGPATH_WEB_UPSTREAM` and `DEBUGPATH_API_UPSTREAM` in `.env`.

4. Apply the database schema and seed local data:

   ```bash
   bun run db:migrate
   bun run db:seed
   ```

5. Sign in through the integrated app shell at `http://localhost:8080/login`.

   Local seeded credentials after `bun run db:seed`:

   - email: `owner@debugpath.dev`
   - password: `debugpath-dev-password`

6. Stop infrastructure when you are done:

   ```bash
   bun run infra:down
   ```

## Verification

The repo verification flow remains:

```bash
bun run verify
```

Current build verification for the database-backed shell also expects PostgreSQL coverage:

```bash
bun run db:migrate
bun run db:seed
bun run test:api:integration
```

## Current architecture boundaries

- `apps/web` owns Astro pages, layouts, sign-in rendering, investigation list rendering, and the first investigation CRUD shell.
- `apps/api` owns auth, session handling, workspace-aware investigation routes, audit-event writes, versioned API entrypoints, and the database access layer.
- `packages/contracts` owns request and response contracts.
- PostgreSQL is the source of truth for accounts, workspaces, investigations, and the wider normalized investigation model.
- Object storage remains the blob layer once artifact ingestion lands in Phase 4.
