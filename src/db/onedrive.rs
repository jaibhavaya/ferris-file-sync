use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{query_as, PgPool};

use crate::db::encryption::{decrypt_token, encrypt_token};
use crate::db::models::{OneDriveAccessToken, OneDriveIntegration, OneDriveRefreshToken};

pub async fn get_integration(pool: &PgPool, owner_id: i64) -> Result<Option<OneDriveIntegration>> {
    let integration = query_as!(
        OneDriveIntegration,
        r#"
        SELECT id, owner_id, user_id, access_token_expires_at, is_active, created_at, updated_at
        FROM onedrive_integrations
        WHERE owner_id = $1 AND is_active = true
        "#,
        owner_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(integration)
}

pub async fn get_refresh_token(
    pool: &PgPool,
    owner_id: i64,
    encryption_key: &str,
) -> Result<Option<OneDriveRefreshToken>> {
    // Fetch the encrypted refresh token
    let record = sqlx::query!(
        r#"
        SELECT encrypted_refresh_token
        FROM onedrive_integrations
        WHERE owner_id = $1 AND is_active = true
        "#,
        owner_id
    )
    .fetch_optional(pool)
    .await?;

    match record {
        Some(record) => {
            // Decrypt the refresh token
            let refresh_token = decrypt_token(&record.encrypted_refresh_token, encryption_key)?;

            Ok(Some(OneDriveRefreshToken { refresh_token }))
        }
        None => Ok(None),
    }
}

pub async fn get_access_token(
    pool: &PgPool,
    owner_id: i64,
    encryption_key: &str,
) -> Result<Option<OneDriveAccessToken>> {
    // Fetch the encrypted access token
    let record = sqlx::query!(
        r#"
        SELECT encrypted_access_token, access_token_expires_at
        FROM onedrive_integrations
        WHERE owner_id = $1
          AND is_active = true
          AND encrypted_access_token IS NOT NULL
          AND access_token_expires_at > NOW()
        "#,
        owner_id
    )
    .fetch_optional(pool)
    .await?;

    match record {
        Some(record) => {
            // Decrypt the access token
            let access_token =
                decrypt_token(&record.encrypted_access_token.unwrap_or_default(), encryption_key)?;

            Ok(Some(OneDriveAccessToken {
                access_token,
                expires_at: record.access_token_expires_at.unwrap(),
            }))
        }
        None => Ok(None),
    }
}

pub async fn save_refresh_token(
    pool: &PgPool,
    owner_id: i64,
    user_id: i64,
    refresh_token: &str,
    encryption_key: &str,
) -> Result<OneDriveIntegration> {
    // Encrypt the refresh token
    let encrypted_refresh_token = encrypt_token(refresh_token, encryption_key)?;

    // Insert or update the integration
    let integration = sqlx::query_as!(
        OneDriveIntegration,
        r#"
        INSERT INTO onedrive_integrations
            (owner_id, user_id, encrypted_refresh_token, is_active)
        VALUES
            ($1, $2, $3, true)
        ON CONFLICT (owner_id)
        DO UPDATE SET
            user_id = $2,
            encrypted_refresh_token = $3,
            is_active = true,
            updated_at = NOW()
        RETURNING id, owner_id, user_id, access_token_expires_at, is_active, created_at, updated_at
        "#,
        owner_id,
        user_id,
        encrypted_refresh_token,
    )
    .fetch_one(pool)
    .await?;

    Ok(integration)
}

pub async fn save_access_token(
    pool: &PgPool,
    owner_id: i64,
    access_token: &str,
    expires_at: DateTime<Utc>,
    encryption_key: &str,
) -> Result<OneDriveIntegration> {
    // Encrypt the access token
    let encrypted_access_token = encrypt_token(access_token, encryption_key)?;

    // Update the integration with the new access token
    let integration = sqlx::query_as!(
        OneDriveIntegration,
        r#"
        UPDATE onedrive_integrations
        SET
            encrypted_access_token = $2,
            access_token_expires_at = $3,
            updated_at = NOW()
        WHERE owner_id = $1 AND is_active = true
        RETURNING id, owner_id, user_id, access_token_expires_at, is_active, created_at, updated_at
        "#,
        owner_id,
        encrypted_access_token,
        expires_at,
    )
    .fetch_one(pool)
    .await?;

    Ok(integration)
}

pub async fn deactivate_integration(pool: &PgPool, owner_id: i64) -> Result<bool> {
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

