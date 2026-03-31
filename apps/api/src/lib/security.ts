import { createHash, randomUUID } from 'node:crypto';

export const sha256Hex = (value: string) => createHash('sha256').update(value).digest('hex');

export const createSessionToken = () => `${randomUUID()}${randomUUID()}`.replaceAll('-', '');

export const slugify = (value: string) => {
  const normalized = value
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');

  return normalized || 'investigation';
};
