mod auth_routes;
mod misc_routes;
mod user_routes;

use axum::{middleware, routing::{get, post, put, delete}, Router};

use crate::{auth::auth_middleware, state::AppState};

pub fn router(state: AppState) -> Router {
    let public = Router::new()
        .route("/api/health", get(misc_routes::health))
        .route("/api/auth/login", post(auth_routes::login))
        .route("/api/auth/refresh", post(auth_routes::refresh));

    let protected = Router::new()
        .route("/api/auth/me", get(auth_routes::me))
        .route("/api/auth/logout", post(auth_routes::logout))
        .route("/api/dashboard", get(misc_routes::dashboard))
        .route("/api/roles", get(misc_routes::list_roles))
        .route("/api/audit", get(misc_routes::list_audit))
        .route("/api/users", get(user_routes::list).post(user_routes::create))
        .route("/api/users/:id", put(user_routes::update).delete(user_routes::remove))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    public.merge(protected).with_state(state)
}
