export type User = {
  id: string;
  username: string;
  admin: boolean;
  permissions: string[];
  csrf_token: string;
};
export type Provider = {
  id: string;
  kind: string;
  name: string;
  enabled: boolean;
  suspended: boolean;
  has_credentials: boolean;
};
export type RecordItem = {
  id: string;
  name: string;
  zone_name: string;
  zone_remote_id: string;
  provider_id: string;
  provider_name: string;
  provider_kind: string;
  record_type: string;
  managed_ip: string;
  manual_ip: string;
  active: boolean;
  interval_seconds: number;
  status: string;
  site: string;
  last_check: string | null;
  last_update: string | null;
  next_check: string;
  provider_suspended: boolean;
};
export type History = {
  id: string;
  fqdn: string;
  provider_name: string;
  previous_ip: string;
  new_ip: string;
  outcome: string;
  trigger: string;
  created_at: string;
  duration_ms: number;
  error_code: string | null;
};
let csrf = '';
export function setCsrf(value: string) {
  csrf = value;
}
export class ApiError extends Error {
  constructor(
    public code: string,
    public status: number
  ) {
    super(code);
  }
}
export async function apiResponse(
  path: string,
  method = 'GET',
  body?: unknown,
  extra?: Record<string, string>
): Promise<Response> {
  const response = await fetch('/api/v1' + path, {
    method,
    credentials: 'same-origin',
    headers: {
      ...(body === undefined ? {} : { 'Content-Type': 'application/json' }),
      ...(method === 'GET' ? {} : { 'X-CSRF-Token': csrf }),
      ...extra
    },
    body: body === undefined ? undefined : JSON.stringify(body)
  });
  if (!response.ok) {
    const data = await response
      .json()
      .catch(() => ({ code: 'request_failed' }));
    throw new ApiError(data.code, response.status);
  }
  return response;
}
export async function api<T = unknown>(
  path: string,
  method = 'GET',
  body?: unknown,
  extra?: Record<string, string>
): Promise<T> {
  const response = await apiResponse(path, method, body, extra);
  if (response.status === 204) return undefined as T;
  return response.json();
}
export function fqdn(record: RecordItem) {
  return record.name === '@'
    ? record.zone_name
    : `${record.name}.${record.zone_name}`;
}
