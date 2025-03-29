use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub id: i32,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OneDriveIntegration {
    pub id: i32,
    pub owner_id: Uuid,
    // Note: encrypted_token is managed internally and not exposed directly
    pub token_expires_at: DateTime<Utc>,
    pub drive_id: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// This is what we'll use in our application code
#[derive(Debug, Serialize, Deserialize)]
pub struct OneDriveToken {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}
