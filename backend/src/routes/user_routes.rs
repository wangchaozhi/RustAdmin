use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Path, Query, State},
    Extension, Json,
};
use serde_json::{json, Value};

use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
    models::{CreateUserReq, Page, PageQuery, UpdateUserReq, User},
    repo,
    state::AppState,
};

fn page_params(q: &PageQuery) -> (i64, i64) {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(10).clamp(1, 100);
    (page, page_size)
}

pub async fn list(
    State(state): State<AppState>,
    Extension(me): Extension<AuthUser>,
    Query(q): Query<PageQuery>,
) -> ApiResult<Json<Page<User>>> {
    me.require("users.read")?;
    let (page, page_size) = page_params(&q);
    Ok(Json(repo::users::list(&state, page, page_size, q.q).await?))
}

pub async fn create(
    State(state): State<AppState>,
    Extension(me): Extension<AuthUser>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<CreateUserReq>,
) -> ApiResult<Json<User>> {
    me.require("users.write")?;
    // 只有 admin 能创建 admin
    if req.role.as_deref() == Some("admin") && me.role != "admin" {
        return Err(ApiError::Forbidden);
    }
    let user = repo::users::create(&state, req).await?;
    repo::audit::record(&state, Some(&me.id), &me.username, "user.create", &user.username, "创建用户", &addr.ip().to_string()).await;
    Ok(Json(user))
}

pub async fn update(
    State(state): State<AppState>,
    Extension(me): Extension<AuthUser>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(id): Path<String>,
    Json(req): Json<UpdateUserReq>,
) -> ApiResult<Json<User>> {
    me.require("users.write")?;
    if req.role.is_some() && me.role != "admin" {
        return Err(ApiError::Forbidden);
    }
    if id == me.id && req.status.as_deref() == Some("disabled") {
        return Err(ApiError::BadRequest("不能禁用自己的账号".into()));
    }
    let user = repo::users::update(&state, &id, req).await?;
    repo::audit::record(&state, Some(&me.id), &me.username, "user.update", &user.username, "更新用户", &addr.ip().to_string()).await;
    Ok(Json(user))
}

pub async fn remove(
    State(state): State<AppState>,
    Extension(me): Extension<AuthUser>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    me.require("users.write")?;
    if id == me.id {
        return Err(ApiError::BadRequest("不能删除自己的账号".into()));
    }
    let target = repo::users::find_by_id(&state, &id).await?;
    if target.role == "admin" && me.role != "admin" {
        return Err(ApiError::Forbidden);
    }
    repo::users::delete(&state, &id).await?;
    repo::tokens::revoke_all(&state, &id).await?;
    repo::audit::record(&state, Some(&me.id), &me.username, "user.delete", &target.username, "删除用户", &addr.ip().to_string()).await;
    Ok(Json(json!({ "ok": true })))
}
