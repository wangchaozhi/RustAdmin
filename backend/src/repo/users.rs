use uuid::Uuid;

use crate::{
    auth,
    error::{ApiError, ApiResult},
    models::{now_iso, CreateUserReq, Page, UpdateUserReq, User},
    state::AppState,
};

pub async fn seed_admin(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await?;
    if count == 0 {
        let now = now_iso();
        let hash = auth::hash_password("Admin@12345").map_err(|_| "hash failed")?;
        sqlx::query(
            "INSERT INTO users (id, username, email, password_hash, display_name, role, status, created_at, updated_at)
             VALUES (?, 'admin', 'admin@example.com', ?, '系统管理员', 'admin', 'active', ?, ?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(hash)
        .bind(&now)
        .bind(&now)
        .execute(&state.db)
        .await?;
        tracing::warn!("已创建默认管理员 admin / Admin@12345 —— 请立即修改密码");
    }
    Ok(())
}

pub async fn find_by_username(state: &AppState, username: &str) -> ApiResult<Option<User>> {
    Ok(sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(&state.db)
        .await?)
}

pub async fn find_by_id(state: &AppState, id: &str) -> ApiResult<User> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("用户不存在".into()))
}

pub async fn list(state: &AppState, page: i64, page_size: i64, q: Option<String>) -> ApiResult<Page<User>> {
    let like = format!("%{}%", q.unwrap_or_default());
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users WHERE username LIKE ? OR email LIKE ? OR display_name LIKE ?",
    )
    .bind(&like).bind(&like).bind(&like)
    .fetch_one(&state.db)
    .await?;

    let items = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE username LIKE ? OR email LIKE ? OR display_name LIKE ?
         ORDER BY created_at DESC LIMIT ? OFFSET ?",
    )
    .bind(&like).bind(&like).bind(&like)
    .bind(page_size)
    .bind((page - 1) * page_size)
    .fetch_all(&state.db)
    .await?;

    Ok(Page { items, total, page, page_size })
}

pub async fn create(state: &AppState, req: CreateUserReq) -> ApiResult<User> {
    if req.username.trim().len() < 3 {
        return Err(ApiError::BadRequest("用户名至少 3 个字符".into()));
    }
    if !req.email.contains('@') {
        return Err(ApiError::BadRequest("邮箱格式不正确".into()));
    }
    if req.password.len() < 8 {
        return Err(ApiError::BadRequest("密码至少 8 位".into()));
    }
    if find_by_username(state, &req.username).await?.is_some() {
        return Err(ApiError::Conflict("用户名已存在".into()));
    }

    let id = Uuid::new_v4().to_string();
    let now = now_iso();
    let hash = auth::hash_password(&req.password)?;
    let role = req.role.unwrap_or_else(|| "viewer".into());

    sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, display_name, role, status, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, 'active', ?, ?)",
    )
    .bind(&id)
    .bind(req.username.trim())
    .bind(req.email.trim())
    .bind(hash)
    .bind(req.display_name.unwrap_or_default())
    .bind(role)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref d) if d.is_unique_violation() => ApiError::Conflict("用户名或邮箱已存在".into()),
        other => ApiError::Db(other),
    })?;

    find_by_id(state, &id).await
}

pub async fn update(state: &AppState, id: &str, req: UpdateUserReq) -> ApiResult<User> {
    let mut user = find_by_id(state, id).await?;

    if let Some(email) = req.email {
        if !email.contains('@') {
            return Err(ApiError::BadRequest("邮箱格式不正确".into()));
        }
        user.email = email;
    }
    if let Some(dn) = req.display_name { user.display_name = dn; }
    if let Some(role) = req.role { user.role = role; }
    if let Some(status) = req.status {
        if status != "active" && status != "disabled" {
            return Err(ApiError::BadRequest("状态只能是 active 或 disabled".into()));
        }
        user.status = status;
    }
    if let Some(pwd) = req.password {
        if pwd.len() < 8 {
            return Err(ApiError::BadRequest("密码至少 8 位".into()));
        }
        user.password_hash = auth::hash_password(&pwd)?;
    }

    sqlx::query(
        "UPDATE users SET email=?, display_name=?, role=?, status=?, password_hash=?, updated_at=? WHERE id=?",
    )
    .bind(&user.email)
    .bind(&user.display_name)
    .bind(&user.role)
    .bind(&user.status)
    .bind(&user.password_hash)
    .bind(now_iso())
    .bind(id)
    .execute(&state.db)
    .await?;

    find_by_id(state, id).await
}

/// 普通用户更新自己的资料(仅姓名与邮箱)
pub async fn update_self(
    state: &AppState,
    id: &str,
    display_name: Option<String>,
    email: Option<String>,
) -> ApiResult<User> {
    let mut user = find_by_id(state, id).await?;
    if let Some(email) = email {
        if !email.contains('@') {
            return Err(ApiError::BadRequest("邮箱格式不正确".into()));
        }
        user.email = email;
    }
    if let Some(dn) = display_name {
        user.display_name = dn;
    }
    sqlx::query("UPDATE users SET email=?, display_name=?, updated_at=? WHERE id=?")
        .bind(&user.email)
        .bind(&user.display_name)
        .bind(now_iso())
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref d) if d.is_unique_violation() => ApiError::Conflict("邮箱已被占用".into()),
            other => ApiError::Db(other),
        })?;
    find_by_id(state, id).await
}

pub async fn set_password(state: &AppState, id: &str, new_password: &str) -> ApiResult<()> {
    let hash = auth::hash_password(new_password)?;
    sqlx::query("UPDATE users SET password_hash=?, updated_at=? WHERE id=?")
        .bind(hash)
        .bind(now_iso())
        .bind(id)
        .execute(&state.db)
        .await?;
    Ok(())
}

/// 导出匹配关键词的全部用户(用于 CSV),上限保护
pub async fn export(state: &AppState, q: Option<String>) -> ApiResult<Vec<User>> {
    let like = format!("%{}%", q.unwrap_or_default());
    Ok(sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE username LIKE ? OR email LIKE ? OR display_name LIKE ?
         ORDER BY created_at DESC LIMIT 50000",
    )
    .bind(&like).bind(&like).bind(&like)
    .fetch_all(&state.db)
    .await?)
}

pub async fn delete(state: &AppState, id: &str) -> ApiResult<()> {
    let res = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await?;
    if res.rows_affected() == 0 {
        return Err(ApiError::NotFound("用户不存在".into()));
    }
    Ok(())
}

pub async fn touch_login(state: &AppState, id: &str) -> ApiResult<()> {
    sqlx::query("UPDATE users SET last_login_at = ? WHERE id = ?")
        .bind(now_iso())
        .bind(id)
        .execute(&state.db)
        .await?;
    Ok(())
}
