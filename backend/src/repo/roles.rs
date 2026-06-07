use crate::{error::ApiResult, models::{Role, RoleOut}, state::AppState};

pub async fn list(state: &AppState) -> ApiResult<Vec<RoleOut>> {
    let rows = sqlx::query_as::<_, Role>("SELECT * FROM roles ORDER BY name")
        .fetch_all(&state.db)
        .await?;
    Ok(rows.into_iter().map(RoleOut::from).collect())
}

pub async fn permissions_of(state: &AppState, role: &str) -> ApiResult<Vec<String>> {
    let perms: Option<String> = sqlx::query_scalar("SELECT permissions FROM roles WHERE name = ?")
        .bind(role)
        .fetch_optional(&state.db)
        .await?;
    Ok(perms
        .map(|p| serde_json::from_str(&p).unwrap_or_default())
        .unwrap_or_default())
}
