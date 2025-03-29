use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub id: i32,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OneDriveIntegration {
    pub id: i32,
    pub owner_id: i64,
    pub user_id: i64,
    // Note: encrypted tokens are managed internally and not exposed directly
    pub access_token_expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Represents a refresh token
#[derive(Debug, Serialize, Deserialize)]
pub struct OneDriveRefreshToken {
    pub refresh_token: String,
}

// Represents an access token with expiry
#[derive(Debug, Serialize, Deserialize)]
pub struct OneDriveAccessToken {
    pub access_token: String,
    pub expires_at: DateTime<Utc>,
}
