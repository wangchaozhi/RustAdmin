use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Path, State},
    http::{header, HeaderMap},
    Extension, Json,
};
use serde_json::{json, Value};

use crate::{
    auth::{self, AuthUser},
    error::{ApiError, ApiResult},
    models::{ChangePasswordReq, LoginReq, RefreshReq, TokenPair, UpdateProfileReq},
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
    let ip = addr.ip().to_string();

    let (refresh_plain, refresh_hash) = auth::new_refresh_token();
    let sid = repo::tokens::store(&state, &user.id, &refresh_hash, &ip, &user_agent(&headers)).await?;

    let auth_user = AuthUser {
        id: user.id.clone(),
        username: user.username.clone(),
        role: user.role.clone(),
        perms,
        sid,
    };
    let (access_token, expires_in) = auth::issue_access_token(&state, &auth_user)?;

    repo::users::touch_login(&state, &user.id).await?;
    repo::audit::record(&state, Some(&user.id), &user.username, "login", "", "登录成功", &ip).await;

    Ok(Json(json!({
        "tokens": TokenPair { access_token, refresh_token: refresh_plain, expires_in },
        "user": user,
    })))
}

pub async fn refresh(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(req): Json<RefreshReq>,
) -> ApiResult<Json<TokenPair>> {
    let user_id = repo::tokens::consume(&state, &auth::hash_token(&req.refresh_token)).await?;
    let user = repo::users::find_by_id(&state, &user_id).await?;
    if user.status != "active" {
        return Err(ApiError::Forbidden);
    }
    let perms = repo::roles::permissions_of(&state, &user.role).await?;
    let ip = addr.ip().to_string();

    let (refresh_plain, refresh_hash) = auth::new_refresh_token();
    let sid = repo::tokens::store(&state, &user.id, &refresh_hash, &ip, &user_agent(&headers)).await?;

    let auth_user = AuthUser { id: user.id.clone(), username: user.username, role: user.role, perms, sid };
    let (access_token, expires_in) = auth::issue_access_token(&state, &auth_user)?;

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

pub async fn update_profile(
    State(state): State<AppState>,
    Extension(me): Extension<AuthUser>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<UpdateProfileReq>,
) -> ApiResult<Json<Value>> {
    let user = repo::users::update_self(&state, &me.id, req.display_name, req.email).await?;
    repo::audit::record(&state, Some(&me.id), &me.username, "auth.profile_update", &user.username, "更新个人资料", &addr.ip().to_string()).await;
    Ok(Json(json!({ "user": user })))
}

pub async fn change_password(
    State(state): State<AppState>,
    Extension(me): Extension<AuthUser>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<ChangePasswordReq>,
) -> ApiResult<Json<Value>> {
    let user = repo::users::find_by_id(&state, &me.id).await?;
    if !auth::verify_password(&req.current_password, &user.password_hash) {
        return Err(ApiError::BadRequest("当前密码不正确".into()));
    }
    if req.new_password.len() < 8 {
        return Err(ApiError::BadRequest("新密码至少 8 位".into()));
    }
    repo::users::set_password(&state, &me.id, &req.new_password).await?;
    // 改密后吊销全部会话,强制各端重新登录
    repo::tokens::revoke_all(&state, &me.id).await?;
    repo::audit::record(&state, Some(&me.id), &me.username, "auth.change_password", &user.username, "修改密码", &addr.ip().to_string()).await;
    Ok(Json(json!({ "ok": true })))
}

pub async fn list_sessions(
    State(state): State<AppState>,
    Extension(me): Extension<AuthUser>,
) -> ApiResult<Json<Value>> {
    let rows = repo::tokens::list_sessions(&state, &me.id).await?;
    let items: Vec<Value> = rows
        .into_iter()
        .map(|s| {
            json!({
                "id": s.id,
                "ip": s.ip,
                "user_agent": s.user_agent,
                "created_at": s.created_at,
                "expires_at": s.expires_at,
                "current": s.id == me.sid,
            })
        })
        .collect();
    Ok(Json(json!({ "items": items })))
}

pub async fn revoke_session(
    State(state): State<AppState>,
    Extension(me): Extension<AuthUser>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    repo::tokens::revoke(&state, &me.id, &id).await?;
    Ok(Json(json!({ "ok": true })))
}
