# Deploy Health Runbook

When CI passes but production edge health fails, compare the smoke-test path,
the service routes in the deploy diff, and the production router health path.
Do not disable health checks unless a separate safety mechanism is active.
