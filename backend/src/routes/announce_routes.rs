use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Path, State},
    Extension, Json,
};
use serde_json::{json, Value};

use crate::{
    auth::AuthUser,
    error::ApiResult,
    models::{Announcement, CreateAnnouncementReq, UpdateAnnouncementReq},
    repo,
    state::AppState,
};

/// 普通用户:获取已发布公告与未读数(任意登录用户可见)
pub async fn feed(
    State(state): State<AppState>,
    Extension(me): Extension<AuthUser>,
) -> ApiResult<Json<Value>> {
    let items = repo::announce::feed(&state, &me.id).await?;
    let unread = items.iter().filter(|a| !a.read).count();
    Ok(Json(json!({ "items": items, "unread": unread })))
}

pub async fn mark_read(
    State(state): State<AppState>,
    Extension(me): Extension<AuthUser>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    repo::announce::mark_read(&state, &id, &me.id).await?;
    Ok(Json(json!({ "ok": true })))
}

// ---------- 管理(需 announcements.write) ----------

pub async fn list_all(
    State(state): State<AppState>,
    Extension(me): Extension<AuthUser>,
) -> ApiResult<Json<Vec<Announcement>>> {
    me.require("announcements.write")?;
    Ok(Json(repo::announce::list_all(&state).await?))
}

pub async fn create(
    State(state): State<AppState>,
    Extension(me): Extension<AuthUser>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<CreateAnnouncementReq>,
) -> ApiResult<Json<Announcement>> {
    me.require("announcements.write")?;
    let a = repo::announce::create(&state, req, &me.username).await?;
    repo::audit::record(&state, Some(&me.id), &me.username, "announce.create", &a.title, "发布公告", &addr.ip().to_string()).await;
    Ok(Json(a))
}

pub async fn update(
    State(state): State<AppState>,
    Extension(me): Extension<AuthUser>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(id): Path<String>,
    Json(req): Json<UpdateAnnouncementReq>,
) -> ApiResult<Json<Announcement>> {
    me.require("announcements.write")?;
    let a = repo::announce::update(&state, &id, req).await?;
    repo::audit::record(&state, Some(&me.id), &me.username, "announce.update", &a.title, "更新公告", &addr.ip().to_string()).await;
    Ok(Json(a))
}

pub async fn remove(
    State(state): State<AppState>,
    Extension(me): Extension<AuthUser>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    me.require("announcements.write")?;
    repo::announce::delete(&state, &id).await?;
    repo::audit::record(&state, Some(&me.id), &me.username, "announce.delete", &id, "删除公告", &addr.ip().to_string()).await;
    Ok(Json(json!({ "ok": true })))
}
