use serde::Serialize;

/// 系统内可分配的权限点定义。角色管理页据此渲染勾选项,后端据此校验合法性。
pub struct PermDef {
    pub key: &'static str,
    pub label: &'static str,
    pub group: &'static str,
}

pub const CATALOG: &[PermDef] = &[
    PermDef { key: "dashboard.read",     label: "查看概览",            group: "概览" },
    PermDef { key: "users.read",         label: "查看用户",            group: "用户" },
    PermDef { key: "users.write",        label: "管理用户(增删改)",   group: "用户" },
    PermDef { key: "roles.read",         label: "查看角色",            group: "角色" },
    PermDef { key: "roles.write",        label: "管理角色与权限",      group: "角色" },
    PermDef { key: "audit.read",         label: "查看审计日志",        group: "审计" },
    PermDef { key: "announcements.write", label: "发布与管理公告",     group: "公告" },
];

/// 内置角色,不可删除;`admin` 额外锁定不可编辑以避免权限自锁。
pub const BUILTIN_ROLES: &[&str] = &["admin", "editor", "viewer"];

pub fn is_valid(key: &str) -> bool {
    CATALOG.iter().any(|p| p.key == key)
}

#[derive(Serialize)]
pub struct PermItem {
    pub key: String,
    pub label: String,
    pub group: String,
}

pub fn catalog() -> Vec<PermItem> {
    CATALOG
        .iter()
        .map(|p| PermItem { key: p.key.into(), label: p.label.into(), group: p.group.into() })
        .collect()
}
