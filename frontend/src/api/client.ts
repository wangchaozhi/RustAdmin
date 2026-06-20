import type { TokenPair } from "./types";

const ACCESS_KEY = "harbor.access";
const REFRESH_KEY = "harbor.refresh";

export const tokenStore = {
  get access() { return localStorage.getItem(ACCESS_KEY); },
  get refresh() { return localStorage.getItem(REFRESH_KEY); },
  save(t: TokenPair) {
    localStorage.setItem(ACCESS_KEY, t.access_token);
    localStorage.setItem(REFRESH_KEY, t.refresh_token);
  },
  clear() {
    localStorage.removeItem(ACCESS_KEY);
    localStorage.removeItem(REFRESH_KEY);
  },
};

export class ApiError extends Error {
  status: number;
  constructor(status: number, message: string) {
    super(message);
    this.status = status;
  }
}

let refreshing: Promise<boolean> | null = null;

async function tryRefresh(): Promise<boolean> {
  if (!refreshing) {
    refreshing = (async () => {
      const refresh = tokenStore.refresh;
      if (!refresh) return false;
      const res = await fetch("/api/auth/refresh", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ refresh_token: refresh }),
      });
      if (!res.ok) {
        tokenStore.clear();
        return false;
      }
      tokenStore.save(await res.json());
      return true;
    })().finally(() => { refreshing = null; });
  }
  return refreshing;
}

export async function api<T>(
  path: string,
  options: { method?: string; body?: unknown; retry?: boolean } = {},
): Promise<T> {
  const { method = "GET", body, retry = true } = options;
  const headers: Record<string, string> = {};
  if (body !== undefined) headers["Content-Type"] = "application/json";
  if (tokenStore.access) headers["Authorization"] = `Bearer ${tokenStore.access}`;

  const res = await fetch(path, {
    method,
    headers,
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });

  if (res.status === 401 && retry && tokenStore.refresh) {
    const ok = await tryRefresh();
    if (ok) return api<T>(path, { method, body, retry: false });
    window.dispatchEvent(new Event("harbor:logout"));
    throw new ApiError(401, "登录已过期,请重新登录");
  }

  if (!res.ok) {
    let message = `请求失败 (${res.status})`;
    try {
      const data = await res.json();
      if (data?.error) message = data.error;
    } catch { /* 非 JSON 响应 */ }
    throw new ApiError(res.status, message);
  }

  return res.json() as Promise<T>;
}

/** 带鉴权地下载文件(如 CSV 导出),触发浏览器另存为 */
export async function download(path: string, filename: string): Promise<void> {
  const authHeaders = (): Record<string, string> =>
    tokenStore.access ? { Authorization: `Bearer ${tokenStore.access}` } : {};

  let res = await fetch(path, { headers: authHeaders() });
  if (res.status === 401 && tokenStore.refresh) {
    const ok = await tryRefresh();
    if (!ok) {
      window.dispatchEvent(new Event("harbor:logout"));
      throw new ApiError(401, "登录已过期,请重新登录");
    }
    res = await fetch(path, { headers: authHeaders() });
  }
  if (!res.ok) throw new ApiError(res.status, `导出失败 (${res.status})`);

  const blob = await res.blob();
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
}
