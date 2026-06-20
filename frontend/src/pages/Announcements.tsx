import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../api/client";
import type { Announcement, AnnouncementFeed, AnnouncementLevel } from "../api/types";
import { Empty, Modal, formatTime } from "../components/ui";
import { useAuth } from "../store/auth";

const LEVEL_LABELS: Record<AnnouncementLevel, string> = {
  info: "通知",
  warning: "重要",
  critical: "紧急",
};

function LevelBadge({ level }: { level: AnnouncementLevel }) {
  return <span className={`badge level-${level}`}>{LEVEL_LABELS[level]}</span>;
}

interface FormState {
  id?: string;
  title: string;
  body: string;
  level: AnnouncementLevel;
  published: boolean;
}

const EMPTY_FORM: FormState = { title: "", body: "", level: "info", published: true };

function ManagePanel() {
  const qc = useQueryClient();
  const [form, setForm] = useState<FormState | null>(null);
  const [confirmDelete, setConfirmDelete] = useState<Announcement | null>(null);
  const [error, setError] = useState("");

  const { data, isLoading } = useQuery({
    queryKey: ["announcements", "all"],
    queryFn: () => api<Announcement[]>("/api/admin/announcements"),
  });

  const invalidate = () => {
    qc.invalidateQueries({ queryKey: ["announcements"] });
  };

  const saveMut = useMutation({
    mutationFn: (f: FormState) => {
      const body = { title: f.title, body: f.body, level: f.level, published: f.published };
      return f.id
        ? api(`/api/admin/announcements/${f.id}`, { method: "PUT", body })
        : api("/api/admin/announcements", { method: "POST", body });
    },
    onSuccess: () => { setForm(null); setError(""); invalidate(); },
    onError: (e) => setError(e instanceof Error ? e.message : "保存失败"),
  });

  const deleteMut = useMutation({
    mutationFn: (id: string) => api(`/api/admin/announcements/${id}`, { method: "DELETE" }),
    onSuccess: () => { setConfirmDelete(null); invalidate(); },
  });

  const togglePublish = (a: Announcement) =>
    saveMut.mutate({ id: a.id, title: a.title, body: a.body, level: a.level, published: !a.published });

  const items = data ?? [];

  return (
    <section className="card">
      <div className="page-head">
        <h2 className="card-title" style={{ margin: 0 }}>公告管理</h2>
        <button className="primary-btn" onClick={() => { setError(""); setForm({ ...EMPTY_FORM }); }}>
          发布公告
        </button>
      </div>

      {isLoading ? (
        <Empty text="加载中…" />
      ) : items.length === 0 ? (
        <Empty text="还没有公告,点击「发布公告」创建" />
      ) : (
        <ul className="manage-list">
          {items.map((a) => (
            <li className="manage-item" key={a.id}>
              <div className="manage-main">
                <div className="manage-title">
                  <LevelBadge level={a.level} />
                  <strong>{a.title}</strong>
                  {!a.published && <span className="tag-draft">草稿</span>}
                </div>
                <div className="muted mono">{formatTime(a.created_at)} · {a.created_by}</div>
              </div>
              <div className="manage-actions">
                <button className="link-btn" onClick={() => togglePublish(a)}>
                  {a.published ? "下架" : "发布"}
                </button>
                <button className="link-btn" onClick={() => {
                  setError("");
                  setForm({ id: a.id, title: a.title, body: a.body, level: a.level, published: a.published });
                }}>编辑</button>
                <button className="link-btn danger" onClick={() => setConfirmDelete(a)}>删除</button>
              </div>
            </li>
          ))}
        </ul>
      )}

      {form && (
        <Modal title={form.id ? "编辑公告" : "发布公告"} onClose={() => setForm(null)}>
          <div className="form">
            <label className="field">
              <span>标题</span>
              <input value={form.title} onChange={(e) => setForm({ ...form, title: e.target.value })} />
            </label>
            <label className="field">
              <span>正文</span>
              <textarea className="textarea" rows={5} value={form.body}
                onChange={(e) => setForm({ ...form, body: e.target.value })} />
            </label>
            <div className="field-row">
              <label className="field">
                <span>级别</span>
                <select value={form.level}
                  onChange={(e) => setForm({ ...form, level: e.target.value as AnnouncementLevel })}>
                  <option value="info">通知</option>
                  <option value="warning">重要</option>
                  <option value="critical">紧急</option>
                </select>
              </label>
              <label className="field">
                <span>状态</span>
                <select value={form.published ? "1" : "0"}
                  onChange={(e) => setForm({ ...form, published: e.target.value === "1" })}>
                  <option value="1">立即发布</option>
                  <option value="0">存为草稿</option>
                </select>
              </label>
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
        <Modal title="删除公告" onClose={() => setConfirmDelete(null)}>
          <div className="form">
            <p>确定删除公告 <strong>{confirmDelete.title}</strong> 吗?该操作不可撤销。</p>
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
    </section>
  );
}

export default function Announcements() {
  const { can } = useAuth();
  const qc = useQueryClient();

  const { data, isLoading } = useQuery({
    queryKey: ["announcements", "feed"],
    queryFn: () => api<AnnouncementFeed>("/api/announcements"),
  });

  const readMut = useMutation({
    mutationFn: (id: string) => api(`/api/announcements/${id}/read`, { method: "POST" }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["announcements"] }),
  });

  const items = data?.items ?? [];

  return (
    <div className="page">
      <h1 className="page-title">系统公告</h1>

      {isLoading ? (
        <Empty text="加载中…" />
      ) : items.length === 0 ? (
        <Empty text="暂无公告" />
      ) : (
        <div className="announce-feed">
          {items.map((a) => (
            <article
              className={`card announce-card ${a.read ? "" : "unread"}`}
              key={a.id}
              onClick={() => { if (!a.read) readMut.mutate(a.id); }}
            >
              <div className="announce-head">
                <LevelBadge level={a.level} />
                <strong>{a.title}</strong>
                {!a.read && <span className="unread-dot" aria-label="未读" />}
              </div>
              {a.body && <p className="announce-body">{a.body}</p>}
              <div className="muted mono">{formatTime(a.created_at)} · {a.created_by}</div>
            </article>
          ))}
        </div>
      )}

      {can("announcements.write") && <ManagePanel />}
    </div>
  );
}
