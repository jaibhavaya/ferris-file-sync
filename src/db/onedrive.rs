use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{query_as, PgPool};
use uuid::Uuid;

use crate::db::encryption::{decrypt_token, encrypt_token};
use crate::db::models::{OneDriveIntegration, OneDriveToken};

pub async fn get_integration(pool: &PgPool, owner_id: Uuid) -> Result<Option<OneDriveIntegration>> {
    let integration = query_as!(
        OneDriveIntegration,
        r#"
        SELECT id, owner_id, token_expires_at, drive_id, is_active, created_at, updated_at
        FROM onedrive_integrations
        WHERE owner_id = $1 AND is_active = true
        "#,
        owner_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(integration)
}

pub async fn get_token(
    pool: &PgPool,
    owner_id: Uuid,
    encryption_key: &str,
) -> Result<Option<OneDriveToken>> {
    // Fetch the encrypted token
    let record = sqlx::query!(
        r#"
        SELECT encrypted_token, token_expires_at
        FROM onedrive_integrations
        WHERE owner_id = $1 AND is_active = true
        "#,
        owner_id
    )
    .fetch_optional(pool)
    .await?;

    match record {
        Some(record) => {
            // Decrypt the token
            let token = decrypt_token(&record.encrypted_token, encryption_key)?;

            Ok(Some(OneDriveToken { token, expires_at: record.token_expires_at }))
        }
        None => Ok(None),
    }
}

pub async fn save_token(
    pool: &PgPool,
    owner_id: Uuid,
    token: &str,
    expires_at: DateTime<Utc>,
    drive_id: Option<String>,
    encryption_key: &str,
) -> Result<OneDriveIntegration> {
    // Encrypt the token
    let encrypted_token = encrypt_token(token, encryption_key)?;

    // Insert or update the integration
    let integration = sqlx::query_as!(
        OneDriveIntegration,
        r#"
        INSERT INTO onedrive_integrations
            (owner_id, encrypted_token, token_expires_at, drive_id, is_active)
        VALUES
            ($1, $2, $3, $4, true)
        ON CONFLICT (owner_id)
        DO UPDATE SET
            encrypted_token = $2,
            token_expires_at = $3,
            drive_id = $4,
            is_active = true,
            updated_at = NOW()
        RETURNING id, owner_id, token_expires_at, drive_id, is_active, created_at, updated_at
        "#,
        owner_id,
        encrypted_token,
        expires_at,
        drive_id
    )
    .fetch_one(pool)
    .await?;

    Ok(integration)
}

pub async fn deactivate_integration(pool: &PgPool, owner_id: Uuid) -> Result<bool> {
    let result = sqlx::query!(
        r#"
        UPDATE onedrive_integrations
        SET is_active = false, updated_at = NOW()
        WHERE owner_id = $1
        "#,
        owner_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

