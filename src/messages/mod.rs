use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Base message structure that all message types use
#[derive(Debug, Serialize, Deserialize)]
pub struct Message<T> {
    pub event_type: String,
    pub payload: T,
}

/// OneDrive authorization event
#[derive(Debug, Serialize, Deserialize)]
pub struct OneDriveAuthorizationPayload {
    pub refresh_token: String,
    pub owner_id: i64,
    pub user_id: i64,
    pub timestamp: DateTime<Utc>,
}

/// File sync request event
#[derive(Debug, Serialize, Deserialize)]
pub struct FileSyncPayload {
    pub bucket: String,
    pub key: String,
    pub destination: String,
    pub owner_id: i64,
    pub user_id: Option<i64>,
    pub timestamp: DateTime<Utc>,
}

/// Message type enumeration for easy pattern matching
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum MessageType {
    #[serde(rename = "onedrive_authorization")]
    OneDriveAuthorization { payload: OneDriveAuthorizationPayload },

    #[serde(rename = "file_sync")]
    FileSync { payload: FileSyncPayload },
}

/// Parse a raw message string into a typed message
pub fn parse_message(message_str: &str) -> Result<MessageType, serde_json::Error> {
    serde_json::from_str(message_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_onedrive_auth_message() {
        let message_str = r#"
        {
            "event_type": "onedrive_authorization",
            "payload": {
                "refresh_token": "M.R3_BAY...",
                "owner_id": 123,
                "user_id": 456,
                "timestamp": "2025-03-24T13:05:23Z"
            }
        }
        "#;

        let message = parse_message(message_str).unwrap();
        match message {
            MessageType::OneDriveAuthorization { payload } => {
                assert_eq!(payload.user_id, 456);
                assert!(payload.refresh_token.starts_with("M.R3_BAY"));
            }
            _ => panic!("Expected OneDriveAuthorization message"),
        }
    }

    #[test]
    fn test_parse_file_sync_message() {
        let message_str = r#"
        {
            "event_type": "file_sync",
            "payload": {
                "bucket": "ferris-file-sync-bucket",
                "key": "test-file.txt",
                "destination": "/Documents/",
                "owner_id": 123,
                "user_id": 789,
                "timestamp": "2025-03-24T13:10:23Z"
            }
        }
        "#;

        let message = parse_message(message_str).unwrap();
        match message {
            MessageType::FileSync { payload } => {
                assert_eq!(payload.bucket, "ferris-file-sync-bucket");
                assert_eq!(payload.key, "test-file.txt");
            }
            _ => panic!("Expected FileSync message"),
        }
    }
}
