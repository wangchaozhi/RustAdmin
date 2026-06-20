import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../api/client";
import type { Session } from "../api/types";
import { Empty, RoleBadge, formatTime } from "../components/ui";
import { useAuth } from "../store/auth";

function ProfileCard() {
  const { user, refresh } = useAuth();
  const [displayName, setDisplayName] = useState(user?.display_name ?? "");
  const [email, setEmail] = useState(user?.email ?? "");
  const [msg, setMsg] = useState("");
  const [error, setError] = useState("");

  const mut = useMutation({
    mutationFn: () => api("/api/auth/profile", { method: "PUT", body: { display_name: displayName, email } }),
    onSuccess: async () => { setError(""); setMsg("已保存"); await refresh(); },
    onError: (e) => { setMsg(""); setError(e instanceof Error ? e.message : "保存失败"); },
  });

  return (
    <section className="card profile-card">
      <h2 className="card-title">个人资料</h2>
      <div className="form">
        <label className="field">
          <span>用户名</span>
          <input value={user?.username ?? ""} disabled />
        </label>
        <label className="field">
          <span>姓名</span>
          <input value={displayName} onChange={(e) => { setDisplayName(e.target.value); setMsg(""); }} />
        </label>
        <label className="field">
          <span>邮箱</span>
          <input value={email} onChange={(e) => { setEmail(e.target.value); setMsg(""); }} />
        </label>
        <div className="field-inline">
          <span className="muted">当前角色</span>
          <RoleBadge role={user?.role ?? "viewer"} />
        </div>
        {error && <p className="form-error" role="alert">{error}</p>}
        {msg && <p className="form-ok" role="status">{msg}</p>}
        <div className="form-actions">
          <button className="primary-btn" disabled={mut.isPending} onClick={() => mut.mutate()}>
            {mut.isPending ? "保存中…" : "保存资料"}
          </button>
        </div>
      </div>
    </section>
  );
}

function PasswordCard() {
  const { logout } = useAuth();
  const navigate = useNavigate();
  const [current, setCurrent] = useState("");
  const [next, setNext] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState("");

  const mut = useMutation({
    mutationFn: () => api("/api/auth/change-password", {
      method: "POST",
      body: { current_password: current, new_password: next },
    }),
    onSuccess: async () => {
      await logout();
      navigate("/login", { replace: true });
    },
    onError: (e) => setError(e instanceof Error ? e.message : "修改失败"),
  });

  const submit = () => {
    setError("");
    if (next.length < 8) { setError("新密码至少 8 位"); return; }
    if (next !== confirm) { setError("两次输入的新密码不一致"); return; }
    mut.mutate();
  };

  return (
    <section className="card profile-card">
      <h2 className="card-title">修改密码</h2>
      <div className="form">
        <label className="field">
          <span>当前密码</span>
          <input type="password" autoComplete="current-password" value={current}
            onChange={(e) => setCurrent(e.target.value)} />
        </label>
        <label className="field">
          <span>新密码(至少 8 位)</span>
          <input type="password" autoComplete="new-password" value={next}
            onChange={(e) => setNext(e.target.value)} />
        </label>
        <label className="field">
          <span>确认新密码</span>
          <input type="password" autoComplete="new-password" value={confirm}
            onChange={(e) => setConfirm(e.target.value)} />
        </label>
        {error && <p className="form-error" role="alert">{error}</p>}
        <p className="muted">修改成功后将退出所有设备,需重新登录。</p>
        <div className="form-actions">
          <button className="primary-btn" disabled={mut.isPending} onClick={submit}>
            {mut.isPending ? "提交中…" : "修改密码"}
          </button>
        </div>
      </div>
    </section>
  );
}

function SessionsCard() {
  const qc = useQueryClient();
  const { data, isLoading } = useQuery({
    queryKey: ["sessions"],
    queryFn: () => api<{ items: Session[] }>("/api/auth/sessions"),
  });

  const revokeMut = useMutation({
    mutationFn: (id: string) => api(`/api/auth/sessions/${id}`, { method: "DELETE" }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["sessions"] }),
  });

  const sessions = data?.items ?? [];

  return (
    <section className="card profile-card profile-sessions">
      <h2 className="card-title">登录会话</h2>
      {isLoading ? (
        <Empty text="加载中…" />
      ) : sessions.length === 0 ? (
        <Empty text="没有活跃会话" />
      ) : (
        <ul className="session-list">
          {sessions.map((s) => (
            <li className="session-item" key={s.id}>
              <div className="session-info">
                <div className="session-ua">
                  {s.user_agent || "未知设备"}
                  {s.current && <span className="tag-current">当前设备</span>}
                </div>
                <div className="muted mono">
                  {s.ip || "—"} · 登录于 {formatTime(s.created_at)} · 过期 {formatTime(s.expires_at)}
                </div>
              </div>
              {!s.current && (
                <button className="link-btn danger" disabled={revokeMut.isPending}
                  onClick={() => revokeMut.mutate(s.id)}>
                  注销
                </button>
              )}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

export default function Profile() {
  return (
    <div className="page">
      <h1 className="page-title">个人中心</h1>
      <div className="profile-grid">
        <ProfileCard />
        <PasswordCard />
      </div>
      <SessionsCard />
    </div>
  );
}
