import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api, download } from "../api/client";
import type { Page, Role, User } from "../api/types";
import { Empty, Modal, Pagination, RoleBadge, StatusBadge, formatTime } from "../components/ui";
import { useAuth } from "../store/auth";

interface FormState {
  id?: string;
  username: string;
  email: string;
  display_name: string;
  role: string;
  status: string;
  password: string;
}

const EMPTY_FORM: FormState = {
  username: "", email: "", display_name: "", role: "viewer", status: "active", password: "",
};

export default function Users() {
  const { can, user: me } = useAuth();
  const qc = useQueryClient();
  const [page, setPage] = useState(1);
  const [q, setQ] = useState("");
  const [search, setSearch] = useState("");
  const [form, setForm] = useState<FormState | null>(null);
  const [confirmDelete, setConfirmDelete] = useState<User | null>(null);
  const [error, setError] = useState("");

  const pageSize = 10;
  const { data, isLoading } = useQuery({
    queryKey: ["users", page, search],
    queryFn: () => api<Page<User>>(`/api/users?page=${page}&page_size=${pageSize}&q=${encodeURIComponent(search)}`),
  });
  const { data: roles } = useQuery({
    queryKey: ["roles"],
    queryFn: () => api<Role[]>("/api/roles"),
  });

  const invalidate = () => {
    qc.invalidateQueries({ queryKey: ["users"] });
    qc.invalidateQueries({ queryKey: ["dashboard"] });
  };

  const saveMut = useMutation({
    mutationFn: async (f: FormState) => {
      if (f.id) {
        const body: Record<string, string> = {
          email: f.email, display_name: f.display_name, role: f.role, status: f.status,
        };
        if (f.password) body.password = f.password;
        return api(`/api/users/${f.id}`, { method: "PUT", body });
      }
      return api("/api/users", {
        method: "POST",
        body: { username: f.username, email: f.email, password: f.password, display_name: f.display_name, role: f.role },
      });
    },
    onSuccess: () => { setForm(null); setError(""); invalidate(); },
    onError: (e) => setError(e instanceof Error ? e.message : "保存失败"),
  });

  const deleteMut = useMutation({
    mutationFn: (id: string) => api(`/api/users/${id}`, { method: "DELETE" }),
    onSuccess: () => { setConfirmDelete(null); invalidate(); },
    onError: (e) => setError(e instanceof Error ? e.message : "删除失败"),
  });

  const openEdit = (u: User) => {
    setError("");
    setForm({
      id: u.id, username: u.username, email: u.email,
      display_name: u.display_name, role: u.role, status: u.status, password: "",
    });
  };

  const writable = can("users.write");

  return (
    <div className="page">
      <div className="page-head">
        <h1 className="page-title">用户管理</h1>
        {writable && (
          <button className="primary-btn" onClick={() => { setError(""); setForm({ ...EMPTY_FORM }); }}>
            新建用户
          </button>
        )}
      </div>

      <div className="toolbar">
        <input
          className="search"
          placeholder="搜索用户名 / 邮箱 / 姓名"
          value={q}
          onChange={(e) => setQ(e.target.value)}
          onKeyDown={(e) => { if (e.key === "Enter") { setSearch(q); setPage(1); } }}
        />
        <button onClick={() => { setSearch(q); setPage(1); }}>搜索</button>
        <button onClick={() => download(`/api/export/users?q=${encodeURIComponent(search)}`, "users.csv")}>
          导出 CSV
        </button>
      </div>

      {isLoading ? (
        <Empty text="加载中…" />
      ) : !data || data.items.length === 0 ? (
        <Empty text={search ? "没有匹配的用户,换个关键词试试" : "还没有用户,点击「新建用户」开始"} />
      ) : (
        <>
          {/* 桌面表格 */}
          <div className="card table-card">
            <table className="table">
              <thead>
                <tr>
                  <th>用户</th><th>邮箱</th><th>角色</th><th>状态</th><th>最近登录</th>
                  {writable && <th className="t-right">操作</th>}
                </tr>
              </thead>
              <tbody>
                {data.items.map((u) => (
                  <tr key={u.id}>
                    <td>
                      <strong>{u.display_name || u.username}</strong>
                      <div className="muted mono">@{u.username}</div>
                    </td>
                    <td>{u.email}</td>
                    <td><RoleBadge role={u.role} /></td>
                    <td><StatusBadge status={u.status} /></td>
                    <td className="mono">{formatTime(u.last_login_at)}</td>
                    {writable && (
                      <td className="t-right">
                        <button className="link-btn" onClick={() => openEdit(u)}>编辑</button>
                        {u.id !== me?.id && (
                          <button className="link-btn danger" onClick={() => { setError(""); setConfirmDelete(u); }}>删除</button>
                        )}
                      </td>
                    )}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          {/* 移动端卡片 */}
          <div className="card-list">
            {data.items.map((u) => (
              <div className="card user-card" key={u.id}>
                <div className="user-card-head">
                  <strong>{u.display_name || u.username}</strong>
                  <StatusBadge status={u.status} />
                </div>
                <div className="muted mono">@{u.username} · {u.email}</div>
                <div className="user-card-meta">
                  <RoleBadge role={u.role} />
                  <span className="muted">最近登录 {formatTime(u.last_login_at)}</span>
                </div>
                {writable && (
                  <div className="user-card-actions">
                    <button onClick={() => openEdit(u)}>编辑</button>
                    {u.id !== me?.id && (
                      <button className="danger" onClick={() => { setError(""); setConfirmDelete(u); }}>删除</button>
                    )}
                  </div>
                )}
              </div>
            ))}
          </div>

          <Pagination page={data.page} pageSize={data.page_size} total={data.total} onPage={setPage} />
        </>
      )}

      {form && (
        <Modal title={form.id ? "编辑用户" : "新建用户"} onClose={() => setForm(null)}>
          <div className="form">
            {!form.id && (
              <label className="field">
                <span>用户名</span>
                <input value={form.username} onChange={(e) => setForm({ ...form, username: e.target.value })} />
              </label>
            )}
            <label className="field">
              <span>邮箱</span>
              <input value={form.email} onChange={(e) => setForm({ ...form, email: e.target.value })} />
            </label>
            <label className="field">
              <span>姓名</span>
              <input value={form.display_name} onChange={(e) => setForm({ ...form, display_name: e.target.value })} />
            </label>
            <div className="field-row">
              <label className="field">
                <span>角色</span>
                <select value={form.role} disabled={me?.role !== "admin"}
                  onChange={(e) => setForm({ ...form, role: e.target.value })}>
                  {(roles ?? []).map((r) => <option key={r.name} value={r.name}>{r.name}</option>)}
                </select>
              </label>
              {form.id && (
                <label className="field">
                  <span>状态</span>
                  <select value={form.status} onChange={(e) => setForm({ ...form, status: e.target.value })}>
                    <option value="active">启用</option>
                    <option value="disabled">禁用</option>
                  </select>
                </label>
              )}
            </div>
            <label className="field">
              <span>{form.id ? "新密码(留空则不变)" : "密码(至少 8 位)"}</span>
              <input type="password" value={form.password}
                onChange={(e) => setForm({ ...form, password: e.target.value })} />
            </label>
            {error && <p className="form-error" role="alert">{error}</p>}
            <div className="form-actions">
              <button className="ghost-btn" onClick={() => setForm(null)}>取消</button>
              <button className="primary-btn" disabled={saveMut.isPending} onClick={() => saveMut.mutate(form)}>
                {saveMut.isPending ? "保存中…" : "保存"}
              </button>
            </div>
          </div>
        </Modal>
      )}

      {confirmDelete && (
        <Modal title="删除用户" onClose={() => setConfirmDelete(null)}>
          <div className="form">
            <p>确定删除用户 <strong>@{confirmDelete.username}</strong> 吗?该操作不可撤销。</p>
            {error && <p className="form-error" role="alert">{error}</p>}
            <div className="form-actions">
              <button className="ghost-btn" onClick={() => setConfirmDelete(null)}>取消</button>
              <button className="danger-btn" disabled={deleteMut.isPending}
                onClick={() => deleteMut.mutate(confirmDelete.id)}>
                {deleteMut.isPending ? "删除中…" : "确认删除"}
              </button>
            </div>
          </div>
        </Modal>
      )}
    </div>
  );
}
