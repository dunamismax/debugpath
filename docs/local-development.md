# Local Development

debugpath.dev is currently scaffolded as a Rust workspace with a deterministic
case loader and engine tests. Production DNS, Cloudflare, Caddy, and hosted
services are not required for local work.

## Prerequisites

- Rust toolchain from `rust-toolchain.toml`
- `just`
- PostgreSQL only when working on `debugpath-db` migrations or persistence

## Common Loop

```sh
just fmt
just check
just test
just build
just validate-cases
```

`just gate` runs the full local scaffold gate.

## Case Validation

Cases live under `cases/`. Validate all authored cases with:

```sh
just validate-cases
```

The validator checks required metadata, artifact references, command fixtures,
diagnosis evidence, hint and false-trail evidence references, root-cause and
symptom fixes, duplicate authored IDs, duplicate slugs, and basic timestamp
shape for logs and case metadata.

## SSH

`debugpath-ssh` starts a local development SSH server. By default it binds only
to loopback:

```sh
cargo run -p debugpath-ssh
ssh -p 2222 localhost
```

The bind address and seed case are configurable without production DNS,
Cloudflare, Caddy, or secrets:

```sh
DEBUGPATH_SSH_BIND=127.0.0.1:2223 DEBUGPATH_CASE_SLUG=slow-checkout cargo run -p debugpath-ssh
```

Development auth accepts anonymous, password, or public-key attempts after the
abuse controls accept the peer. Each SSH session receives a fresh in-memory
game state loaded through `debugpath-content` and `debugpath-engine`.

The terminal screen is rendered by `debugpath-tui` with Ratatui and sent over
the SSH channel. Player input is interpreted by the TUI/engine command model;
SSH `exec`, environment requests, subsystems, host shell access, and host
filesystem access are rejected. Current controls include per-peer connection
rate windows, active session limits, command-size checks, and structured audit
events with redacted peer metadata.

## Site

`debugpath-site` currently exposes renderable route HTML helpers with smoke
tests. Phase 4 will wire the public Axum and Leptos surface to real or seeded
game data.

## PostgreSQL

`debugpath-db` owns migrations under `crates/debugpath-db/migrations/`. The
initial schema covers players, published cases, attempts, diagnosis
submissions, scores, replay events, unlocks, and authored drafts.

Run migration work against a local database URL provided by the developer
environment. Do not require production secrets for local checks.

Backup and restore notes live in
[`postgres-backup-restore.md`](postgres-backup-restore.md).
