# BUILD.md

Active build plan for debugpath.dev.

`README.md` explains the product. `AGENTS.md` holds durable repo operating
rules. This file stays focused on current state, milestone scope, and
verification.

Last reviewed: 2026-05-17.

---

## Current Baseline

Observed on 2026-05-17:

- The repo contains the initial Rust workspace, foundation docs, three valid
  seed cases, content validation, deterministic engine scaffolding,
  route/render smoke tests, and a `just`-based local gate.
- The domain `debugpath.dev` is owned by Stephen and configured on
  Cloudflare.
- No production SSH server, full Ratatui app, PostgreSQL migrations, or full
  Axum/Leptos site has been implemented yet.
- The intended stack is Rust-first:
  - SSH entrypoint through a Rust SSH server.
  - Ratatui for the primary incident console.
  - Axum + Leptos for the public website.
  - PostgreSQL for attempts, scores, submissions, replay events, users,
    unlocks, and authored drafts.
  - Git-authored structured case content.
  - `xtask` for validation, seed data, release checks, and deploy helpers
    once those tasks become non-trivial.

debugpath.dev should combine the terminal-native product discipline of
FileFerry with the structured, reviewable content discipline of LangIndex.

---

## Product Definition

debugpath.dev is an SSH-native debugging game for developers who like real
systems. Players solve production-style incidents from a terminal:

```sh
ssh debugpath.dev
```

The first product surface is the SSH/Ratatui incident console. The public
Leptos site supports the game by showing leaderboards, solved cases, replay
viewers, authoring standards, player profiles, and an obvious "SSH in now"
path. The website is not a marketing shell.

The MVP should be a deterministic simulation, not real containers. Shell,
SQL, metrics, traces, logs, deploy diffs, runbooks, and packet snippets are
fake but realistic artifacts served by a strict case engine.

## Core Loop

1. Player connects over SSH.
2. They choose a case or accept a random incident.
3. The Ratatui interface opens with incident panes such as `Brief`,
   `Systems`, `Logs`, `Metrics`, `Shell`, `SQL`, `Trace`, and `Notes`.
4. They inspect artifacts and run constrained commands.
5. They submit a diagnosis:
   - root cause
   - evidence
   - affected component
   - proposed fix
   - blast radius
6. They apply one fix from a controlled option set.
7. The engine grades correctness, time, hint cost, evidence quality, root
   cause coverage, and operational damage caused by bad commands.

## Target Workspace Shape

Start with a Cargo workspace:

```text
crates/
  debugpath-ssh/      russh server, auth, sessions, terminal IO
  debugpath-tui/      Ratatui incident console and input model
  debugpath-engine/   case state machine, command simulation, scoring, replay
  debugpath-content/  typed case loader, parser, fixtures, validation
  debugpath-db/       PostgreSQL schema and sqlx queries
  debugpath-site/     Axum + Leptos public site
  debugpath-worker/   optional async replay/import/leaderboard jobs
xtask/                validation, seed cases, release checks, deploy helpers
cases/                Git-authored incident cases
docs/                 durable design, authoring, deployment, and ops docs
```

Prefer focused crates over a monolith once boundaries are real. Do not split
for ceremony before shared types and ownership are clear.

## Case Content Model

Cases live in Git first and should be reviewable like LangIndex content:

```text
cases/
  postgres-cpu-pinned/
    case.toml
    brief.md
    logs.ndjson
    metrics.toml
    schema.sql
    traces.json
    diffs/
    commands.toml
    scoring.toml
```

The loader must validate required metadata, artifact references, command
fixtures, scoring rules, diagnosis expectations, fix options, false trails,
and deterministic replay behavior before a case can ship.

## MVP Definition

MVP means the core game is usable end to end:

- [x] Rust workspace scaffolded with the target crate boundaries or a clearly
      documented smaller initial subset.
- [ ] `ssh debugpath.dev` can land a player in a Ratatui session.
- [ ] Local SSH development mode works without production DNS or Cloudflare
      changes.
- [ ] One Ratatui session runs per player connection.
- [x] Three polished deterministic cases ship with realistic artifacts and at
      least one plausible false trail each.
- [x] Structured case loader validates cases before runtime.
- [ ] Player can browse brief, logs, metrics, shell output, SQL output,
      traces, diffs, runbooks, notes, and hints where a case provides them.
- [x] Constrained shell and SQL commands are backed by case fixtures, not by
      host shell access.
- [x] Diagnosis submission captures root cause, evidence, affected component,
      fix, and blast radius.
- [x] Fix application is limited to authored options.
- [x] Scoring covers correctness, time, hint cost, evidence quality, symptom
      masking, and command damage.
- [ ] PostgreSQL stores users or anonymous handles, attempts, submissions,
      scores, replay events, unlocks, and published case state.
- [ ] Public Axum/Leptos site shows live or seeded leaderboard, recent solves,
      featured incident, player profiles, replay viewer, authoring docs, and
      case quality standards.
- [x] Replay events are captured and can be rendered on the site.
- [x] Local verification commands are documented and pass from a clean
      checkout.

## First Cases

Build fewer cases with more craft. The first five target cases are:

- [x] Slow Checkout: API latency jumps from 80ms to 4s after a deploy. Root
      cause is a missing database index exposed by a changed query shape.
- [x] Pinned Postgres: database CPU is maxed. Root cause is a dashboard query
      doing repeated full scans after a feature flag enabled extra joins.
- [x] Green CI, Bad Prod: deploy passes, production returns 502. Root cause is
      config or environment mismatch plus health check path drift.
- [x] Memory Tide: memory climbs under load. Root cause is unbounded request
      body buffering or leaked cache entries.
- [x] Corrupt Uploads: large archive uploads intermittently fail. Root cause
      is chunk reassembly ordering or partial-write retry behavior.

MVP requires three polished cases, not all five.

## Phase Plan

### Phase 1 - Foundation Scaffold

- [x] Create Rust workspace, `rust-toolchain.toml`, `justfile`, clippy/fmt/test
      gates, and initial crate layout.
- [x] Add route and crate smoke tests early enough that later scaffolding does
      not drift.
- [x] Add local development docs for SSH, site, PostgreSQL, and case
      validation.
- [x] Add placeholder case fixtures only if they are valid according to the
      first loader schema.

Exit criteria: the repo builds, formats, tests, and has a credible local loop.

### Phase 2 - Case Engine And Content

- [x] Define typed case schema and validation errors.
- [x] Implement artifact loading for brief, logs, metrics, SQL schema and
      rows, traces, diffs, runbooks, command fixtures, hints, and scoring.
- [x] Implement deterministic command simulation for shell and SQL.
- [x] Implement diagnosis and fix state machine.
- [x] Implement replay event model.
- [x] Build the first polished case and use it as a fixture for tests.

Exit criteria: one complete case can be loaded, explored through engine APIs,
diagnosed, fixed, scored, and replayed without a UI.

### Phase 3 - SSH And TUI

- [ ] Implement local SSH server mode with safe development auth.
- [ ] Map SSH terminal IO into the Ratatui app.
- [ ] Build the incident panes, command palette, notes, hints, diagnosis
      form, fix selection, and results view.
- [ ] Keep host filesystem and host shell unavailable to players.
- [ ] Capture replay events from meaningful player actions.
- [ ] Test terminal sizing, disconnect behavior, narrow layouts, and bad
      input paths.

Exit criteria: a local SSH session can play one complete case end to end.

### Phase 4 - Database And Site

- [x] Add PostgreSQL schema and migrations.
- [ ] Store attempts, submissions, scores, replay events, player handles, and
      published case metadata.
- [ ] Build Axum + Leptos public site with:
      - SSH command as the primary action.
      - leaderboard
      - recent solved cases
      - featured incident
      - player profiles
      - replay viewer
      - authoring docs
      - case quality standards
- [x] Add route-level tests and browser checks for the public site.

Exit criteria: the web surface reflects real or seeded game data and can
display a replay.

### Phase 5 - MVP Hardening

- [x] Ship three polished cases.
- [x] Add case authoring guide and review checklist.
- [x] Add operational runbook for deploying the SSH server, site, database,
      and worker.
- [ ] Add rate limiting, session limits, audit logs, and basic abuse controls.
- [x] Add backup and restore notes for PostgreSQL.
- [ ] Add release smoke checks for SSH, TUI, site, database migrations, and
      replay rendering.

Exit criteria: debugpath.dev is ready for a small public MVP.

## Deployment Notes

- `debugpath.dev` is already configured on Cloudflare, but do not change DNS,
  Cloudflare settings, Caddy config, or production hosts without explicit
  approval.
- The likely production shape is Caddy terminating HTTPS for the site and a
  separately exposed SSH service for the game.
- Keep local development independent of the production domain.
- Store cases in Git; store player history and published state in PostgreSQL.
- Do not require production secrets for local case validation or site tests.

## Verification

Docs-only work:

```sh
git diff --check
```

Expected normal Rust gate once the workspace exists:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo build --workspace
```

Expected project gate once `just` exists:

```sh
just fmt
just check
just test
just build
just validate-cases
```

Broaden verification for production or UI changes with SSH smoke tests,
database migration checks, browser screenshots, replay fixture tests, and
deployment health checks.
