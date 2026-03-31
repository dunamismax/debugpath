import { expect, test } from 'bun:test';

import {
  apiBasePath,
  apiVersion,
  createOkEnvelope,
  healthPayloadSchema,
  healthResponseSchema,
} from '../src/index';

test('health envelope matches the exported contract', () => {
  const response = createOkEnvelope(healthPayloadSchema, {
    service: 'api',
    version: apiVersion,
    environment: 'test',
    uptimeSeconds: 3,
    timestamp: new Date().toISOString(),
  });

  expect(response.data.version).toBe(apiVersion);
  expect(response.meta.generatedAt).toBeString();
  expect(healthResponseSchema.parse(response).ok).toBeTrue();
  expect(apiBasePath).toBe('/api/v1');
});
