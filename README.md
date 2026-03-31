# DebugPath

DebugPath is a browser-first debug artifact workspace for production investigations. It is being built as a Bun monorepo with an Astro web app, an Elysia API, shared Zod contracts, PostgreSQL metadata, MinIO-backed artifact storage, and Caddy for local integration parity.

## Phase 1 status

This repo now has the Phase 1 application skeleton in place:

- `apps/web` is an Astro app with explicit unauthenticated and authenticated layout placeholders.
- `apps/api` is an Elysia service with direct and versioned health routes.
- `packages/contracts` holds shared Zod contracts and response envelopes.
- `compose.yaml` starts PostgreSQL, MinIO, and Caddy.
- `Caddyfile` routes `/api/*` to the API and everything else to the Astro app.

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

4. Stop infrastructure when you are done:

   ```bash
   bun run infra:down
   ```

## Verification

The base repo verification flow is:

```bash
bun run verify
```

Phase 1 verification also expects:

```bash
bun run dev
bun run typecheck
bun run astro:check
docker compose exec caddy sh -lc 'curl -fsS http://127.0.0.1:8080/ >/dev/null && curl -fsS http://127.0.0.1:8080/api/v1/health >/dev/null && echo CADDY_SMOKE_OK'
```

## Current architecture boundaries

- `apps/web` owns Astro pages, layouts, placeholder auth shell routing, and first render.
- `apps/api` owns service routes and versioned API entrypoints.
- `packages/contracts` owns request and response contracts.
- PostgreSQL remains the source of truth for metadata once migrations land in Phase 2.
- Object storage remains the blob layer once artifact ingestion lands in Phase 4.
