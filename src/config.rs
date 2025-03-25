pub struct Config {
    pub database_url: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let database_url = std::env::var("DATABASE_URL")?;
        let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string()).parse()?;

        Ok(Config { database_url, port })
    }
}
