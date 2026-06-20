use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, State},
    http::{header, HeaderMap},
    Json,
};
use serde_json::{json, Value};

use crate::{
    auth,
    error::{ApiError, ApiResult},
    models::{LoginReq, TokenPair},
    repo,
    state::AppState,
};

fn user_agent(headers: &HeaderMap) -> String {
    headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .chars()
        .take(255)
        .collect()
}

pub async fn login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(req): Json<LoginReq>,
) -> ApiResult<Json<Value>> {
    let user = repo::mobile_users::find_by_username(&state, req.username.trim())
        .await?
        .ok_or_else(|| ApiError::BadRequest("invalid username or password".into()))?;

    if !auth::verify_password(&req.password, &user.password_hash) {
        return Err(ApiError::BadRequest("invalid username or password".into()));
    }
    if user.status != "active" {
        return Err(ApiError::Forbidden);
    }

    let ip = addr.ip().to_string();
    let (refresh_plain, refresh_hash) = auth::new_refresh_token();
    let sid =
        repo::tokens::store_mobile(&state, &user.id, &refresh_hash, &ip, &user_agent(&headers))
            .await?;
    let (access_token, expires_in) =
        auth::issue_mobile_access_token(&state, &user.id, &user.username, &sid)?;

    repo::mobile_users::touch_login(&state, &user.id).await?;

    Ok(Json(json!({
        "tokens": TokenPair { access_token, refresh_token: refresh_plain, expires_in },
        "user": user,
    })))
}
