# Checkout Confirmation Runbook

The confirmation path reads recent pending orders after payment authorization.
Expected steady-state latency is below 120ms p95. Redis warnings can delay
recommendations but should not block order confirmation. If PostgreSQL CPU and
confirmation latency climb together, inspect the query plan before increasing
request timeouts.
