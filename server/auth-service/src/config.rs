#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub jwt_secret: String,
    pub access_token_ttl_seconds: i64,
    pub user_service_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            port: std::env::var("SERVICE_PORT")
                .unwrap_or_else(|_| "8083".into())
                .parse()
                .expect("SERVICE_PORT must be a number"),
            jwt_secret: std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            access_token_ttl_seconds: std::env::var("ACCESS_TOKEN_TTL_SECONDS")
                .unwrap_or_else(|_| "900".into())
                .parse()
                .expect("ACCESS_TOKEN_TTL_SECONDS must be a number"),
            user_service_url: std::env::var("USER_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:8082".into()),
        }
    }
}
