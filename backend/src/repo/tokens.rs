use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::{error::{ApiError, ApiResult}, models::now_iso, state::AppState};

/// 一条登录会话(刷新令牌)对外展示的信息
#[derive(serde::Serialize, sqlx::FromRow)]
pub struct SessionRow {
    pub id: String,
    pub ip: String,
    pub user_agent: String,
    pub created_at: String,
    pub expires_at: String,
}

/// 保存刷新令牌并返回新会话 id(refresh_tokens.id)
pub async fn store(
    state: &AppState,
    user_id: &str,
    token_hash: &str,
    ip: &str,
    user_agent: &str,
) -> ApiResult<String> {
    let id = Uuid::new_v4().to_string();
    let expires = (Utc::now() + Duration::days(state.config.refresh_ttl_days)).to_rfc3339();
    sqlx::query(
        "INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at, created_at, ip, user_agent)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(user_id)
    .bind(token_hash)
    .bind(expires)
    .bind(now_iso())
    .bind(ip)
    .bind(user_agent)
    .execute(&state.db)
    .await?;
    Ok(id)
}

pub async fn store_mobile(
    state: &AppState,
    user_id: &str,
    token_hash: &str,
    ip: &str,
    user_agent: &str,
) -> ApiResult<String> {
    let id = Uuid::new_v4().to_string();
    let expires = (Utc::now() + Duration::days(state.config.refresh_ttl_days)).to_rfc3339();
    sqlx::query(
        "INSERT INTO mobile_refresh_tokens (id, user_id, token_hash, expires_at, created_at, ip, user_agent)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(user_id)
    .bind(token_hash)
    .bind(expires)
    .bind(now_iso())
    .bind(ip)
    .bind(user_agent)
    .execute(&state.db)
    .await?;
    Ok(id)
}

/// 校验并旋转(删除旧的)refresh token,返回 user_id
pub async fn consume(state: &AppState, token_hash: &str) -> ApiResult<String> {
    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT user_id, expires_at FROM refresh_tokens WHERE token_hash = ?",
    )
    .bind(token_hash)
    .fetch_optional(&state.db)
    .await?;

    let (user_id, expires_at) = row.ok_or(ApiError::Unauthorized)?;

    sqlx::query("DELETE FROM refresh_tokens WHERE token_hash = ?")
        .bind(token_hash)
        .execute(&state.db)
        .await?;

    let exp = chrono::DateTime::parse_from_rfc3339(&expires_at)
        .map_err(|_| ApiError::Unauthorized)?;
    if exp < Utc::now() {
        return Err(ApiError::Unauthorized);
    }
    Ok(user_id)
}

pub async fn revoke_all(state: &AppState, user_id: &str) -> ApiResult<()> {
    sqlx::query("DELETE FROM refresh_tokens WHERE user_id = ?")
        .bind(user_id)
        .execute(&state.db)
        .await?;
    Ok(())
}

/// 列出某用户当前所有活跃会话,按创建时间倒序
pub async fn list_sessions(state: &AppState, user_id: &str) -> ApiResult<Vec<SessionRow>> {
    Ok(sqlx::query_as::<_, SessionRow>(
        "SELECT id, ip, user_agent, created_at, expires_at FROM refresh_tokens
         WHERE user_id = ? ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?)
}

/// 注销指定会话(仅限本人)
pub async fn revoke(state: &AppState, user_id: &str, session_id: &str) -> ApiResult<()> {
    let res = sqlx::query("DELETE FROM refresh_tokens WHERE id = ? AND user_id = ?")
        .bind(session_id)
        .bind(user_id)
        .execute(&state.db)
        .await?;
    if res.rows_affected() == 0 {
        return Err(ApiError::NotFound("会话不存在".into()));
    }
    Ok(())
}
