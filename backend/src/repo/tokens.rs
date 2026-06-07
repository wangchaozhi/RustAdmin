use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::{error::{ApiError, ApiResult}, models::now_iso, state::AppState};

pub async fn store(state: &AppState, user_id: &str, token_hash: &str) -> ApiResult<()> {
    let expires = (Utc::now() + Duration::days(state.config.refresh_ttl_days)).to_rfc3339();
    sqlx::query(
        "INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(user_id)
    .bind(token_hash)
    .bind(expires)
    .bind(now_iso())
    .execute(&state.db)
    .await?;
    Ok(())
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
