export interface User {
  id: string;
  username: string;
  email: string;
  display_name: string;
  role: string;
  status: "active" | "disabled";
  created_at: string;
  updated_at: string;
  last_login_at: string | null;
}

export interface Role {
  name: string;
  description: string;
  permissions: string[];
}

export interface AuditLog {
  id: string;
  user_id: string | null;
  username: string;
  action: string;
  target: string;
  detail: string;
  ip: string;
  created_at: string;
}

export interface Page<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
}

export interface TokenPair {
  access_token: string;
  refresh_token: string;
  expires_in: number;
}

export interface DayCount {
  day: string;
  count: number;
}

export interface DashboardStats {
  total_users: number;
  active_users: number;
  disabled_users: number;
  logins_today: number;
  recent_actions: AuditLog[];
  logins_last_7_days: DayCount[];
}
