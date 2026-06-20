import { createContext, useCallback, useContext, useEffect, useMemo, useState } from "react";
import { api, tokenStore } from "../api/client";
import type { TokenPair, User } from "../api/types";

interface AuthState {
  user: User | null;
  permissions: string[];
  loading: boolean;
  login: (username: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  refresh: () => Promise<void>;
  can: (perm: string) => boolean;
}

const AuthContext = createContext<AuthState | null>(null);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [permissions, setPermissions] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);

  // 启动时恢复会话
  useEffect(() => {
    if (!tokenStore.access && !tokenStore.refresh) {
      setLoading(false);
      return;
    }
    api<{ user: User; permissions: string[] }>("/api/auth/me")
      .then((d) => { setUser(d.user); setPermissions(d.permissions); })
      .catch(() => tokenStore.clear())
      .finally(() => setLoading(false));
  }, []);

  // 刷新失败时全局登出
  useEffect(() => {
    const onLogout = () => { setUser(null); setPermissions([]); };
    window.addEventListener("harbor:logout", onLogout);
    return () => window.removeEventListener("harbor:logout", onLogout);
  }, []);

  const login = useCallback(async (username: string, password: string) => {
    const data = await api<{ tokens: TokenPair; user: User }>("/api/auth/login", {
      method: "POST",
      body: { username, password },
    });
    tokenStore.save(data.tokens);
    setUser(data.user);
    const me = await api<{ user: User; permissions: string[] }>("/api/auth/me");
    setPermissions(me.permissions);
  }, []);

  const logout = useCallback(async () => {
    try { await api("/api/auth/logout", { method: "POST" }); } catch { /* 忽略 */ }
    tokenStore.clear();
    setUser(null);
    setPermissions([]);
  }, []);

  const refresh = useCallback(async () => {
    const me = await api<{ user: User; permissions: string[] }>("/api/auth/me");
    setUser(me.user);
    setPermissions(me.permissions);
  }, []);

  const can = useCallback((perm: string) => permissions.includes(perm), [permissions]);

  const value = useMemo(
    () => ({ user, permissions, loading, login, logout, refresh, can }),
    [user, permissions, loading, login, logout, refresh, can],
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error("useAuth 必须在 AuthProvider 内使用");
  return ctx;
}
