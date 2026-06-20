-- 0002_sessions.sql:为刷新令牌记录设备信息,支撑「登录会话」管理
ALTER TABLE refresh_tokens ADD COLUMN user_agent TEXT NOT NULL DEFAULT '';
ALTER TABLE refresh_tokens ADD COLUMN ip TEXT NOT NULL DEFAULT '';
