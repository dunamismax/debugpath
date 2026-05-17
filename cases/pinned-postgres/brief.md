# Pinned Postgres

PostgreSQL CPU has been at 99% since 15:20 UTC. Checkout reads are slower, but
the first complaints came from internal support staff opening the analytics
dashboard. The handoff mentions an autovacuum log line and a new dashboard flag
that was enabled for support managers shortly before the spike.

Find the workload pinning the database, separate it from routine maintenance
noise, and choose the fix that removes the repeated scan rather than simply
adding capacity.
