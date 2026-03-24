#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            port: std::env::var("SERVICE_PORT")
                .unwrap_or_else(|_| "8082".into())
                .parse()
                .expect("SERVICE_PORT must be a number"),
        }
    }
}
