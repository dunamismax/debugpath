# PostgreSQL Backup And Restore

PostgreSQL stores player history and published state. Case definitions remain
in git, so backups focus on mutable product data: players, attempts,
submissions, scores, replay events, unlocks, published case metadata, and
authored drafts.

## Backup

Use `pg_dump` in custom format for routine backups:

```sh
pg_dump --format=custom --file=debugpath-$(date -u +%Y%m%dT%H%M%SZ).dump "$DATABASE_URL"
```

Store backups outside the application host when possible. Restrict read access
because replay and diagnosis fields may contain free-form player text.

Minimum backup checks:

- the command exits successfully
- the output file is non-empty
- the file is copied to the expected backup destination
- retention removes old backups only after a newer backup is verified

## Restore Drill

Run restore drills against a local or staging database, never directly into
production:

```sh
createdb debugpath_restore_check
pg_restore --dbname=debugpath_restore_check --clean --if-exists <backup-file>
```

After restore, verify:

- core tables exist
- row counts are plausible for `players`, `attempts`, `scores`, and
  `replay_events`
- recent published cases are present
- at least one replay can be rendered by the site or route helper

## Production Restore

Before restoring production:

- stop writers or put the service into maintenance mode
- take a fresh backup of the current production database
- confirm the target backup timestamp and source
- restore into a temporary database first when time permits
- switch application configuration only after verification passes

Do not overwrite production data during an incident without an explicit
operator decision.

## Schema Migrations

Migrations live in `crates/debugpath-db/migrations/` and are exposed by the
`debugpath-db` crate for tests and future migration runners. Apply migrations
before starting application code that depends on new tables or columns.
