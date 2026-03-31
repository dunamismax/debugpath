import { expect, test } from 'bun:test';

import { healthResponseSchema } from '@debugpath/contracts';

import { app } from '../src/index';

test('api v1 health route returns the shared contract', async () => {
  const response = await app.handle(new Request('http://debugpath.local/api/v1/health'));
  const payload = await response.json();

  expect(response.status).toBe(200);
  expect(healthResponseSchema.parse(payload).data.service).toBe('api');
});

test('root health route stays available for direct service checks', async () => {
  const response = await app.handle(new Request('http://debugpath.local/health'));

  expect(response.status).toBe(200);
});
