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

`debugpath-ssh` is scaffolded but does not yet start a local SSH server. Phase 3
will add safe development auth and local bind settings that do not depend on
`debugpath.dev` DNS.

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
