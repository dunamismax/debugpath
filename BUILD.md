# BUILD.md

## Status: greenfield build plan

> **Mandatory rule for every future agent:** if you complete work from this document, change scope, make an architectural decision, discover a hidden constraint, or replace a planned step with a better one, update `BUILD.md` in the same change set. This file must stay current. Do not treat it as a historical artifact or a stale wish list.

## How to use this document

- [ ] Read this file before making repo changes.
- [ ] Keep checkboxes accurate. Mark completed items when work is actually done and verified.
- [ ] If a phase is split, add sub-items rather than leaving hidden work in chat.
- [ ] If a decision changes the plan, update the affected section, acceptance criteria, and verification steps.
- [ ] Keep the plan implementation-ready. Replace vague text with current truth.
- [ ] Do not build side systems that are not justified by this product.

## Product vision

DebugPath is a browser-first debug artifact workspace for engineers investigating production problems. It accepts messy, real-world debugging inputs such as stack traces, structured logs, HAR files, screenshots, console output, environment details, and repro steps. It turns them into one coherent investigation surface with:

- [ ] a unified event timeline
- [ ] searchable artifacts and metadata
- [ ] linked request IDs, session IDs, trace IDs, user IDs, and related correlation handles
- [ ] annotated investigation notes
- [ ] a shareable debug bundle that captures the evidence and narrative of an incident

The product should feel like a serious engineering tool, not a generic file uploader.

### Primary users

- [ ] application engineers debugging production incidents
- [ ] support and QA staff collecting repro details for engineers
- [ ] team leads who need a shareable package of evidence and conclusions

### Product goals

- [ ] Make it fast to ingest heterogeneous debug artifacts into one case workspace.
- [ ] Normalize enough structure that timelines, search, and correlation become useful immediately.
- [ ] Preserve raw uploads so users can verify the system did not lose context.
- [ ] Let users capture human investigation notes alongside machine-parsed evidence.
- [ ] Generate a shareable debug bundle that can be reviewed asynchronously.
- [ ] Keep the product self-hostable and operationally boring.

### Non-goals for v1

- [ ] No live log streaming or full observability platform ambitions.
- [ ] No full APM replacement.
- [ ] No automatic root cause claims presented as fact.
- [ ] No multi-tenant enterprise admin feature sprawl beyond what is needed for small teams.
- [ ] No complex workflow engine, ticketing system, or chat system.
- [ ] No native mobile apps.
- [ ] No OCR-heavy screenshot intelligence in v1 beyond basic metadata extraction.

## Product shape and opinionated scope

### Core workflow

- [ ] User creates or opens an investigation.
- [ ] User uploads files or pastes raw inputs.
- [ ] System stores original artifacts, extracts metadata, normalizes parseable events, and records ingestion status.
- [ ] System correlates identifiers across artifacts.
- [ ] UI presents a timeline, artifact browser, search, and notes panel.
- [ ] User selects artifacts and notes to publish into a shareable debug bundle.

### Day one supported artifact classes

- [ ] stack traces pasted as text
- [ ] structured logs as JSON, NDJSON, or line-delimited text
- [ ] HAR files
- [ ] screenshots and other investigation images
- [ ] console output pasted as text
- [ ] environment details captured through structured forms and freeform text
- [ ] repro steps captured as ordered text notes

### V1 quality bar

- [ ] One investigation can contain many uploads, pasted artifacts, notes, and timeline events.
- [ ] Raw artifact content is preserved.
- [ ] Parsed metadata is queryable.
- [ ] Timeline ordering is deterministic even when timestamps are partial or inconsistent.
- [ ] Search can find investigations by artifact content, IDs, filenames, note text, and key metadata.
- [ ] Sharing can expose a bundle intentionally without exposing the whole account.

## Stack and architecture direction

This repo should follow the TypeScript full-stack Bun web lane, with Astro as the default browser surface. Vue is optional. Only introduce Vue where a DebugPath workflow has enough sustained client-side state or interaction complexity to clearly earn it.

### Required stack

- [x] TypeScript across frontend, backend, and shared contracts
- [x] Bun workspace monorepo
- [x] Astro for pages, routing, layouts, and server-first rendering
- [x] Vue 3 only for substantial interactive islands such as dense timeline filtering, inspectors, upload state, or bundle composition when Astro plus HTML becomes awkward
- [x] Plain CSS with design tokens
- [x] Elysia for API and backend services
- [x] Zod for request, response, and ingestion contract validation
- [ ] PostgreSQL as the system of record
- [ ] Raw SQL first with thin helpers and checked-in SQL migrations
- [ ] Object storage interface for uploaded artifacts, starting with local S3-compatible development via MinIO or equivalent
- [x] Docker Compose for local orchestration
- [x] Caddy for reverse proxy and local-production parity
- [ ] Biome, TypeScript checks, Astro checks, bun test, and Playwright as the quality baseline

### Repo target layout

```text
apps/
  web/
  api/
packages/
  contracts/
  ui/        # optional, only after shared interactive components are real
  config/    # optional, only if shared tooling config earns a package
db/
  migrations/
  seeds/
ops/
  docker/
  scripts/
compose.yaml
Caddyfile
```

### Architectural principles

- [ ] Preserve raw uploads and parsed derivatives separately.
- [ ] Treat PostgreSQL as the truth source for metadata, normalized events, notes, bundle manifests, and share state.
- [ ] Treat object storage as the blob layer for original files and derived exports.
- [ ] Keep the first production deployment shape simple: web, api, postgres, object storage, caddy.
- [ ] Keep background work in the same repo and same language. Start with a worker process only when ingestion latency or export work justifies it.
- [ ] Put shared DTOs, parse result schemas, and identifier shapes in `packages/contracts`.
- [ ] Keep Astro in charge of routes, page composition, page data loading, and share-page rendering. Do not drift into a client-heavy SPA.
- [ ] Add Vue only as targeted islands inside Astro pages when a specific DebugPath workflow truly needs richer client state.
- [ ] Use progressive disclosure in the UI. Investigation clarity matters more than dashboard theater.

## System domains

### Domain objects

- [ ] **User**: authenticated person using DebugPath.
- [ ] **Workspace**: team or personal container for investigations.
- [ ] **Investigation**: a debugging case with status, title, description, tags, and participants.
- [ ] **Artifact**: raw uploaded or pasted item such as a HAR file, stack trace, screenshot, or repro note.
- [ ] **Artifact extraction**: parsed metadata and structured fields extracted from an artifact.
- [ ] **Timeline event**: normalized event derived from one artifact or note.
- [ ] **Correlation key**: request ID, trace ID, session ID, user ID, hostname, release SHA, and similar linking handles.
- [ ] **Note**: investigator-authored annotation or conclusion.
- [ ] **Bundle**: immutable or versioned export of selected artifacts, timeline slices, and notes.
- [ ] **Share link**: access mechanism for a bundle or investigation subset.
- [ ] **Ingestion job**: status record for parsing, normalization, thumbnail generation, and bundle creation.

### System boundaries

- [ ] `apps/web` owns Astro pages, routing, layouts, server-rendered investigation shells, upload flows, search UI, and bundle views.
- [ ] Vue islands in `apps/web` are optional and should be limited to interaction-heavy investigation tools such as timeline filters, inspectors, drag-and-drop upload state, or bundle composition.
- [ ] `apps/api` owns auth, data APIs, ingestion endpoints, bundle generation, and background entrypoints.
- [ ] `packages/contracts` owns Zod schemas and DTOs for requests, responses, normalized artifacts, and search results.
- [ ] `packages/ui` owns reusable UI primitives only after duplication becomes real.
- [ ] `db/migrations` owns SQL schema evolution.

## Data model and migration plan

### Initial schema direction

- [ ] `users`
- [ ] `workspaces`
- [ ] `workspace_members`
- [ ] `investigations`
- [ ] `artifacts`
- [ ] `artifact_contents` or external blob references metadata
- [ ] `artifact_extractions`
- [ ] `timeline_events`
- [ ] `correlation_keys`
- [ ] `artifact_correlation_keys`
- [ ] `notes`
- [ ] `bundle_exports`
- [ ] `bundle_items`
- [ ] `share_links`
- [ ] `ingestion_jobs`
- [ ] `audit_events`

### Table intent

- [ ] `investigations` store lifecycle fields such as status, severity, owner, summary, and timestamps.
- [ ] `artifacts` store artifact type, source type, filename, mime type, size, storage key, checksum, investigation linkage, ingest status, and submitted-by metadata.
- [ ] `artifact_extractions` store parser version, extraction JSON, parse warnings, and confidence or completeness markers.
- [ ] `timeline_events` store normalized timestamp, sort timestamp, source artifact ID, event kind, summary text, severity, environment, actor, service, and lightweight event JSON.
- [ ] `correlation_keys` store normalized key type plus normalized value, with strict uniqueness scoped appropriately.
- [ ] join tables connect artifacts and timeline events to correlation keys.
- [ ] `notes` support plain text or markdown, anchored to an investigation and optionally to an artifact or timeline event.
- [ ] `bundle_exports` store bundle version, creator, status, manifest, export path, and expiration state.
- [ ] `share_links` define public versus authenticated access, scope, expiry, revocation, and secret token hashes.

### Migration rules

- [ ] Use checked-in SQL migrations only.
- [ ] Name migrations sequentially and descriptively.
- [ ] Never edit applied migrations in place.
- [ ] Add rollback notes in migration comments when destructive changes are unavoidable.
- [ ] Keep seed data minimal and deterministic.

### Storage strategy

- [ ] Store raw file bytes in object storage, not in PostgreSQL large objects.
- [ ] Store pasted text artifacts in object storage too once saved, while retaining essential searchable copies or extracted text in PostgreSQL.
- [ ] Compute checksums for deduplication and artifact integrity checks.
- [ ] Generate derivative records for previews, thumbnails, and normalized text when needed.

### Indexing and search posture

- [ ] Use PostgreSQL full text search for v1 across note text, extracted text, filenames, and investigation summaries.
- [ ] Add targeted B-tree indexes for investigation status, created_at, normalized timestamps, artifact type, and correlation key lookups.
- [ ] Add GIN indexes where JSONB and full text queries justify them.
- [ ] Delay Elasticsearch or OpenSearch unless PostgreSQL clearly fails real use cases.

## Auth and sharing model

### Authentication

- [ ] Start with server-side sessions and secure cookies.
- [ ] Support email plus password or magic link first. Choose one and document the decision before implementation.
- [ ] Reserve SSO and passkeys for later phases unless required by launch customer needs.
- [ ] Keep auth local to the product unless a hosted identity provider materially reduces risk without undermining self-hosting.

### Authorization

- [ ] Workspace-level membership roles: owner, editor, viewer.
- [ ] Investigation access inherits from workspace membership unless explicitly shared.
- [ ] Bundle sharing is narrower than workspace access by default.
- [ ] Audit all share creation, revocation, and bundle publication actions.

### Sharing

- [ ] Support private investigations by default.
- [ ] Support share links for specific bundles, not raw investigation-wide public exposure, in the first launch.
- [ ] Support expiring links and manual revocation.
- [ ] Support redacted bundles later if launch scope is at risk.

## Artifact ingestion and normalization roadmap

### Ingestion principles

- [ ] Ingestion must be idempotent for retried uploads.
- [ ] Raw artifact preservation is mandatory.
- [ ] Parsing failures must degrade gracefully and still preserve the raw artifact.
- [ ] Parser versions must be tracked so future reprocessing is possible.
- [ ] Normalization should produce explainable outputs, not magic summaries.

### Artifact-specific v1 roadmap

#### Stack traces

- [ ] Detect language or runtime when possible.
- [ ] Extract exception type, message, frames, file paths, line numbers, and obvious request or trace IDs.
- [ ] Generate timeline events for the thrown error and surrounding context when timestamps are present.

#### Structured logs

- [ ] Support JSON object arrays, NDJSON, and common line-oriented structured log formats.
- [ ] Extract timestamp, severity, message, service, environment, host, request or trace identifiers, and payload fields.
- [ ] Normalize each log line or event into timeline events where feasible.

#### HAR files

- [ ] Extract pages, requests, responses, timings, headers, cookies, query params, and status codes.
- [ ] Generate request and response timeline entries.
- [ ] Highlight slow requests, error responses, and repeated failures.

#### Screenshots

- [ ] Store image metadata and generate safe thumbnails.
- [ ] Support investigator notes anchored to screenshots.
- [ ] Defer OCR unless it becomes necessary for a concrete launch requirement.

#### Console output

- [ ] Preserve line order.
- [ ] Attempt timestamp and severity extraction when patterns are obvious.
- [ ] Link identifiable request, session, or trace values.

#### Environment details and repro steps

- [ ] Capture structured fields such as app version, commit SHA, environment, browser, OS, locale, and feature flags.
- [ ] Capture repro steps as ordered list items that can also appear on the timeline.

### Normalization outputs

- [ ] Canonical artifact record
- [ ] extraction result JSON
- [ ] zero or more timeline events
- [ ] zero or more correlation keys
- [ ] normalized text for search
- [ ] parser warnings and errors

## Search and correlation plan

### Correlation goals

- [ ] Link artifacts that share request IDs, trace IDs, session IDs, user IDs, release SHAs, hostnames, endpoints, and timestamps.
- [ ] Surface likely related artifacts within the same investigation automatically.
- [ ] Let users pivot from a correlation key to all matching artifacts and events.

### Correlation implementation order

- [ ] Ship exact-match correlation on well-known keys first.
- [ ] Add normalized key registries and type-specific parsing rules.
- [ ] Add heuristic correlation only after exact-match correlation is trustworthy and explainable.
- [ ] Keep every derived relationship inspectable in the UI.

### Search implementation order

- [ ] Investigation list search
- [ ] In-investigation artifact and note search
- [ ] Full-text search over extracted text and note content
- [ ] Facets for artifact type, severity, environment, service, and time range
- [ ] Saved searches only after the core query model is stable

## API milestones

### Initial API surface

- [ ] session and auth routes
- [ ] workspace and membership routes
- [ ] investigation CRUD routes
- [ ] artifact upload and paste ingestion routes
- [ ] ingestion status routes
- [ ] timeline query routes
- [ ] search routes
- [ ] note CRUD routes
- [ ] bundle export and share routes

### API rules

- [ ] Validate every request and response boundary with Zod.
- [ ] Return explicit ingest and parse statuses.
- [ ] Keep route naming consistent and boring.
- [ ] Prefer pagination and filters from the start for list endpoints.
- [ ] Document stable error shapes early.

## UI milestones

### App shell and navigation

- [ ] Astro owns investigation routes, layout composition, and initial page delivery.
- [ ] Clear distinction between workspace, investigation list, and individual investigation views.
- [ ] Fast jump from investigation summary to artifacts, timeline, notes, and bundle builder.
- [ ] Minimal but polished dashboarding. No chart spam.

### Investigation experience

- [ ] Upload and paste entry points above the fold.
- [ ] Artifact list with type, status, ingestion state, and quick metadata.
- [ ] Unified timeline with filters and event detail drawer, keeping the page Astro-owned and adding Vue only if the filter and inspector state genuinely needs it.
- [ ] Correlation sidebar or panel for IDs and linked artifacts.
- [ ] Notes panel for hypotheses, conclusions, and evidence links.
- [ ] Bundle builder that makes scope obvious before sharing, implemented with Astro-first page ownership and optional Vue only if multi-select state becomes unwieldy.

### UX rules

- [ ] The product should reward evidence-based debugging, not overwhelm with raw volume.
- [ ] Every parsed view should link back to the original artifact.
- [ ] Empty states should teach the ingestion workflow.
- [ ] Error states should preserve user trust by showing what failed and what was still saved.

## Build phases

### Sequencing rules

- [ ] Execute phases in order unless a documented dependency allows safe overlap.
- [ ] Do not mark a phase complete until its acceptance criteria and verification steps are satisfied.
- [ ] If a later phase forces a redesign of an earlier phase, update both sections and explain the change in the relevant PR or commit.
- [ ] Keep the first launch path narrow. Defer optional sophistication when it threatens the critical path.

### Phase order summary

- [x] Phase 0: repo bootstrap and execution guardrails
- [x] Phase 1: application skeleton and local platform
- [ ] Phase 2: database foundation and migration runner
- [ ] Phase 3: authentication, workspace model, and investigation shell
- [ ] Phase 4: artifact ingestion MVP
- [ ] Phase 5: normalization pipeline and unified timeline
- [ ] Phase 6: search and exact-match correlation
- [ ] Phase 7: notes, bundle builder, and controlled sharing
- [ ] Phase 8: polish, resilience, and operational hardening
- [ ] Phase 9: deployment, staging, and launch readiness
- [ ] Phase 10: post-launch follow-up

## Phase 0: repo bootstrap and execution guardrails

### Objectives

- [x] Initialize the Bun workspace and root scripts.
- [x] Establish repo structure, code style, env handling, and verification entrypoints.
- [x] Add a README that explains the product and local development flow.
- [x] Add this `BUILD.md` and keep it current.

### Deliverables

- [x] Workspace `package.json` with `dev`, `build`, `test`, `typecheck`, and `verify` scripts
- [x] Bun workspace config
- [x] Biome config
- [x] root TypeScript config
- [x] base env example files
- [x] README with setup and architecture summary
- [x] compose file with postgres, object storage, and caddy placeholders or initial definitions

### Acceptance criteria

- [x] Fresh clone can install dependencies and run the base verification command.
- [x] Repo shape matches the intended app and package layout.
- [x] No hidden toolchain assumptions live only in chat.

### Verification

- [x] `bun install`
- [x] `bun run verify`

## Phase 1: application skeleton and local platform

### Objectives

- [x] Create `apps/web`, `apps/api`, and `packages/contracts`.
- [x] Wire Astro, Elysia, Zod, and shared TypeScript configs.
- [x] Keep Astro in charge of routes, layouts, and first-rendered app shells from the start.
- [x] Add Vue only if an initial investigation workflow already proves awkward without an interactive island.
- [x] Stand up Caddy routing for local integration.
- [x] Bring up PostgreSQL and object storage locally.

### Deliverables

- [x] web app with Astro-owned authenticated and unauthenticated layout placeholders
- [x] api service health endpoint and versioned API routing base
- [x] contracts package with initial DTOs and response envelopes
- [x] compose services for postgres and object storage
- [x] Caddyfile routing local web and api services
- [x] documented rule for when Vue is allowed into the web app

### Acceptance criteria

- [x] Web and API can run together through the local reverse proxy.
- [x] Contracts compile cleanly across packages.
- [x] Routes, layouts, and initial page delivery are Astro-owned.
- [x] Local developers can start the stack with one documented flow.

### Verification

- [x] `bun run dev`
- [x] `bun run typecheck`
- [x] `bun run astro:check`
- [x] smoke test through Caddy

Verified repo command for the Caddy smoke: `docker compose exec caddy sh -lc 'curl -fsS http://127.0.0.1:8080/ >/dev/null && curl -fsS http://127.0.0.1:8080/api/v1/health >/dev/null && echo CADDY_SMOKE_OK'`

## Phase 2: database foundation and migration runner

### Objectives

- [ ] Implement SQL migration runner and initial schema.
- [ ] Add database connectivity, query helpers, and transaction patterns.
- [ ] Establish seed strategy for local development.

### Deliverables

- [ ] migration runner script
- [ ] initial schema for users, workspaces, investigations, artifacts, notes, ingestion jobs, and basic sharing tables
- [ ] database access layer with explicit query functions
- [ ] local seed command

### Acceptance criteria

- [ ] Schema can be created from scratch in a clean database.
- [ ] Migrations are repeatable and deterministic.
- [ ] Basic relational constraints protect data integrity.

### Verification

- [ ] run migrations on empty database
- [ ] run seeds
- [ ] integration tests against local postgres

## Phase 3: authentication, workspace model, and investigation shell

### Objectives

- [ ] Implement auth, sessions, workspace membership, and investigation CRUD.
- [ ] Deliver the first usable app shell after login.

### Deliverables

- [ ] login and logout flows
- [ ] session middleware
- [ ] workspace switcher or personal workspace default
- [ ] Astro-rendered investigation list page
- [ ] create, edit, archive investigation flows

### Acceptance criteria

- [ ] Users can sign in, create an investigation, and revisit it later.
- [ ] Authorization rules prevent cross-workspace access.
- [ ] The investigation list and investigation shell remain Astro-owned pages.
- [ ] Audit trail exists for important security and sharing actions.

### Verification

- [ ] unit tests for auth and permission checks
- [ ] Playwright flow for sign in and create investigation
- [ ] negative authorization tests

## Phase 4: artifact ingestion MVP

### Objectives

- [ ] Support upload and paste flows for all day one artifact classes.
- [ ] Preserve raw artifacts in object storage.
- [ ] Track ingestion status and parser outcomes.

### Deliverables

- [ ] multipart upload endpoint
- [ ] paste submission endpoint for text artifacts
- [ ] artifact list UI with ingest status
- [ ] object storage adapter
- [ ] checksum and content-type detection
- [ ] parser stubs per artifact class

### Acceptance criteria

- [ ] Users can add each supported artifact type to an investigation.
- [ ] Raw content is retrievable for verification.
- [ ] Failed parsing does not destroy or hide the original artifact.

### Verification

- [ ] API tests for upload and paste routes
- [ ] storage integration tests
- [ ] browser flow for adding artifacts and seeing statuses

## Phase 5: normalization pipeline and unified timeline

### Objectives

- [ ] Turn ingested artifacts into extraction records, normalized timeline events, and correlation keys.
- [ ] Build the first serious investigation view.

### Deliverables

- [ ] parser implementations for stack traces, structured logs, HAR files, screenshots metadata, console output, and repro steps
- [ ] timeline event writer
- [ ] Astro investigation page with a timeline surface, using Vue only if filter and inspector state is clearly too rich for plain Astro plus HTML
- [ ] event detail panel with source links

### Acceptance criteria

- [ ] At least one realistic investigation containing mixed artifact types renders a coherent timeline.
- [ ] Users can trace each normalized event back to its source artifact.
- [ ] Page ownership still sits with Astro even if part of the timeline becomes a Vue island.
- [ ] Parser warnings are visible without breaking the main workflow.

### Verification

- [ ] fixture-based parser tests per artifact type
- [ ] integration tests for normalization persistence
- [ ] Playwright flow for timeline filtering and event inspection

## Phase 6: search and exact-match correlation

### Objectives

- [ ] Ship useful search and first-pass linkage across artifacts.
- [ ] Make request, trace, and session identifiers first-class navigation pivots.

### Deliverables

- [ ] extracted text indexing in PostgreSQL
- [ ] correlation key extraction registry
- [ ] search UI with filters and result grouping
- [ ] linked artifact panel for exact-match IDs

### Acceptance criteria

- [ ] Search can find cases by note text, filenames, stack trace content, request IDs, and log messages.
- [ ] Users can pivot from a correlation key to every matching artifact and event within an investigation.
- [ ] Query latency remains acceptable on realistic fixture data.

### Verification

- [ ] query plan review for key searches
- [ ] integration tests for correlation extraction and lookup
- [ ] benchmark on seeded fixture set

## Phase 7: notes, bundle builder, and controlled sharing

### Objectives

- [ ] Support human investigation notes and publishable debug bundles.
- [ ] Make asynchronous collaboration useful.

### Deliverables

- [ ] markdown-capable notes with timestamps and authorship
- [ ] note anchors to artifacts or timeline events
- [ ] bundle composition UI, keeping Astro in charge of the page and adding Vue only if the selection workflow clearly needs it
- [ ] bundle export manifest and renderable share page
- [ ] expiring and revocable share links

### Acceptance criteria

- [ ] A user can turn one investigation into a reviewable bundle with narrative context.
- [ ] Shared bundles expose only the selected scope.
- [ ] Shared bundle pages remain Astro-owned and do not require a full client app.
- [ ] Revoking a share link immediately blocks access.

### Verification

- [ ] bundle generation tests
- [ ] auth and access tests for shared versus private resources
- [ ] Playwright flow for create note, build bundle, open share link, revoke link

## Phase 8: polish, resilience, and operational hardening

### Objectives

- [ ] Raise the product from functional to launchable.
- [ ] Harden security, performance, and failure handling.

### Deliverables

- [ ] structured logging across web and api
- [ ] request IDs and traceability within the product itself
- [ ] rate limiting on auth and upload endpoints
- [ ] file size limits and content-type controls
- [ ] antivirus or malware scanning hook decision documented if public uploads are allowed
- [ ] background reprocessing path for parser upgrades
- [ ] bundle export cleanup and storage lifecycle rules
- [ ] accessibility and keyboard navigation pass
- [ ] visual polish pass with consistent tokens and states

### Acceptance criteria

- [ ] Common failure modes are handled explicitly and tested.
- [ ] Security-sensitive endpoints have rate limits and audit coverage.
- [ ] UI remains usable for long investigations and large artifact counts.

### Verification

- [ ] load and soak tests for upload and timeline queries
- [ ] security checklist review
- [ ] accessibility pass and regression checks
- [ ] manual usability review on realistic incident fixtures

## Phase 9: deployment, staging, and launch readiness

### Objectives

- [ ] Produce a boring, repeatable deployment shape.
- [ ] Validate the full system in staging with realistic data.

### Deliverables

- [ ] production Dockerfiles
- [ ] compose-based deployment shape or equivalent documented single-host deployment
- [ ] Caddy production config
- [ ] staging environment variables and secret handling guidance
- [ ] backup and restore procedures for postgres and object storage
- [ ] admin runbook for migrations, bundle storage, and share link revocation

### Acceptance criteria

- [ ] The app can be deployed to a fresh host from documented steps.
- [ ] Backups and restores are verified, not assumed.
- [ ] Staging validates the full critical path before launch.

### Verification

- [ ] deploy to staging from scratch
- [ ] run migrations in staging
- [ ] restore backup into isolated environment and verify data integrity
- [ ] full regression pass in staging

## Phase 10: post-launch follow-up

### Objectives

- [ ] Capture operational lessons and prioritize the next real improvements.
- [ ] Remove temporary build-phase scaffolding once the repo leaves greenfield mode.

### Deliverables

- [ ] launch retrospective notes
- [ ] prioritized backlog for v1.1
- [ ] documentation cleanup plan
- [ ] decision on when to retire `BUILD.md` and fold durable guidance into stable docs

### Acceptance criteria

- [ ] First production feedback is reflected in docs and backlog.
- [ ] Temporary greenfield instructions are not left to rot.
- [ ] Current-state docs exist for development and operations.

### Verification

- [ ] post-launch issue review
- [ ] docs review for stale build-specific content
- [ ] explicit decision on BUILD.md retirement timing

## Testing and verification gates

### Minimum repo gates

- [x] `bunx biome check .`
- [x] `bun run typecheck`
- [x] `bun run astro:check`
- [x] `bun test`
- [x] `bun run build:web`
- [x] `bun run build:api`
- [x] `bun run verify`

### Additional required gates by feature area

- [ ] Parser fixtures for every supported artifact type
- [ ] Database integration tests for migrations and core queries
- [ ] API tests for auth, uploads, notes, search, and sharing
- [ ] Playwright coverage for sign in, investigation creation, upload flows, timeline inspection, search, note authoring, and bundle sharing
- [ ] Regression fixtures that reflect real-world messy inputs, not only ideal examples

### Definition of done for each completed phase

- [ ] Code, tests, docs, and env examples are updated together.
- [ ] Phase acceptance criteria are met.
- [ ] Verification steps were actually run and noted in the relevant PR or commit.
- [ ] `BUILD.md` checkboxes and scope notes were updated.

## Launch hardening checklist

- [ ] Enforce secure cookies, CSRF protections where needed, and strict session handling.
- [ ] Enforce upload file size, content-type, and rate limits.
- [ ] Sanitize filenames and isolate object storage paths.
- [ ] Protect against path traversal, zip bombs, and malicious HAR or JSON payload behavior.
- [ ] Review bundle sharing defaults for least privilege.
- [ ] Add retention and deletion rules for artifacts and bundles.
- [ ] Add audit visibility for auth, share, revoke, and delete operations.
- [ ] Add observability for ingest failures, parser exceptions, slow search queries, and bundle generation errors.
- [ ] Verify backup, restore, and disaster recovery steps.
- [ ] Verify production logging avoids leaking secrets and sensitive payloads unintentionally.

## Risks and open questions

### Product risks

- [ ] Artifact variability may make normalization harder than the UI implies.
- [ ] Search quality can disappoint if extracted text and metadata are weak.
- [ ] Sharing can create privacy risk if scope boundaries are sloppy.
- [ ] Large HAR files and log dumps can stress ingestion, storage, and query paths early.

### Technical risks

- [ ] Parser complexity can sprawl if not constrained by contracts and fixtures.
- [ ] Timeline ordering can get messy when timestamps are absent or conflicting.
- [ ] PostgreSQL full text search may be enough for v1, but only if extraction quality is good.
- [ ] Object storage lifecycle and export cleanup must be designed early enough to avoid orphaned data.

### Questions to resolve during build

- [ ] Exact auth method for launch
- [ ] Whether public internet uploads are supported at launch or only authenticated users can upload
- [ ] Whether bundle exports are generated as static snapshots, live views, or both
- [ ] Whether screenshots need OCR before v1
- [ ] Whether background processing is inline at first or moved to a dedicated worker before launch
- [ ] What retention defaults apply to artifacts, bundles, and revoked shares

## Immediate next actions

- [x] Complete Phase 0 repo bootstrap.
- [x] Create the workspace structure for web, api, and contracts.
- [x] Stand up local postgres, object storage, and caddy in Compose.
- [ ] Commit initial migrations and contract scaffolding before feature work starts.

## Exit criteria for removing BUILD.md later

- [ ] The repo is no longer in active greenfield build mode.
- [ ] Current-state guidance has been moved into `README.md`, `CONTRIBUTING.md`, and `docs/operations.md` or equivalent.
- [ ] Remaining roadmap items live in issues or milestone tracking instead of this file.
- [ ] The team explicitly decides the build document has served its purpose.
