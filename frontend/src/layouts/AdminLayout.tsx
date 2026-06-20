import { NavLink, Outlet, useNavigate } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { api } from "../api/client";
import type { AnnouncementFeed } from "../api/types";
import { useAuth } from "../store/auth";

interface NavItem {
  to: string;
  label: string;
  icon: string;
  perm?: string;
  end?: boolean;
}

const NAV: NavItem[] = [
  { to: "/", label: "概览", icon: "◧", perm: "dashboard.read", end: true },
  { to: "/users", label: "用户", icon: "◫", perm: "users.read" },
  { to: "/roles", label: "角色", icon: "◨", perm: "users.read" },
  { to: "/audit", label: "审计", icon: "≡", perm: "audit.read" },
  { to: "/announcements", label: "公告", icon: "❖" },
  { to: "/profile", label: "我的", icon: "◓" },
];

export default function AdminLayout() {
  const { user, can, logout } = useAuth();
  const navigate = useNavigate();
  const items = NAV.filter((n) => !n.perm || can(n.perm));

  const { data: feed } = useQuery({
    queryKey: ["announcements", "feed"],
    queryFn: () => api<AnnouncementFeed>("/api/announcements"),
    staleTime: 30_000,
    refetchInterval: 60_000,
  });
  const unread = feed?.unread ?? 0;

  const onLogout = async () => {
    await logout();
    navigate("/login");
  };

  const badge = (to: string) =>
    to === "/announcements" && unread > 0
      ? <span className="nav-badge">{unread > 99 ? "99+" : unread}</span>
      : null;

  return (
    <div className="shell">
      {/* 桌面侧边栏 */}
      <aside className="sidebar">
        <div className="brand">
          <span className="brand-mark">H</span>
          <span className="brand-name">Harbor 控制台</span>
        </div>
        <nav className="side-nav">
          {items.map((n) => (
            <NavLink key={n.to} to={n.to} end={n.end}
              className={({ isActive }) => `side-link ${isActive ? "active" : ""}`}>
              <span className="nav-icon" aria-hidden>{n.icon}</span>
              {n.label}
              {badge(n.to)}
            </NavLink>
          ))}
        </nav>
        <div className="side-foot">
          <div className="me">
            <div className="avatar">{(user?.display_name || user?.username || "?").slice(0, 1)}</div>
            <div className="me-info">
              <strong>{user?.display_name || user?.username}</strong>
              <span className="muted">{user?.role}</span>
            </div>
          </div>
          <button className="ghost-btn" onClick={onLogout}>退出登录</button>
        </div>
      </aside>

      {/* 移动端顶栏 */}
      <header className="topbar">
        <div className="brand">
          <span className="brand-mark">H</span>
          <span className="brand-name">Harbor</span>
        </div>
        <button className="ghost-btn" onClick={onLogout}>退出</button>
      </header>

      <main className="content">
        <Outlet />
      </main>

      {/* 移动端底部 Tab */}
      <nav className="tabbar">
        {items.map((n) => (
          <NavLink key={n.to} to={n.to} end={n.end}
            className={({ isActive }) => `tab-link ${isActive ? "active" : ""}`}>
            <span className="nav-icon" aria-hidden>{n.icon}{badge(n.to)}</span>
            <span>{n.label}</span>
          </NavLink>
        ))}
      </nav>
    </div>
  );
}
