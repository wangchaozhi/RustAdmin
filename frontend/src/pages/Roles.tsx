import { useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../api/client";
import type { Permission, Role } from "../api/types";
import { Empty, Modal, RoleBadge } from "../components/ui";
import { useAuth } from "../store/auth";

const BUILTIN = ["admin", "editor", "viewer"];

interface FormState {
  name: string;
  description: string;
  permissions: string[];
  isNew: boolean;
}

export default function Roles() {
  const { can } = useAuth();
  const qc = useQueryClient();
  const writable = can("roles.write");

  const { data: roles, isLoading } = useQuery({
    queryKey: ["roles"],
    queryFn: () => api<Role[]>("/api/roles"),
  });
  const { data: perms } = useQuery({
    queryKey: ["permissions"],
    queryFn: () => api<Permission[]>("/api/permissions"),
    enabled: writable,
  });

  const [form, setForm] = useState<FormState | null>(null);
  const [confirmDelete, setConfirmDelete] = useState<Role | null>(null);
  const [error, setError] = useState("");

  const grouped = useMemo(() => {
    const map = new Map<string, Permission[]>();
    for (const p of perms ?? []) {
      const arr = map.get(p.group) ?? [];
      arr.push(p);
      map.set(p.group, arr);
    }
    return Array.from(map.entries());
  }, [perms]);

  const invalidate = () => qc.invalidateQueries({ queryKey: ["roles"] });

  const saveMut = useMutation({
    mutationFn: async (f: FormState) => {
      if (f.isNew) {
        return api("/api/roles", {
          method: "POST",
          body: { name: f.name.trim(), description: f.description, permissions: f.permissions },
        });
      }
      return api(`/api/roles/${encodeURIComponent(f.name)}`, {
        method: "PUT",
        body: { description: f.description, permissions: f.permissions },
      });
    },
    onSuccess: () => { setForm(null); setError(""); invalidate(); },
    onError: (e) => setError(e instanceof Error ? e.message : "保存失败"),
  });

  const deleteMut = useMutation({
    mutationFn: (name: string) => api(`/api/roles/${encodeURIComponent(name)}`, { method: "DELETE" }),
    onSuccess: () => { setConfirmDelete(null); invalidate(); },
    onError: (e) => setError(e instanceof Error ? e.message : "删除失败"),
  });

  const openCreate = () => {
    setError("");
    setForm({ name: "", description: "", permissions: [], isNew: true });
  };
  const openEdit = (r: Role) => {
    setError("");
    setForm({ name: r.name, description: r.description, permissions: [...r.permissions], isNew: false });
  };

  const togglePerm = (key: string) => {
    if (!form) return;
    const has = form.permissions.includes(key);
    setForm({
      ...form,
      permissions: has ? form.permissions.filter((p) => p !== key) : [...form.permissions, key],
    });
  };

  return (
    <div className="page">
      <div className="page-head">
        <h1 className="page-title">角色与权限</h1>
        {writable && <button className="primary-btn" onClick={openCreate}>新建角色</button>}
      </div>

      {isLoading ? (
        <Empty text="加载中…" />
      ) : (
        <div className="role-grid">
          {(roles ?? []).map((r) => (
            <div className="card role-card" key={r.name}>
              <div className="role-card-head">
                <RoleBadge role={r.name} />
                <span className="mono muted">{r.name}</span>
                {BUILTIN.includes(r.name) && <span className="tag-builtin">内置</span>}
              </div>
              <p>{r.description || "—"}</p>
              <div className="perm-list">
                {r.permissions.length === 0
                  ? <span className="muted">无权限</span>
                  : r.permissions.map((p) => <span className="perm mono" key={p}>{p}</span>)}
              </div>
              {writable && (
                <div className="role-card-actions">
                  {r.name !== "admin" && (
                    <button className="link-btn" onClick={() => openEdit(r)}>编辑</button>
                  )}
                  {!BUILTIN.includes(r.name) && (
                    <button className="link-btn danger" onClick={() => { setError(""); setConfirmDelete(r); }}>
                      删除
                    </button>
                  )}
                </div>
              )}
            </div>
          ))}
        </div>
      )}

      {form && (
        <Modal title={form.isNew ? "新建角色" : `编辑角色 · ${form.name}`} onClose={() => setForm(null)}>
          <div className="form">
            {form.isNew && (
              <label className="field">
                <span>角色标识(小写字母/数字/_/-)</span>
                <input value={form.name} placeholder="如 auditor"
                  onChange={(e) => setForm({ ...form, name: e.target.value })} />
              </label>
            )}
            <label className="field">
              <span>描述</span>
              <input value={form.description}
                onChange={(e) => setForm({ ...form, description: e.target.value })} />
            </label>
            <div className="field">
              <span>权限</span>
              <div className="perm-groups">
                {grouped.map(([group, list]) => (
                  <fieldset className="perm-group" key={group}>
                    <legend>{group}</legend>
                    {list.map((p) => (
                      <label className="perm-check" key={p.key}>
                        <input type="checkbox"
                          checked={form.permissions.includes(p.key)}
                          onChange={() => togglePerm(p.key)} />
                        <span>{p.label}</span>
                        <code className="mono muted">{p.key}</code>
                      </label>
                    ))}
                  </fieldset>
                ))}
              </div>
            </div>
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
        <Modal title="删除角色" onClose={() => setConfirmDelete(null)}>
          <div className="form">
            <p>确定删除角色 <strong>{confirmDelete.name}</strong> 吗?若仍有用户使用该角色将无法删除。</p>
            {error && <p className="form-error" role="alert">{error}</p>}
            <div className="form-actions">
              <button className="ghost-btn" onClick={() => setConfirmDelete(null)}>取消</button>
              <button className="danger-btn" disabled={deleteMut.isPending}
                onClick={() => deleteMut.mutate(confirmDelete.name)}>
                {deleteMut.isPending ? "删除中…" : "确认删除"}
              </button>
            </div>
          </div>
        </Modal>
      )}
    </div>
  );
}
