pub struct Config {
    pub database_url: String,
    pub queue_url: String,
    pub aws_region: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let database_url = std::env::var("DATABASE_URL")?;
        let queue_url = std::env::var("QUEUE_URL")?;
        let aws_region = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());

        Ok(Config { database_url, queue_url, aws_region })
    }
}
