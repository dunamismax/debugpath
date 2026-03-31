import type { APIContext } from 'astro';

const defaultInternalApiOrigin = 'http://localhost:3000';

export const getInternalApiOrigin = () =>
  import.meta.env.INTERNAL_API_ORIGIN || defaultInternalApiOrigin;

export const fetchFromDebugPathApi = (Astro: APIContext, path: string, init?: RequestInit) => {
  const headers = new Headers(init?.headers ?? {});
  const cookie = Astro.request.headers.get('cookie');

  if (cookie) {
    headers.set('cookie', cookie);
  }

  headers.set('accept', 'application/json');

  return fetch(new URL(path, getInternalApiOrigin()), {
    ...init,
    headers,
  });
};
