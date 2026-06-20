use uuid::Uuid;

use crate::{
    auth,
    error::ApiResult,
    models::{now_iso, MobileUser},
    state::AppState,
};

pub async fn seed_mobile_user(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM mobile_users")
        .fetch_one(&state.db)
        .await?;
    if count == 0 {
        let now = now_iso();
        let hash = auth::hash_password("123456").map_err(|_| "hash failed")?;
        sqlx::query(
            "INSERT INTO mobile_users (id, username, password_hash, display_name, status, created_at, updated_at)
             VALUES (?, 'user', ?, 'Mobile User', 'active', ?, ?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(hash)
        .bind(&now)
        .bind(&now)
        .execute(&state.db)
        .await?;
        tracing::warn!("created default mobile user user / 123456; change it before production");
    }
    Ok(())
}

pub async fn find_by_username(state: &AppState, username: &str) -> ApiResult<Option<MobileUser>> {
    Ok(
        sqlx::query_as::<_, MobileUser>("SELECT * FROM mobile_users WHERE username = ?")
            .bind(username)
            .fetch_optional(&state.db)
            .await?,
    )
}

pub async fn touch_login(state: &AppState, id: &str) -> ApiResult<()> {
    sqlx::query("UPDATE mobile_users SET last_login_at = ? WHERE id = ?")
        .bind(now_iso())
        .bind(id)
        .execute(&state.db)
        .await?;
    Ok(())
}
