# Dashboard Incident Runbook

The analytics dashboard uses the read-only database pool. If dashboard latency
and shared database CPU rise together, first compare recent feature flags with
`pg_stat_statements` before restarting database services. Autovacuum can add
noise, but it should not keep CPU pinned after the worker exits.
