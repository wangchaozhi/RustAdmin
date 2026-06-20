use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Path, State},
    Extension, Json,
};
use serde_json::{json, Value};

use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
    models::{CreateRoleReq, RoleOut, UpdateRoleReq},
    repo,
    state::AppState,
};

pub async fn create(
    State(state): State<AppState>,
    Extension(me): Extension<AuthUser>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<CreateRoleReq>,
) -> ApiResult<Json<RoleOut>> {
    me.require("roles.write")?;
    let role = repo::roles::create(&state, req).await?;
    repo::audit::record(&state, Some(&me.id), &me.username, "role.create", &role.name, "创建角色", &addr.ip().to_string()).await;
    Ok(Json(role))
}

pub async fn update(
    State(state): State<AppState>,
    Extension(me): Extension<AuthUser>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(name): Path<String>,
    Json(req): Json<UpdateRoleReq>,
) -> ApiResult<Json<RoleOut>> {
    me.require("roles.write")?;
    // admin 角色锁定,防止误删权限导致系统失去管理员
    if name == "admin" {
        return Err(ApiError::BadRequest("内置 admin 角色不可修改".into()));
    }
    let role = repo::roles::update(&state, &name, req).await?;
    repo::audit::record(&state, Some(&me.id), &me.username, "role.update", &role.name, "更新角色", &addr.ip().to_string()).await;
    Ok(Json(role))
}

pub async fn remove(
    State(state): State<AppState>,
    Extension(me): Extension<AuthUser>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(name): Path<String>,
) -> ApiResult<Json<Value>> {
    me.require("roles.write")?;
    repo::roles::delete(&state, &name).await?;
    repo::audit::record(&state, Some(&me.id), &me.username, "role.delete", &name, "删除角色", &addr.ip().to_string()).await;
    Ok(Json(json!({ "ok": true })))
}
