use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub display_name: String,
    pub role: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub last_login_at: Option<String>,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Role {
    pub name: String,
    pub description: String,
    pub permissions: String, // JSON 字符串,序列化前会被解析
}

#[derive(Debug, Serialize)]
pub struct RoleOut {
    pub name: String,
    pub description: String,
    pub permissions: Vec<String>,
}

impl From<Role> for RoleOut {
    fn from(r: Role) -> Self {
        let permissions = serde_json::from_str(&r.permissions).unwrap_or_default();
        Self { name: r.name, description: r.description, permissions }
    }
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct AuditLog {
    pub id: String,
    pub user_id: Option<String>,
    pub username: String,
    pub action: String,
    pub target: String,
    pub detail: String,
    pub ip: String,
    pub created_at: String,
}

// ---------- 请求体 ----------

#[derive(Debug, Deserialize)]
pub struct LoginReq {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshReq {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserReq {
    pub username: String,
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserReq {
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PageQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub q: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Serialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

pub fn now_iso() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.to_rfc3339()
}
