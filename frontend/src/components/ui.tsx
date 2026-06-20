import { useEffect } from "react";

export function Modal({ title, onClose, children }: {
  title: string;
  onClose: () => void;
  children: React.ReactNode;
}) {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") onClose(); };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  return (
    <div className="modal-backdrop" onClick={onClose} role="dialog" aria-modal="true" aria-label={title}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <header className="modal-head">
          <h2>{title}</h2>
          <button className="icon-btn" onClick={onClose} aria-label="关闭">✕</button>
        </header>
        {children}
      </div>
    </div>
  );
}

export function Pagination({ page, pageSize, total, onPage }: {
  page: number;
  pageSize: number;
  total: number;
  onPage: (p: number) => void;
}) {
  const pages = Math.max(1, Math.ceil(total / pageSize));
  return (
    <div className="pagination">
      <span className="muted">共 {total} 条</span>
      <div className="pagination-ctrl">
        <button disabled={page <= 1} onClick={() => onPage(page - 1)}>上一页</button>
        <span className="mono">{page} / {pages}</span>
        <button disabled={page >= pages} onClick={() => onPage(page + 1)}>下一页</button>
      </div>
    </div>
  );
}

export function StatusBadge({ status }: { status: string }) {
  const active = status === "active";
  return (
    <span className={`badge ${active ? "badge-ok" : "badge-off"}`}>
      {active ? "启用" : "禁用"}
    </span>
  );
}

const ROLE_LABELS: Record<string, string> = { admin: "管理员", editor: "编辑", viewer: "只读" };

export function RoleBadge({ role }: { role: string }) {
  const label = ROLE_LABELS[role] ?? role;
  return <span className={`badge badge-role badge-role-${role}`}>{label}</span>;
}

export function Empty({ text }: { text: string }) {
  return <div className="empty">{text}</div>;
}

export function formatTime(iso: string | null): string {
  if (!iso) return "—";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  return d.toLocaleString("zh-CN", { hour12: false });
}
