use crate::{
    error::{ApiError, ApiResult},
    models::{CreateRoleReq, Role, RoleOut, UpdateRoleReq},
    perms,
    state::AppState,
};

pub async fn list(state: &AppState) -> ApiResult<Vec<RoleOut>> {
    let rows = sqlx::query_as::<_, Role>("SELECT * FROM roles ORDER BY name")
        .fetch_all(&state.db)
        .await?;
    Ok(rows.into_iter().map(RoleOut::from).collect())
}

pub async fn permissions_of(state: &AppState, role: &str) -> ApiResult<Vec<String>> {
    let perms: Option<String> = sqlx::query_scalar("SELECT permissions FROM roles WHERE name = ?")
        .bind(role)
        .fetch_optional(&state.db)
        .await?;
    Ok(perms
        .map(|p| serde_json::from_str(&p).unwrap_or_default())
        .unwrap_or_default())
}

async fn find(state: &AppState, name: &str) -> ApiResult<Role> {
    sqlx::query_as::<_, Role>("SELECT * FROM roles WHERE name = ?")
        .bind(name)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("角色不存在".into()))
}

/// 校验权限点合法 + 去重,非法返回 BadRequest
fn clean_perms(perms_in: Vec<String>) -> ApiResult<Vec<String>> {
    let mut out: Vec<String> = Vec::new();
    for p in perms_in {
        if !perms::is_valid(&p) {
            return Err(ApiError::BadRequest(format!("未知权限点:{p}")));
        }
        if !out.contains(&p) {
            out.push(p);
        }
    }
    Ok(out)
}

fn valid_name(name: &str) -> bool {
    let len = name.chars().count();
    (2..=32).contains(&len)
        && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

pub async fn create(state: &AppState, req: CreateRoleReq) -> ApiResult<RoleOut> {
    let name = req.name.trim().to_string();
    if !valid_name(&name) {
        return Err(ApiError::BadRequest("角色标识需为 2-32 位小写字母、数字、下划线或连字符".into()));
    }
    let perms = clean_perms(req.permissions)?;
    let perms_json = serde_json::to_string(&perms).unwrap_or_else(|_| "[]".into());
    let description = req.description.unwrap_or_default();

    sqlx::query("INSERT INTO roles (name, description, permissions) VALUES (?, ?, ?)")
        .bind(&name)
        .bind(&description)
        .bind(&perms_json)
        .execute(&state.db)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref d) if d.is_unique_violation() => ApiError::Conflict("角色标识已存在".into()),
            other => ApiError::Db(other),
        })?;

    Ok(find(state, &name).await?.into())
}

pub async fn update(state: &AppState, name: &str, req: UpdateRoleReq) -> ApiResult<RoleOut> {
    let mut role = find(state, name).await?;
    if let Some(desc) = req.description {
        role.description = desc;
    }
    if let Some(perms_in) = req.permissions {
        let perms = clean_perms(perms_in)?;
        role.permissions = serde_json::to_string(&perms).unwrap_or_else(|_| "[]".into());
    }
    sqlx::query("UPDATE roles SET description = ?, permissions = ? WHERE name = ?")
        .bind(&role.description)
        .bind(&role.permissions)
        .bind(name)
        .execute(&state.db)
        .await?;
    Ok(find(state, name).await?.into())
}

pub async fn delete(state: &AppState, name: &str) -> ApiResult<()> {
    if perms::BUILTIN_ROLES.contains(&name) {
        return Err(ApiError::BadRequest("内置角色不可删除".into()));
    }
    find(state, name).await?; // 不存在则 404
    let in_use: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE role = ?")
        .bind(name)
        .fetch_one(&state.db)
        .await?;
    if in_use > 0 {
        return Err(ApiError::Conflict(format!("仍有 {in_use} 个用户使用该角色,无法删除")));
    }
    sqlx::query("DELETE FROM roles WHERE name = ?")
        .bind(name)
        .execute(&state.db)
        .await?;
    Ok(())
}
