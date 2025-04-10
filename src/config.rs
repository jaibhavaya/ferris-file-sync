pub struct Config {
    pub database_url: String,
    pub queue_url: String,
    pub aws_region: String,
    pub s3_bucket: String,
    pub s3_endpoint: Option<String>,
    pub encryption_key: String,
    pub onedrive_client_id: String,
    pub onedrive_client_secret: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let database_url = std::env::var("DATABASE_URL")?;
        let queue_url = std::env::var("QUEUE_URL")?;
        let aws_region = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());
        let s3_bucket = std::env::var("S3_BUCKET")?;
        let s3_endpoint = std::env::var("S3_ENDPOINT").ok();
        let encryption_key = std::env::var("ENCRYPTION_KEY")
            .unwrap_or_else(|_| "default-dev-key-please-change-in-production".to_string());
        let onedrive_client_id =
            std::env::var("ONEDRIVE_CLIENT_ID").unwrap_or_else(|_| "your-client-id".to_string());
        let onedrive_client_secret = std::env::var("ONEDRIVE_CLIENT_SECRET")
            .unwrap_or_else(|_| "your-client-secret".to_string());

        Ok(Config {
            database_url,
            queue_url,
            aws_region,
            s3_bucket,
            s3_endpoint,
            encryption_key,
            onedrive_client_id,
            onedrive_client_secret,
        })
    }
}
