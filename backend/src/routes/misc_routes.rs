use axum::{
    extract::{Query, State},
    Extension, Json,
};
use serde_json::{json, Value};

use crate::{
    auth::AuthUser,
    error::ApiResult,
    models::{AuditLog, Page, PageQuery, RoleOut},
    perms,
    repo::{self, audit::DashboardStats},
    state::AppState,
};

pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

pub async fn list_permissions(
    Extension(me): Extension<AuthUser>,
) -> ApiResult<Json<Vec<perms::PermItem>>> {
    me.require("roles.read").or_else(|_| me.require("roles.write"))?;
    Ok(Json(perms::catalog()))
}

pub async fn dashboard(
    State(state): State<AppState>,
    Extension(me): Extension<AuthUser>,
) -> ApiResult<Json<DashboardStats>> {
    me.require("dashboard.read")?;
    Ok(Json(repo::audit::dashboard(&state).await?))
}

pub async fn list_roles(
    State(state): State<AppState>,
    Extension(me): Extension<AuthUser>,
) -> ApiResult<Json<Vec<RoleOut>>> {
    me.require("roles.read").or_else(|_| me.require("users.read"))?;
    Ok(Json(repo::roles::list(&state).await?))
}

pub async fn list_audit(
    State(state): State<AppState>,
    Extension(me): Extension<AuthUser>,
    Query(q): Query<PageQuery>,
) -> ApiResult<Json<Page<AuditLog>>> {
    me.require("audit.read")?;
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).clamp(1, 100);
    Ok(Json(repo::audit::list(&state, page, page_size, q.q).await?))
}
