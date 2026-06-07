import { useQuery } from "@tanstack/react-query";
import { api } from "../api/client";
import type { DashboardStats } from "../api/types";
import { Empty, formatTime } from "../components/ui";

function TrendChart({ data }: { data: { day: string; count: number }[] }) {
  if (data.length === 0) return <Empty text="近 7 天暂无登录记录" />;
  const w = 560, h = 140, pad = 8;
  const max = Math.max(...data.map((d) => d.count), 1);
  const step = data.length > 1 ? (w - pad * 2) / (data.length - 1) : 0;
  const points = data.map((d, i) => {
    const x = pad + i * step;
    const y = h - pad - (d.count / max) * (h - pad * 2);
    return `${x},${y}`;
  });
  return (
    <svg viewBox={`0 0 ${w} ${h}`} className="trend" role="img" aria-label="近 7 天登录趋势">
      <polyline points={points.join(" ")} fill="none" stroke="var(--accent)" strokeWidth="2.5" />
      {points.map((p, i) => {
        const [x, y] = p.split(",").map(Number);
        return <circle key={i} cx={x} cy={y} r="3.5" fill="var(--accent)" />;
      })}
    </svg>
  );
}

export default function Dashboard() {
  const { data, isLoading, error } = useQuery({
    queryKey: ["dashboard"],
    queryFn: () => api<DashboardStats>("/api/dashboard"),
    refetchInterval: 30_000,
  });

  if (isLoading) return <div className="page"><Empty text="加载中…" /></div>;
  if (error || !data) return <div className="page"><Empty text="加载概览失败,请刷新重试" /></div>;

  const stats = [
    { label: "用户总数", value: data.total_users },
    { label: "启用中", value: data.active_users },
    { label: "已禁用", value: data.disabled_users },
    { label: "今日登录", value: data.logins_today },
  ];

  return (
    <div className="page">
      <h1 className="page-title">概览</h1>

      <div className="stat-grid">
        {stats.map((s) => (
          <div className="card stat" key={s.label}>
            <span className="stat-value mono">{s.value}</span>
            <span className="stat-label">{s.label}</span>
          </div>
        ))}
      </div>

      <div className="dash-grid">
        <section className="card">
          <h2 className="card-title">近 7 天登录趋势</h2>
          <TrendChart data={data.logins_last_7_days} />
        </section>

        <section className="card">
          <h2 className="card-title">最近操作</h2>
          {data.recent_actions.length === 0 ? (
            <Empty text="还没有操作记录" />
          ) : (
            <ul className="feed">
              {data.recent_actions.map((a) => (
                <li key={a.id}>
                  <span className="mono feed-action">{a.action}</span>
                  <span>{a.username}{a.target ? ` → ${a.target}` : ""}</span>
                  <time className="muted">{formatTime(a.created_at)}</time>
                </li>
              ))}
            </ul>
          )}
        </section>
      </div>
    </div>
  );
}
