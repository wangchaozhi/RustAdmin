use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, State},
    Extension, Json,
};
use serde_json::{json, Value};

use crate::{
    auth::{self, AuthUser},
    error::{ApiError, ApiResult},
    models::{LoginReq, RefreshReq, TokenPair},
    repo,
    state::AppState,
};

pub async fn login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<LoginReq>,
) -> ApiResult<Json<Value>> {
    let user = repo::users::find_by_username(&state, req.username.trim())
        .await?
        .ok_or_else(|| ApiError::BadRequest("用户名或密码错误".into()))?;

    if !auth::verify_password(&req.password, &user.password_hash) {
        return Err(ApiError::BadRequest("用户名或密码错误".into()));
    }
    if user.status != "active" {
        return Err(ApiError::Forbidden);
    }

    let perms = repo::roles::permissions_of(&state, &user.role).await?;
    let auth_user = AuthUser {
        id: user.id.clone(),
        username: user.username.clone(),
        role: user.role.clone(),
        perms,
    };

    let (access_token, expires_in) = auth::issue_access_token(&state, &auth_user)?;
    let (refresh_plain, refresh_hash) = auth::new_refresh_token();
    repo::tokens::store(&state, &user.id, &refresh_hash).await?;
    repo::users::touch_login(&state, &user.id).await?;
    repo::audit::record(&state, Some(&user.id), &user.username, "login", "", "登录成功", &addr.ip().to_string()).await;

    Ok(Json(json!({
        "tokens": TokenPair { access_token, refresh_token: refresh_plain, expires_in },
        "user": user,
    })))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshReq>,
) -> ApiResult<Json<TokenPair>> {
    let user_id = repo::tokens::consume(&state, &auth::hash_token(&req.refresh_token)).await?;
    let user = repo::users::find_by_id(&state, &user_id).await?;
    if user.status != "active" {
        return Err(ApiError::Forbidden);
    }
    let perms = repo::roles::permissions_of(&state, &user.role).await?;
    let auth_user = AuthUser { id: user.id.clone(), username: user.username, role: user.role, perms };

    let (access_token, expires_in) = auth::issue_access_token(&state, &auth_user)?;
    let (refresh_plain, refresh_hash) = auth::new_refresh_token();
    repo::tokens::store(&state, &user.id, &refresh_hash).await?;

    Ok(Json(TokenPair { access_token, refresh_token: refresh_plain, expires_in }))
}

pub async fn logout(
    State(state): State<AppState>,
    Extension(me): Extension<AuthUser>,
) -> ApiResult<Json<Value>> {
    repo::tokens::revoke_all(&state, &me.id).await?;
    Ok(Json(json!({ "ok": true })))
}

pub async fn me(
    State(state): State<AppState>,
    Extension(me): Extension<AuthUser>,
) -> ApiResult<Json<Value>> {
    let user = repo::users::find_by_id(&state, &me.id).await?;
    Ok(Json(json!({ "user": user, "permissions": me.perms })))
}
