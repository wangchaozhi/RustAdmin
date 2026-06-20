use uuid::Uuid;

use crate::{
    error::ApiResult,
    models::{now_iso, AuditLog, Page},
    state::AppState,
};

pub async fn record(
    state: &AppState,
    user_id: Option<&str>,
    username: &str,
    action: &str,
    target: &str,
    detail: &str,
    ip: &str,
) {
    let result = sqlx::query(
        "INSERT INTO audit_logs (id, user_id, username, action, target, detail, ip, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(user_id)
    .bind(username)
    .bind(action)
    .bind(target)
    .bind(detail)
    .bind(ip)
    .bind(now_iso())
    .execute(&state.db)
    .await;
    if let Err(e) = result {
        tracing::error!("写入审计日志失败: {e}");
    }
}

pub async fn list(state: &AppState, page: i64, page_size: i64, q: Option<String>) -> ApiResult<Page<AuditLog>> {
    let like = format!("%{}%", q.unwrap_or_default());
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_logs WHERE username LIKE ? OR action LIKE ? OR target LIKE ?",
    )
    .bind(&like).bind(&like).bind(&like)
    .fetch_one(&state.db)
    .await?;

    let items = sqlx::query_as::<_, AuditLog>(
        "SELECT * FROM audit_logs WHERE username LIKE ? OR action LIKE ? OR target LIKE ?
         ORDER BY created_at DESC LIMIT ? OFFSET ?",
    )
    .bind(&like).bind(&like).bind(&like)
    .bind(page_size)
    .bind((page - 1) * page_size)
    .fetch_all(&state.db)
    .await?;

    Ok(Page { items, total, page, page_size })
}

/// 导出匹配关键词的全部审计日志(用于 CSV),上限保护
pub async fn export(state: &AppState, q: Option<String>) -> ApiResult<Vec<AuditLog>> {
    let like = format!("%{}%", q.unwrap_or_default());
    Ok(sqlx::query_as::<_, AuditLog>(
        "SELECT * FROM audit_logs WHERE username LIKE ? OR action LIKE ? OR target LIKE ?
         ORDER BY created_at DESC LIMIT 50000",
    )
    .bind(&like).bind(&like).bind(&like)
    .fetch_all(&state.db)
    .await?)
}

#[derive(serde::Serialize)]
pub struct DashboardStats {
    pub total_users: i64,
    pub active_users: i64,
    pub disabled_users: i64,
    pub logins_today: i64,
    pub recent_actions: Vec<AuditLog>,
    pub logins_last_7_days: Vec<DayCount>,
}

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct DayCount {
    pub day: String,
    pub count: i64,
}

pub async fn dashboard(state: &AppState) -> ApiResult<DashboardStats> {
    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users").fetch_one(&state.db).await?;
    let active_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE status='active'")
        .fetch_one(&state.db).await?;
    let logins_today: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_logs WHERE action='login' AND date(created_at)=date('now')",
    )
    .fetch_one(&state.db).await?;
    let recent_actions = sqlx::query_as::<_, AuditLog>(
        "SELECT * FROM audit_logs ORDER BY created_at DESC LIMIT 8",
    )
    .fetch_all(&state.db).await?;
    let logins_last_7_days = sqlx::query_as::<_, DayCount>(
        "SELECT date(created_at) AS day, COUNT(*) AS count FROM audit_logs
         WHERE action='login' AND created_at >= datetime('now','-7 days')
         GROUP BY date(created_at) ORDER BY day",
    )
    .fetch_all(&state.db).await?;

    Ok(DashboardStats {
        disabled_users: total_users - active_users,
        total_users,
        active_users,
        logins_today,
        recent_actions,
        logins_last_7_days,
    })
}
