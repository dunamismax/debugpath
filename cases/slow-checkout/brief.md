# Slow Checkout

Checkout latency jumped from a normal 80ms p95 to about 4s within minutes of
the `checkout-api` deploy at 14:05 UTC. The customer-visible symptom is delayed
order confirmation after payment authorization. Payments are not double-charged,
but customers are refreshing and opening support tickets because confirmation
pages sometimes time out.

The on-call handoff says Redis logged a handful of reconnect warnings, and a
dashboard shows elevated PostgreSQL CPU. Determine the root cause, cite the
evidence, and choose a fix that removes the bottleneck rather than masking the
timeout.
