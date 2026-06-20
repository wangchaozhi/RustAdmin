-- 0003_announcements.sql:系统公告与按用户的已读状态
CREATE TABLE IF NOT EXISTS announcements (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    body        TEXT NOT NULL DEFAULT '',
    level       TEXT NOT NULL DEFAULT 'info',   -- info | warning | critical
    published   INTEGER NOT NULL DEFAULT 1,     -- 0 草稿 / 1 已发布
    created_by  TEXT NOT NULL DEFAULT '',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_announce_created ON announcements(created_at DESC);

CREATE TABLE IF NOT EXISTS announcement_reads (
    announcement_id TEXT NOT NULL REFERENCES announcements(id) ON DELETE CASCADE,
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    read_at         TEXT NOT NULL,
    PRIMARY KEY (announcement_id, user_id)
);

-- 让内置 admin 角色获得新增的公告管理权限
UPDATE roles
SET permissions = '["dashboard.read","users.read","users.write","roles.read","roles.write","audit.read","announcements.write"]'
WHERE name = 'admin';
