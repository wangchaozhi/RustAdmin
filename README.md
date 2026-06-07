# Harbor 管理控制台

Rust + React 全栈后台管理系统,桌面端与移动端自适应。

## 功能

- JWT 双令牌认证(Access 15 分钟 + Refresh 7 天,刷新令牌哈希存储并轮换)
- RBAC 权限模型:admin / editor / viewer 三角色,接口级权限校验
- 用户管理:搜索、分页、新建、编辑、禁用、删除(含自我保护规则)
- 审计日志:登录与所有写操作自动记录,含 IP
- 仪表盘:用户统计、近 7 天登录趋势、最近操作流
- 响应式 UI:≥1024px 侧边栏布局,移动端顶栏 + 底部 Tab,表格自动切换为卡片

## 架构

```
backend/                      Rust (Axum 0.7 + SQLx + SQLite)
├── migrations/               SQL 迁移,启动时自动执行
└── src/
    ├── main.rs               启动:日志、连接池、迁移、CORS、路由
    ├── config.rs             环境变量配置
    ├── error.rs              统一 ApiError → HTTP 响应
    ├── state.rs              AppState(连接池 + 配置)
    ├── auth.rs               Argon2 哈希、JWT 签发/校验、鉴权中间件、权限检查
    ├── models.rs             实体与请求/响应 DTO
    ├── repo/                 数据访问层(users / roles / tokens / audit)
    └── routes/               HTTP 处理器(auth / users / misc)

frontend/                     React 18 + TypeScript + Vite
└── src/
    ├── api/client.ts         fetch 封装:令牌注入、401 自动刷新重试
    ├── store/auth.tsx        认证 Context:登录、登出、会话恢复、权限判断
    ├── layouts/AdminLayout   响应式骨架(侧边栏 / 底部 Tab)
    ├── pages/                Login · Dashboard · Users · Roles · Audit
    └── styles.css            设计令牌 + 响应式样式
```

## 本地运行

后端(需要 Rust 1.75+):

```bash
cd backend
cp .env.example .env        # 修改 JWT_SECRET!
cargo run                   # http://localhost:8080
```

前端(需要 Node 18+):

```bash
cd frontend
npm install
npm run dev                 # http://localhost:5173,/api 已代理到 8080
```

首次启动自动创建管理员 `admin / Admin@12345`,登录后请立即修改密码。

## Docker

```bash
docker compose up --build
# 前端 http://localhost:5173,后端 http://localhost:8080
```

## 切换 PostgreSQL

1. `Cargo.toml` 中将 sqlx 的 `sqlite` 特性换成 `postgres`
2. `main.rs` 中改用 `PgPoolOptions`,`AppState` 换成 `PgPool`
3. 迁移脚本里 SQLite 的日期函数(`date('now')` 等)换成 Postgres 等价写法
4. `DATABASE_URL=postgres://user:pass@host/db`

仓储层 SQL 均为参数化查询,绝大部分无需修改。

## 生产部署清单

- [ ] 更换 `JWT_SECRET` 为 ≥32 字节随机串
- [ ] 前端 `npm run build` 后由 Nginx/Caddy 托管,反代 `/api`
- [ ] 全站 HTTPS;`FRONTEND_ORIGIN` 改为正式域名
- [ ] 登录接口加限流(如 tower-governor)
- [ ] 数据库定期备份;SQLite 建议开启 WAL
