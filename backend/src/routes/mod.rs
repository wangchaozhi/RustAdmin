mod announce_routes;
mod auth_routes;
mod export_routes;
mod misc_routes;
mod mobile_routes;
mod role_routes;
mod user_routes;

use axum::{middleware, routing::{get, post, put, delete}, Router};

use crate::{auth::auth_middleware, state::AppState};

pub fn router(state: AppState) -> Router {
    let public = Router::new()
        .route("/api/health", get(misc_routes::health))
        .route("/api/auth/login", post(auth_routes::login))
        .route("/api/auth/refresh", post(auth_routes::refresh))
        .route("/api/mobile/login", post(mobile_routes::login));

    let protected = Router::new()
        .route("/api/auth/me", get(auth_routes::me))
        .route("/api/auth/logout", post(auth_routes::logout))
        .route("/api/auth/profile", put(auth_routes::update_profile))
        .route("/api/auth/change-password", post(auth_routes::change_password))
        .route("/api/auth/sessions", get(auth_routes::list_sessions))
        .route("/api/auth/sessions/:id", delete(auth_routes::revoke_session))
        .route("/api/dashboard", get(misc_routes::dashboard))
        .route("/api/permissions", get(misc_routes::list_permissions))
        .route("/api/roles", get(misc_routes::list_roles).post(role_routes::create))
        .route("/api/roles/:name", put(role_routes::update).delete(role_routes::remove))
        .route("/api/audit", get(misc_routes::list_audit))
        .route("/api/users", get(user_routes::list).post(user_routes::create))
        .route("/api/users/:id", put(user_routes::update).delete(user_routes::remove))
        .route("/api/export/users", get(export_routes::users))
        .route("/api/export/audit", get(export_routes::audit))
        .route("/api/announcements", get(announce_routes::feed))
        .route("/api/announcements/:id/read", post(announce_routes::mark_read))
        .route("/api/admin/announcements", get(announce_routes::list_all).post(announce_routes::create))
        .route("/api/admin/announcements/:id", put(announce_routes::update).delete(announce_routes::remove))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    public.merge(protected).with_state(state)
}
