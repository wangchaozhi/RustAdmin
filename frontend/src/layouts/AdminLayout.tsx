import { NavLink, Outlet, useNavigate } from "react-router-dom";
import { useAuth } from "../store/auth";

const NAV = [
  { to: "/", label: "概览", icon: "◧", perm: "dashboard.read", end: true },
  { to: "/users", label: "用户", icon: "◫", perm: "users.read" },
  { to: "/roles", label: "角色", icon: "◨", perm: "users.read" },
  { to: "/audit", label: "审计", icon: "≡", perm: "audit.read" },
];

export default function AdminLayout() {
  const { user, can, logout } = useAuth();
  const navigate = useNavigate();
  const items = NAV.filter((n) => can(n.perm));

  const onLogout = async () => {
    await logout();
    navigate("/login");
  };

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
            <span className="nav-icon" aria-hidden>{n.icon}</span>
            <span>{n.label}</span>
          </NavLink>
        ))}
      </nav>
    </div>
  );
}
