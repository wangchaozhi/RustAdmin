use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{error::ApiError, state::AppState};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,   // user id
    pub username: String,
    pub role: String,
    pub perms: Vec<String>,
    #[serde(default)] // 兼容升级前签发、不含 sid 的旧令牌
    pub sid: String,   // 当前会话(refresh_tokens.id)
    pub exp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MobileClaims {
    pub sub: String,
    pub username: String,
    pub sid: String,
    pub typ: String,
    pub exp: i64,
}

/// 注入到 request extensions 的当前用户
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: String,
    pub username: String,
    pub role: String,
    pub perms: Vec<String>,
    pub sid: String,
}

impl AuthUser {
    pub fn require(&self, perm: &str) -> Result<(), ApiError> {
        if self.perms.iter().any(|p| p == perm) {
            Ok(())
        } else {
            Err(ApiError::Forbidden)
        }
    }
}

pub fn hash_password(plain: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(plain.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| ApiError::Internal)
}

pub fn verify_password(plain: &str, hash: &str) -> bool {
    PasswordHash::new(hash)
        .map(|parsed| Argon2::default().verify_password(plain.as_bytes(), &parsed).is_ok())
        .unwrap_or(false)
}

pub fn issue_access_token(state: &AppState, user: &AuthUser) -> Result<(String, i64), ApiError> {
    let ttl = state.config.access_ttl_minutes;
    let exp = (Utc::now() + Duration::minutes(ttl)).timestamp();
    let claims = Claims {
        sub: user.id.clone(),
        username: user.username.clone(),
        role: user.role.clone(),
        perms: user.perms.clone(),
        sid: user.sid.clone(),
        exp,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .map_err(|_| ApiError::Internal)?;
    Ok((token, ttl * 60))
}

pub fn issue_mobile_access_token(
    state: &AppState,
    user_id: &str,
    username: &str,
    sid: &str,
) -> Result<(String, i64), ApiError> {
    let ttl = state.config.access_ttl_minutes;
    let exp = (Utc::now() + Duration::minutes(ttl)).timestamp();
    let claims = MobileClaims {
        sub: user_id.to_owned(),
        username: username.to_owned(),
        sid: sid.to_owned(),
        typ: "mobile".to_owned(),
        exp,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .map_err(|_| ApiError::Internal)?;
    Ok((token, ttl * 60))
}

pub fn decode_access_token(state: &AppState, token: &str) -> Result<Claims, ApiError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|d| d.claims)
    .map_err(|_| ApiError::Unauthorized)
}

/// 生成不透明 refresh token,返回 (明文, sha256哈希)
pub fn new_refresh_token() -> (String, String) {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let plain = hex::encode(bytes);
    (plain.clone(), hash_token(&plain))
}

pub fn hash_token(plain: &str) -> String {
    hex::encode(Sha256::digest(plain.as_bytes()))
}

/// Axum 中间件:校验 Bearer Token 并注入 AuthUser
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(ApiError::Unauthorized)?;

    let claims = decode_access_token(&state, token)?;
    req.extensions_mut().insert(AuthUser {
        id: claims.sub,
        username: claims.username,
        role: claims.role,
        perms: claims.perms,
        sid: claims.sid,
    });
    Ok(next.run(req).await)
}
