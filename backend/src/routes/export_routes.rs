use axum::{
    extract::{Query, State},
    http::header,
    response::IntoResponse,
    Extension,
};

use crate::{
    auth::AuthUser,
    csv,
    error::ApiResult,
    models::PageQuery,
    repo,
    state::AppState,
};

fn csv_response(filename: &str, body: String) -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8".to_string()),
            (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{filename}\"")),
        ],
        body,
    )
}

pub async fn users(
    State(state): State<AppState>,
    Extension(me): Extension<AuthUser>,
    Query(q): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    me.require("users.read")?;
    let users = repo::users::export(&state, q.q).await?;
    let rows = users.into_iter().map(|u| {
        vec![
            u.username,
            u.email,
            u.display_name,
            u.role,
            u.status,
            u.created_at,
            u.last_login_at.unwrap_or_default(),
        ]
    });
    let body = csv::build(
        &["用户名", "邮箱", "姓名", "角色", "状态", "创建时间", "最近登录"],
        rows,
    );
    Ok(csv_response("users.csv", body))
}

pub async fn audit(
    State(state): State<AppState>,
    Extension(me): Extension<AuthUser>,
    Query(q): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    me.require("audit.read")?;
    let logs = repo::audit::export(&state, q.q).await?;
    let rows = logs.into_iter().map(|a| {
        vec![a.created_at, a.username, a.action, a.target, a.detail, a.ip]
    });
    let body = csv::build(&["时间", "用户", "操作", "对象", "详情", "IP"], rows);
    Ok(csv_response("audit.csv", body))
}
