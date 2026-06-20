use uuid::Uuid;

use crate::{
    error::{ApiError, ApiResult},
    models::{now_iso, Announcement, AnnouncementFeedRow, CreateAnnouncementReq, UpdateAnnouncementReq},
    state::AppState,
};

const LEVELS: [&str; 3] = ["info", "warning", "critical"];

fn check_level(level: &str) -> ApiResult<()> {
    if LEVELS.contains(&level) {
        Ok(())
    } else {
        Err(ApiError::BadRequest("级别只能是 info / warning / critical".into()))
    }
}

/// 普通用户视图:仅已发布公告,附带当前用户已读状态
pub async fn feed(state: &AppState, user_id: &str) -> ApiResult<Vec<AnnouncementFeedRow>> {
    Ok(sqlx::query_as::<_, AnnouncementFeedRow>(
        "SELECT a.id, a.title, a.body, a.level, a.published, a.created_by, a.created_at, a.updated_at,
                CASE WHEN r.user_id IS NOT NULL THEN 1 ELSE 0 END AS read
         FROM announcements a
         LEFT JOIN announcement_reads r ON r.announcement_id = a.id AND r.user_id = ?
         WHERE a.published = 1
         ORDER BY a.created_at DESC",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?)
}

pub async fn mark_read(state: &AppState, ann_id: &str, user_id: &str) -> ApiResult<()> {
    sqlx::query(
        "INSERT OR IGNORE INTO announcement_reads (announcement_id, user_id, read_at) VALUES (?, ?, ?)",
    )
    .bind(ann_id)
    .bind(user_id)
    .bind(now_iso())
    .execute(&state.db)
    .await?;
    Ok(())
}

/// 管理视图:全部公告(含草稿)
pub async fn list_all(state: &AppState) -> ApiResult<Vec<Announcement>> {
    Ok(sqlx::query_as::<_, Announcement>(
        "SELECT * FROM announcements ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await?)
}

async fn find(state: &AppState, id: &str) -> ApiResult<Announcement> {
    sqlx::query_as::<_, Announcement>("SELECT * FROM announcements WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("公告不存在".into()))
}

pub async fn create(state: &AppState, req: CreateAnnouncementReq, author: &str) -> ApiResult<Announcement> {
    let title = req.title.trim().to_string();
    if title.is_empty() {
        return Err(ApiError::BadRequest("标题不能为空".into()));
    }
    let level = req.level.unwrap_or_else(|| "info".into());
    check_level(&level)?;

    let id = Uuid::new_v4().to_string();
    let now = now_iso();
    sqlx::query(
        "INSERT INTO announcements (id, title, body, level, published, created_by, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&title)
    .bind(req.body.unwrap_or_default())
    .bind(&level)
    .bind(req.published.unwrap_or(true))
    .bind(author)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await?;
    find(state, &id).await
}

pub async fn update(state: &AppState, id: &str, req: UpdateAnnouncementReq) -> ApiResult<Announcement> {
    let mut a = find(state, id).await?;
    if let Some(title) = req.title {
        if title.trim().is_empty() {
            return Err(ApiError::BadRequest("标题不能为空".into()));
        }
        a.title = title.trim().to_string();
    }
    if let Some(body) = req.body {
        a.body = body;
    }
    if let Some(level) = req.level {
        check_level(&level)?;
        a.level = level;
    }
    if let Some(published) = req.published {
        a.published = published;
    }
    sqlx::query(
        "UPDATE announcements SET title=?, body=?, level=?, published=?, updated_at=? WHERE id=?",
    )
    .bind(&a.title)
    .bind(&a.body)
    .bind(&a.level)
    .bind(a.published)
    .bind(now_iso())
    .bind(id)
    .execute(&state.db)
    .await?;
    find(state, id).await
}

pub async fn delete(state: &AppState, id: &str) -> ApiResult<()> {
    let res = sqlx::query("DELETE FROM announcements WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await?;
    if res.rows_affected() == 0 {
        return Err(ApiError::NotFound("公告不存在".into()));
    }
    Ok(())
}
