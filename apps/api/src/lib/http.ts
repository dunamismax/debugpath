import { createErrorEnvelope } from '@debugpath/contracts';

const defaultHeaders = {
  'content-type': 'application/json; charset=utf-8',
  'cache-control': 'no-store',
};

export const jsonResponse = (payload: unknown, status = 200, headers?: HeadersInit) =>
  new Response(JSON.stringify(payload), {
    status,
    headers: {
      ...defaultHeaders,
      ...(headers ?? {}),
    },
  });

export const errorResponse = (
  status: number,
  code: string,
  message: string,
  headers?: HeadersInit
) => jsonResponse(createErrorEnvelope(code, message), status, headers);

export const redirectResponse = (location: string, headers?: HeadersInit, status = 302) =>
  new Response(null, {
    status,
    headers: {
      location,
      ...(headers ?? {}),
    },
  });

export const parseCookies = (headerValue: string | null) => {
  const cookies = new Map<string, string>();

  if (!headerValue) {
    return cookies;
  }

  for (const pair of headerValue.split(';')) {
    const separatorIndex = pair.indexOf('=');
    if (separatorIndex < 0) {
      continue;
    }

    const key = pair.slice(0, separatorIndex).trim();
    const value = pair.slice(separatorIndex + 1).trim();
    if (!key) {
      continue;
    }

    cookies.set(key, decodeURIComponent(value));
  }

  return cookies;
};

export const serializeCookie = ({
  name,
  value,
  maxAge,
  httpOnly = true,
  path = '/',
  sameSite = 'Lax',
  secure = false,
}: {
  name: string;
  value: string;
  maxAge?: number;
  httpOnly?: boolean;
  path?: string;
  sameSite?: 'Lax' | 'Strict' | 'None';
  secure?: boolean;
}) => {
  const parts = [`${name}=${encodeURIComponent(value)}`, `Path=${path}`, `SameSite=${sameSite}`];

  if (typeof maxAge === 'number') {
    parts.push(`Max-Age=${Math.max(0, Math.floor(maxAge))}`);
  }

  if (httpOnly) {
    parts.push('HttpOnly');
  }

  if (secure) {
    parts.push('Secure');
  }

  return parts.join('; ');
};

export const safeRedirectPath = (value: string | null, fallback: string) => {
  if (!value) {
    return fallback;
  }

  if (!value.startsWith('/') || value.startsWith('//')) {
    return fallback;
  }

  return value;
};
