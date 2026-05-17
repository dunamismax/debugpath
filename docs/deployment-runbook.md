# Deployment Runbook

This runbook describes the intended local-to-production shape for the SSH
server, public site, PostgreSQL database, and worker. It is a checklist for a
small self-hosted MVP, not permission to change DNS, Cloudflare, Caddy, a
firewall, or production hosts without explicit approval.

## Services

- `debugpath-ssh`: public SSH entrypoint for player sessions.
- `debugpath-site`: public HTTP site behind Caddy or an equivalent reverse
  proxy.
- `debugpath-db`: PostgreSQL database for players, attempts, submissions,
  scores, replay events, unlocks, published case state, and authored drafts.
- `debugpath-worker`: background jobs for replay processing, import
  validation, leaderboard aggregation, or future notifications.

## Preflight

Before deployment:

- Confirm the git commit to deploy and record it in the release notes.
- Run `just gate` from a clean checkout.
- Run database migrations against a staging or local database first.
- Confirm production configuration is supplied through the host environment,
  not committed files.
- Confirm SSH bind address, host key path, rate limits, session limits, and
  audit log destination.
- Confirm site bind address and reverse-proxy health check path.
- Confirm PostgreSQL backup location and restore procedure.

## SSH Server

The SSH service should run as a locked-down service user with no shell access
needed by player sessions. Player commands remain simulated by the engine.

Required production controls before public launch:

- stable host keys with restricted filesystem permissions
- explicit listen address and port
- safe authentication path for anonymous or handle-based play
- per-IP and per-handle rate limits
- per-process and per-player session limits
- structured audit logs with redaction
- graceful disconnect and terminal resize handling

The current `debugpath-ssh` crate includes deterministic primitives for the
abuse-control pieces: per-peer rate windows, active session caps, command-size
rejection, and redacted audit event records. Wire those controls into the real
SSH accept loop before the service is exposed publicly.

Local development must not depend on `debugpath.dev` DNS.

## Site

The site should be served behind HTTPS and should remain useful without an
account. The homepage must keep `ssh debugpath.dev` visible as the primary
entrypoint and link to case catalog, leaderboard, recent solves, profiles,
replays, authoring docs, and case quality standards.

The site process should expose a health route that verifies the web process is
alive without requiring production secrets.

Current runtime knobs:

- `DEBUGPATH_SITE_ADDR`: bind address for the Axum listener. Use loopback
  behind Caddy, for example `127.0.0.1:4000`.
- `DEBUGPATH_CASES_DIR`: optional path to the Git-authored case fixture root.
  When set, the site validates the cases on startup and renders the catalog
  from those fixtures. Startup should fail if the checked-out cases are
  invalid.
- `DEBUGPATH_SSH_ENTRYPOINT`: displayed command for the primary product path,
  normally `ssh debugpath.dev`.
- `DEBUGPATH_PUBLIC_BASE_URL`: displayed canonical public site URL.

Routes expected by the reverse proxy and release smoke checks:

- `/healthz`: process liveness check, returns `ok`.
- `/readyz`: readiness check for the current in-process site, returns `ready`.
- `/status`: human-readable status page showing case, solve, replay, and data
  source counts.

Recommended Ubuntu service shape for the next deployment pass:

```ini
[Unit]
Description=debugpath public site
After=network-online.target
Wants=network-online.target

[Service]
User=debugpath
Group=debugpath
WorkingDirectory=/opt/debugpath/current
Environment=DEBUGPATH_SITE_ADDR=127.0.0.1:4000
Environment=DEBUGPATH_CASES_DIR=/opt/debugpath/current/cases
Environment=DEBUGPATH_SSH_ENTRYPOINT=ssh debugpath.dev
Environment=DEBUGPATH_PUBLIC_BASE_URL=https://debugpath.dev
ExecStart=/opt/debugpath/current/target/release/debugpath-site
Restart=on-failure
RestartSec=3
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/debugpath

[Install]
WantedBy=multi-user.target
```

Keep Caddy as the HTTPS edge and proxy only to the loopback site listener. Do
not point Caddy at a wildcard local address unless there is a deliberate reason
to expose the Rust process directly.

```caddyfile
debugpath.dev {
    encode zstd gzip
    reverse_proxy 127.0.0.1:4000
}
```

## PostgreSQL

Apply migrations in order from `crates/debugpath-db/migrations/`. The first
schema creates storage for players, published cases, attempts, diagnosis
submissions, scores, replay events, unlocks, and authored drafts.

Operational expectations:

- use a dedicated database user with least privilege
- keep migrations in git and review them like code
- test restore from the current backup before public launch
- do not copy production data into fixtures or logs

## Worker

The worker is optional until background jobs exist. When enabled, it should use
the same database schema and structured logging conventions as the site and SSH
server. Jobs must be idempotent where practical because retries are expected.

## Release Smoke Checks

After deployment:

- connect to the SSH service from a non-production shell account
- start one anonymous or handle-based session
- load the site homepage and featured case page
- render one replay
- verify migrations applied to the expected database
- verify logs redact connection metadata as designed
- verify backup job completion and restore instructions are current
