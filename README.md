# DebugPath · debugpath.dev

> Self-hostable investigation workspace for turning messy production evidence into one coherent debug surface.

DebugPath is a browser-first debug artifact workspace for production investigations. It is being built as a Bun monorepo with an Astro web app, an Elysia API, shared Zod contracts, PostgreSQL metadata, MinIO-backed artifact storage, and Caddy for local integration parity.

## Phase 2 status

This repo now has the Phase 2 database foundation in place:

- `apps/web` stays Astro-owned for routes, layouts, and first render.
- `apps/api` now includes a PostgreSQL access layer with explicit query functions and transaction helpers.
- `db/migrations/0001_initial_schema.sql` creates the initial relational model for users, workspaces, investigations, artifacts, notes, ingestion jobs, bundles, and share links.
- `db/scripts/migrate.ts` applies deterministic SQL migrations with checksum tracking.
- `db/scripts/seed.ts` creates a rerunnable local seed graph rooted in `debugpath.dev` sample data.
- `apps/api/test/integration/database.integration.test.ts` verifies migrations, seed idempotency, and relational constraints against PostgreSQL.

## Vue admission rule

Astro owns routes, layouts, page data loading, and first-rendered investigation shells.
Vue is **not** part of the initial skeleton.
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

5. Stop infrastructure when you are done:

   ```bash
   bun run infra:down
   ```

## Verification

The base repo verification flow is still:

```bash
bun run verify
```

Phase 2 verification also expects local PostgreSQL coverage:

```bash
bun run db:migrate
bun run db:seed
bun run test:api:integration
```

## Current architecture boundaries

- `apps/web` owns Astro pages, layouts, placeholder auth shell routing, and first render.
- `apps/api` owns service routes, versioned API entrypoints, and the database access layer.
- `packages/contracts` owns request and response contracts.
- PostgreSQL is now the source of truth for metadata and normalized investigation structure.
- Object storage remains the blob layer once artifact ingestion lands in Phase 4.
