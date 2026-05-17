# debugpath.dev

debugpath.dev is an SSH-native debugging game for developers who like real
systems.

```sh
ssh debugpath.dev
```

The player lands in a terminal incident console and works through a realistic
production failure: logs, metrics, traces, SQL, deploy diffs, config files,
shell output, packet snippets, customer reports, runbooks, and an incident
timeline. The goal is not to guess the answer. The goal is to investigate,
form a hypothesis, find evidence, test it, and fix the root cause.

The project is in early implementation. The Rust workspace, deterministic
case fixtures, content loader, engine scaffold, local SSH mode, PostgreSQL
schema, and seeded Axum + Leptos public site are in place. The active build
plan lives in [`BUILD.md`](BUILD.md), durable repo rules live in
[`AGENTS.md`](AGENTS.md), and authoring, deployment, and backup notes live
under [`docs/`](docs/).

## Positioning

debugpath.dev is part game, part incident lab, and part portfolio piece for
serious Rust systems:

> Solve production incidents from the terminal. Read the logs, query the
> database, inspect traces, chase false leads, and prove you can fix the root
> cause under pressure.

The first version is a deterministic simulation, not real containers. Fake
shell, fake SQL, fake metrics, and fake logs are backed by structured case data
and a strict engine so the experience is fair, replayable, testable, and
self-hostable.

## Core Game Loop

1. Connect over SSH.
2. Choose a case or accept a random incident.
3. Investigate in a Ratatui console with panes like `Brief`, `Systems`,
   `Logs`, `Metrics`, `Shell`, `SQL`, `Trace`, and `Notes`.
4. Run constrained commands against case fixtures.
5. Submit a diagnosis:
   - root cause
   - evidence
   - affected component
   - proposed fix
   - blast radius
6. Apply one of the available fixes.
7. Receive a score based on correctness, time, hint cost, evidence quality,
   operational damage, and whether the fix solved the root cause or masked a
   symptom.

## What Makes A Case

Cases should feel like production incidents, not puzzle riddles. A strong case
has:

- logs with noise, timestamps, correlation IDs, and misleading warnings
- metrics panels for latency, CPU, memory, queue depth, locks, cache hit rate,
  and error rates
- SQL schema and fake-but-realistic rows
- trace spans with coherent timing
- deploy diffs and config files
- shell command output
- packet or protocol snippets when useful
- customer reports
- runbooks
- incident timeline
- at least one plausible false trail

Cases live in Git first:

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

The case model should be reviewable, versioned, validated, and eventually
community-contributable.

## Initial Cases

The first target set:

- **Slow Checkout**: API latency jumps from 80ms to 4s after a deploy. Root
  cause is a missing database index exposed by a changed query shape.
- **Pinned Postgres**: CPU is maxed. Root cause is a dashboard query doing
  repeated full scans after a feature flag enabled extra joins.
- **Green CI, Bad Prod**: deploy passes, production returns 502. Root cause is
  config or environment mismatch plus health check path drift.
- **Memory Tide**: memory climbs forever under load. Root cause is unbounded
  request body buffering or leaked cache entries.
- **Corrupt Uploads**: large archive uploads sometimes fail. Root cause is
  chunk reassembly ordering or partial-write retry behavior.

The MVP needs three polished cases, not a large catalog.

## Architecture

The intended implementation is a Rust workspace:

```text
crates/
  debugpath-ssh/      SSH server, auth, sessions, terminal IO
  debugpath-tui/      Ratatui incident console
  debugpath-engine/   case state machine, commands, scoring, replay events
  debugpath-content/  typed case loader and validator
  debugpath-db/       PostgreSQL schema and sqlx queries
  debugpath-site/     Axum + Leptos public site
  debugpath-worker/   optional replay, leaderboard, and import jobs
xtask/                validation, seed cases, release checks, deploy helpers
cases/                Git-authored incident cases
```

PostgreSQL stores users or anonymous handles, attempts, scores, submitted
diagnoses, replay events, unlocks, published case state, and authored puzzle
drafts. Case definitions remain Git-authored so they can be reviewed and
tested like code.

## Public Site

The website should support the SSH product rather than distract from it. It
should show:

- SSH entrypoint as the main action
- live leaderboard
- recent solved cases
- featured incident of the week
- player profiles
- replay viewer
- authoring docs
- case quality standards

The replay viewer is a core feature: it should make engineering judgment
inspectable by showing how a developer moved through the incident.

## MVP Scope

The first complete version should include:

- SSH login
- one Ratatui session per player
- three polished deterministic cases
- structured case loader
- command and artifact browsing
- diagnosis submission
- fix application
- scoring
- PostgreSQL-backed attempts and leaderboard
- public Axum/Leptos site
- replay event capture and replay viewer

Real containers and advanced labs can come later after the deterministic
simulation is excellent.

## Development

The repository is a Rust workspace. The expected local loop is:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --all-features
cargo test --workspace --all-features
cargo build --workspace
```

The project loop is:

```sh
just fmt
just check
just test
just build
just validate-cases
```

## License

MIT. See [`LICENSE`](LICENSE).
