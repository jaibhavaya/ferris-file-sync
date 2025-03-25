use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub id: i32,
    pub name: String,
    pub created_at: DateTime<Utc>,
}
