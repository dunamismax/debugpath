# BUILD.md

> **Required operating rule:** any agent who completes work from this plan, changes scope, makes an architectural decision, or discovers a hidden constraint must update this file in the same change set. Keep the boxes current. Do not leave hidden work in chat.

DebugPath is still in active greenfield build mode. This file is the execution authority for taking the repo from a verified skeleton to a serious production investigation tool.

## Current repo truth

- [x] The repo is a Bun workspace monorepo.
- [x] `apps/web` exists as an Astro-owned web app shell.
- [x] `apps/api` exists as an Elysia API shell.
- [x] `packages/contracts` exists for shared Zod contracts.
- [x] `compose.yaml` exists for PostgreSQL, MinIO, and Caddy.
- [x] `Caddyfile` routes `/api/*` to the API and everything else to the Astro app.
- [x] The repo has a root verification surface with `lint`, `typecheck`, `astro:check`, `test`, `build`, and `verify` scripts.
- [x] Vue is intentionally not part of the initial skeleton.
- [x] SQL migrations are implemented.
- [ ] Authentication and workspace membership are implemented.
- [ ] Artifact ingestion is implemented.
- [ ] Search, notes, bundles, and sharing are implemented.

## Product promise

- [ ] Turn messy debugging inputs into one coherent investigation surface.
- [ ] Preserve raw evidence so users can verify the system did not lose context.
- [ ] Normalize enough structure that timelines, search, and correlation are useful immediately.
- [ ] Let users capture human investigation notes alongside machine-parsed evidence.
- [ ] Produce a reviewable debug bundle for asynchronous collaboration.
- [ ] Keep the product self-hostable and operationally boring.

## Scope guardrails

- [x] DebugPath is browser-first and Astro-first.
- [x] Vue is opt-in only for workflows with sustained client-side state such as dense timeline filtering, drag-and-drop upload state, or bundle composition.
- [x] PostgreSQL is the future system of record for metadata, normalized events, notes, bundle manifests, and sharing state.
- [x] Object storage is the future blob layer for raw artifacts and derived exports.
- [x] The product is not a full observability platform, APM replacement, ticketing system, or chat system.
- [x] v1 should stay focused on investigation ingestion, correlation, notes, bundles, and controlled sharing.

## Phase status summary

- [x] Phase 0 - Repo bootstrap and execution guardrails.
- [x] Phase 1 - Application skeleton and local platform.
- [x] Phase 2 - Database foundation and migration runner.
- [ ] Phase 3 - Authentication, workspace model, and investigation shell.
- [ ] Phase 4 - Artifact ingestion MVP.
- [ ] Phase 5 - Normalization pipeline and unified timeline.
- [ ] Phase 6 - Search and exact-match correlation.
- [ ] Phase 7 - Notes, bundle builder, and controlled sharing.
- [ ] Phase 8 - Polish, resilience, and operational hardening.
- [ ] Phase 9 - Deployment, staging, and launch readiness.
- [ ] Phase 10 - Post-launch follow-up.

## Phase 0 - Repo bootstrap and execution guardrails

### Objectives

- [x] Initialize the Bun workspace and root scripts.
- [x] Establish repo structure, code style, env handling, and verification entrypoints.
- [x] Add a README that explains product intent and local development flow.
- [x] Make this `BUILD.md` the live execution manual.

### Checklist

- [x] Root `package.json` with `dev`, `build`, `test`, `typecheck`, and `verify` scripts.
- [x] Bun workspace configuration.
- [x] Biome configuration.
- [x] Root TypeScript configuration.
- [x] Base env example files.
- [x] README with setup and architecture summary.
- [x] Compose file with PostgreSQL, object storage, and Caddy placeholders or initial definitions.

### Exit criteria

- [x] A fresh clone can install dependencies and run the base verification command.
- [x] The repo shape matches the intended app and package layout.
- [x] No hidden toolchain assumptions live only in chat.

### Verification

- [x] `bun install`
- [x] `bun run verify`

## Phase 1 - Application skeleton and local platform

### Objectives

- [x] Create `apps/web`, `apps/api`, and `packages/contracts`.
- [x] Wire Astro, Elysia, Zod, and shared TypeScript configs.
- [x] Keep Astro in charge of routes, layouts, and first-rendered shells from the start.
- [x] Stand up Caddy routing for local integration.
- [x] Bring up PostgreSQL and object storage locally.

### Checklist

- [x] Web app with Astro-owned authenticated and unauthenticated layout placeholders.
- [x] API service with direct and versioned health endpoints.
- [x] Contracts package with initial DTOs and response envelopes.
- [x] Compose services for PostgreSQL and MinIO.
- [x] Caddyfile routing local web and API services.
- [x] Documented rule for when Vue is allowed into the web app.

### Exit criteria

- [x] Web and API can run together through the local reverse proxy.
- [x] Contracts compile cleanly across packages.
- [x] Routes, layouts, and initial page delivery are Astro-owned.
- [x] Local developers can start the stack with one documented flow.

### Verification

- [x] `bun run dev`
- [x] `bun run typecheck`
- [x] `bun run astro:check`
- [x] Caddy smoke test through the documented local stack.

## Phase 2 - Database foundation and migration runner

### Objectives

- [x] Implement SQL migrations and the initial schema.
- [x] Add database connectivity, query helpers, and transaction patterns.
- [x] Establish a seed strategy for local development.

### Checklist

- [x] Migration runner script.
- [x] Initial schema for users, workspaces, investigations, artifacts, notes, ingestion jobs, and basic sharing tables.
- [x] Database access layer with explicit query functions.
- [x] Local seed command.

### Exit criteria

- [x] Schema can be created from scratch in a clean database.
- [x] Migrations are repeatable and deterministic.
- [x] Basic relational constraints protect data integrity.

### Verification

- [x] Run migrations on an empty database.
- [x] Run seeds.
- [x] Run integration tests against local PostgreSQL.

## Phase 3 - Authentication, workspace model, and investigation shell

### Objectives

- [ ] Implement auth, sessions, workspace membership, and investigation CRUD.
- [ ] Deliver the first usable post-login application shell.

### Checklist

- [ ] Login and logout flows.
- [ ] Session middleware.
- [ ] Workspace switcher or personal-workspace default.
- [ ] Astro-rendered investigation list page.
- [ ] Create, edit, and archive investigation flows.
- [ ] Audit trail for important security and sharing actions.

### Exit criteria

- [ ] Users can sign in, create an investigation, and revisit it later.
- [ ] Authorization rules prevent cross-workspace access.
- [ ] Investigation list and shell remain Astro-owned pages.
- [ ] Important account and sharing actions are auditable.

### Verification

- [ ] Unit tests for auth and permission checks.
- [ ] Playwright flow for sign in and create investigation.
- [ ] Negative authorization tests.

## Phase 4 - Artifact ingestion MVP

### Objectives

- [ ] Support upload and paste flows for the initial artifact classes.
- [ ] Preserve raw artifacts in object storage.
- [ ] Track ingestion status and parser outcomes.

### Checklist

- [ ] Multipart upload endpoint.
- [ ] Paste submission endpoint for text artifacts.
- [ ] Artifact list UI with ingest status.
- [ ] Object storage adapter.
- [ ] Checksum and content-type detection.
- [ ] Parser stubs for stack traces, structured logs, HAR files, screenshots metadata, console output, environment details, and repro steps.

### Exit criteria

- [ ] Users can add each supported day-one artifact type to an investigation.
- [ ] Raw content is retrievable for verification.
- [ ] Failed parsing does not destroy or hide the original artifact.

### Verification

- [ ] API tests for upload and paste routes.
- [ ] Storage integration tests.
- [ ] Browser flow for adding artifacts and seeing statuses.

## Phase 5 - Normalization pipeline and unified timeline

### Objectives

- [ ] Turn ingested artifacts into extraction records, normalized timeline events, and correlation keys.
- [ ] Build the first serious investigation view.

### Checklist

- [ ] Parser implementations for the initial supported artifact classes.
- [ ] Timeline event writer.
- [ ] Astro-owned investigation page with a timeline surface.
- [ ] Vue island only if timeline filters or inspectors clearly outgrow plain Astro plus HTML.
- [ ] Event detail panel with source links back to the original artifact.

### Exit criteria

- [ ] One realistic mixed-artifact investigation renders a coherent timeline.
- [ ] Users can trace each normalized event back to its source artifact.
- [ ] Page ownership still sits with Astro even if part of the timeline uses Vue.
- [ ] Parser warnings are visible without breaking the workflow.

### Verification

- [ ] Fixture-based parser tests per artifact type.
- [ ] Integration tests for normalization persistence.
- [ ] Playwright flow for timeline filtering and event inspection.

## Phase 6 - Search and exact-match correlation

### Objectives

- [ ] Ship useful search and first-pass linkage across artifacts.
- [ ] Make request, trace, session, and related identifiers first-class investigation pivots.

### Checklist

- [ ] Extracted-text indexing in PostgreSQL.
- [ ] Correlation-key extraction registry.
- [ ] Search UI with filters and result grouping.
- [ ] Linked-artifact panel for exact-match IDs.

### Exit criteria

- [ ] Search finds cases by note text, filenames, stack trace content, request IDs, and log messages.
- [ ] Users can pivot from a correlation key to matching artifacts and events.
- [ ] Query latency remains acceptable on realistic fixture data.

### Verification

- [ ] Query-plan review for key searches.
- [ ] Integration tests for correlation extraction and lookup.
- [ ] Benchmark on a seeded fixture set.

## Phase 7 - Notes, bundle builder, and controlled sharing

### Objectives

- [ ] Support human investigation notes and publishable debug bundles.
- [ ] Make asynchronous collaboration useful without oversharing.

### Checklist

- [ ] Markdown-capable notes with timestamps and authorship.
- [ ] Note anchors to artifacts or timeline events.
- [ ] Bundle-composition UI with Astro owning the page.
- [ ] Vue only if selection state clearly earns it.
- [ ] Bundle export manifest and renderable share page.
- [ ] Expiring and revocable share links.

### Exit criteria

- [ ] A user can turn an investigation into a reviewable bundle with narrative context.
- [ ] Shared bundles expose only the selected scope.
- [ ] Shared bundle pages remain Astro-owned and do not require a full client app.
- [ ] Revoking a share link immediately blocks access.

### Verification

- [ ] Bundle-generation tests.
- [ ] Auth and access tests for shared versus private resources.
- [ ] Playwright flow for create note, build bundle, open share link, and revoke link.

## Phase 8 - Polish, resilience, and operational hardening

### Objectives

- [ ] Raise the product from functional to launchable.
- [ ] Harden security, performance, and failure handling.

### Checklist

- [ ] Structured logging across web and API.
- [ ] Request IDs and traceability within the product itself.
- [ ] Rate limiting on auth and upload endpoints.
- [ ] File-size limits and content-type controls.
- [ ] Antivirus or malware-scanning decision documented if public uploads are allowed.
- [ ] Background reprocessing path for parser upgrades.
- [ ] Bundle export cleanup and storage lifecycle rules.
- [ ] Accessibility and keyboard-navigation pass.
- [ ] Visual polish pass with consistent tokens and states.

### Exit criteria

- [ ] Common failure modes are handled explicitly and tested.
- [ ] Security-sensitive endpoints have rate limits and audit coverage.
- [ ] UI remains usable for long investigations and large artifact counts.

### Verification

- [ ] Load and soak tests for upload and timeline queries.
- [ ] Security checklist review.
- [ ] Accessibility pass and regression checks.
- [ ] Manual usability review on realistic incident fixtures.

## Phase 9 - Deployment, staging, and launch readiness

### Objectives

- [ ] Produce a boring, repeatable deployment shape.
- [ ] Validate the full system in staging with realistic data.

### Checklist

- [ ] Production Dockerfiles.
- [ ] Compose-based or equivalent single-host deployment shape.
- [ ] Caddy production configuration.
- [ ] Staging environment-variable and secret-handling guidance.
- [ ] Backup and restore procedures for PostgreSQL and object storage.
- [ ] Admin runbook for migrations, bundle storage, and share-link revocation.

### Exit criteria

- [ ] The app can be deployed to a fresh host from documented steps.
- [ ] Backups and restores are verified, not assumed.
- [ ] Staging validates the full critical path before launch.

### Verification

- [ ] Deploy to staging from scratch.
- [ ] Run migrations in staging.
- [ ] Restore a backup into an isolated environment and verify data integrity.
- [ ] Run a full regression pass in staging.

## Phase 10 - Post-launch follow-up

### Objectives

- [ ] Capture operational lessons and prioritize the next real improvements.
- [ ] Remove temporary build-phase scaffolding once the repo leaves greenfield mode.

### Checklist

- [ ] Launch retrospective notes.
- [ ] Prioritized backlog for the next version.
- [ ] Documentation cleanup plan.
- [ ] Explicit decision on when to retire `BUILD.md`.

### Exit criteria

- [ ] First production feedback is reflected in docs and backlog.
- [ ] Temporary greenfield instructions are not left to rot.
- [ ] Current-state docs exist for development and operations.

### Verification

- [ ] Post-launch issue review.
- [ ] Docs review for stale build-specific content.
- [ ] Explicit decision on BUILD retirement timing.

## Cross-phase verification gates

- [x] Root repo gates exist: `bunx biome check .`, `bun run typecheck`, `bun run astro:check`, `bun test`, and build scripts.
- [ ] Parser fixtures should exist for every supported artifact type.
- [x] Database integration tests should cover migrations and core queries.
- [ ] API tests should cover auth, uploads, notes, search, and sharing.
- [ ] Playwright should cover sign in, investigation creation, upload flows, timeline inspection, search, note authoring, and bundle sharing.
- [ ] Regression fixtures should reflect real-world messy inputs, not just ideal examples.

## Definition of done for any completed phase

- [x] Completed phases update code, tests, docs, and env examples together.
- [x] Completed phases satisfy their exit criteria.
- [x] Completed phases have verification expectations checked honestly.
- [x] Completed phases update `BUILD.md` at the same time as the repo change.

## When to retire this file

- [ ] Retire `BUILD.md` only after DebugPath leaves active greenfield build mode.
- [ ] Move enduring current-state guidance into `README.md`, `CONTRIBUTING.md`, `docs/operations.md`, or equivalent before deleting it.
- [ ] Keep roadmap tracking in issues or milestones once the build-phase tracker is no longer the right tool.