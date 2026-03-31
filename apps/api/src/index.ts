import {
  apiBasePath,
  apiVersion,
  createOkEnvelope,
  healthPayloadSchema,
} from '@debugpath/contracts';
import { Elysia } from 'elysia';

const startedAt = Date.now();

const buildHealthResponse = () =>
  createOkEnvelope(healthPayloadSchema, {
    service: 'api',
    version: apiVersion,
    environment: Bun.env.APP_ENV ?? 'development',
    uptimeSeconds: Math.floor((Date.now() - startedAt) / 1000),
    timestamp: new Date().toISOString(),
  });

export const app = new Elysia()
  .get('/health', () => buildHealthResponse())
  .group(apiBasePath, (group) => group.get('/health', () => buildHealthResponse()));

const port = Number(Bun.env.PORT ?? 3000);
const hostname = Bun.env.HOST ?? '0.0.0.0';

if (import.meta.main) {
  app.listen({
    port,
    hostname,
  });

  console.log(`debugpath api listening on http://${hostname}:${port}${apiBasePath}/health`);
}
