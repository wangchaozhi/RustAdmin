import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api, download } from "../api/client";
import type { AuditLog, Page } from "../api/types";
import { Empty, Pagination, formatTime } from "../components/ui";

export default function Audit() {
  const [page, setPage] = useState(1);
  const [q, setQ] = useState("");
  const [search, setSearch] = useState("");
  const pageSize = 20;

  const { data, isLoading } = useQuery({
    queryKey: ["audit", page, search],
    queryFn: () => api<Page<AuditLog>>(`/api/audit?page=${page}&page_size=${pageSize}&q=${encodeURIComponent(search)}`),
  });

  return (
    <div className="page">
      <h1 className="page-title">审计日志</h1>

      <div className="toolbar">
        <input className="search" placeholder="搜索用户 / 操作 / 对象" value={q}
          onChange={(e) => setQ(e.target.value)}
          onKeyDown={(e) => { if (e.key === "Enter") { setSearch(q); setPage(1); } }} />
        <button onClick={() => { setSearch(q); setPage(1); }}>搜索</button>
        <button onClick={() => download(`/api/export/audit?q=${encodeURIComponent(search)}`, "audit.csv")}>
          导出 CSV
        </button>
      </div>

      {isLoading ? (
        <Empty text="加载中…" />
      ) : !data || data.items.length === 0 ? (
        <Empty text="暂无日志记录" />
      ) : (
        <>
          <div className="card table-card">
            <table className="table">
              <thead>
                <tr><th>时间</th><th>用户</th><th>操作</th><th>对象</th><th>详情</th><th>IP</th></tr>
              </thead>
              <tbody>
                {data.items.map((a) => (
                  <tr key={a.id}>
                    <td className="mono">{formatTime(a.created_at)}</td>
                    <td>{a.username}</td>
                    <td><span className="mono feed-action">{a.action}</span></td>
                    <td>{a.target || "—"}</td>
                    <td>{a.detail || "—"}</td>
                    <td className="mono muted">{a.ip || "—"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          <div className="card-list">
            {data.items.map((a) => (
              <div className="card user-card" key={a.id}>
                <div className="user-card-head">
                  <span className="mono feed-action">{a.action}</span>
                  <time className="muted">{formatTime(a.created_at)}</time>
                </div>
                <div>{a.username}{a.target ? ` → ${a.target}` : ""}</div>
                <div className="muted">{a.detail || "—"} · {a.ip || "—"}</div>
              </div>
            ))}
          </div>

          <Pagination page={data.page} pageSize={data.page_size} total={data.total} onPage={setPage} />
        </>
      )}
    </div>
  );
}
