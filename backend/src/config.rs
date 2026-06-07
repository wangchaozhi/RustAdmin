#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub frontend_origin: String,
    pub access_ttl_minutes: i64,
    pub refresh_ttl_days: i64,
}

impl Config {
    pub fn from_env() -> Self {
        let get = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.to_string());
        Self {
            port: get("PORT", "8080").parse().expect("PORT must be a number"),
            database_url: get("DATABASE_URL", "sqlite://admin.db"),
            jwt_secret: get("JWT_SECRET", "change-me-in-production"),
            frontend_origin: get("FRONTEND_ORIGIN", "http://localhost:5173"),
            access_ttl_minutes: get("ACCESS_TTL_MINUTES", "15").parse().unwrap_or(15),
            refresh_ttl_days: get("REFRESH_TTL_DAYS", "7").parse().unwrap_or(7),
        }
    }
}
