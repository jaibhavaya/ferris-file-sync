use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use reqwest::Client;
use serde::Deserialize;
use sqlx::PgPool;

use crate::db;

// Microsoft Graph API configuration
const MICROSOFT_LOGIN_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: i64,
    refresh_token: Option<String>,
}

pub struct OneDriveClient {
    http_client: Client,
    pool: PgPool,
    encryption_key: String,
    client_id: String,
    client_secret: String,
}

impl OneDriveClient {
    pub fn new(
        pool: PgPool,
        encryption_key: String,
        client_id: String,
        client_secret: String,
    ) -> Self {
        let http_client = Client::new();

        Self { http_client, pool, encryption_key, client_id, client_secret }
    }

    /// Get a valid access token for an owner, refreshing if necessary
    pub async fn get_access_token(&self, owner_id: i64) -> Result<String> {
        // First try to get a cached, non-expired access token
        if let Some(token) =
            db::onedrive::get_access_token(&self.pool, owner_id, &self.encryption_key).await?
        {
            println!("Found existing access token valid until {}", token.expires_at);
            return Ok(token.access_token);
        }

        println!("No valid access token found, refreshing...");

        // No valid access token found, get the refresh token and use it to get a new access token
        let refresh_token = self.get_refresh_token(owner_id).await?;

        // Exchange refresh token for a new access token
        let token_response = self.refresh_access_token(&refresh_token).await?;

        // Calculate expiry time (subtract 5 minutes for safety margin)
        let expires_at =
            Utc::now() + Duration::seconds(token_response.expires_in) - Duration::minutes(5);

        println!("Obtained new access token valid until {}", expires_at);

        // Save the new access token
        db::onedrive::save_access_token(
            &self.pool,
            owner_id,
            &token_response.access_token,
            expires_at,
            &self.encryption_key,
        )
        .await?;

        // If we got a new refresh token, update it too
        if let Some(new_refresh_token) = token_response.refresh_token {
            println!("Received new refresh token, updating...");

            // Get the user_id from the existing integration
            let integration = db::onedrive::get_integration(&self.pool, owner_id)
                .await?
                .context("No OneDrive integration found for this owner")?;

            db::onedrive::save_refresh_token(
                &self.pool,
                owner_id,
                integration.user_id,
                &new_refresh_token,
                &self.encryption_key,
            )
            .await?;
        }

        Ok(token_response.access_token)
    }

    /// Get the refresh token for an owner
    async fn get_refresh_token(&self, owner_id: i64) -> Result<String> {
        let refresh_token =
            db::onedrive::get_refresh_token(&self.pool, owner_id, &self.encryption_key)
                .await?
                .context("No OneDrive refresh token found for this owner")?;

        Ok(refresh_token.refresh_token)
    }

    /// Exchange a refresh token for a new access token
    async fn refresh_access_token(&self, refresh_token: &str) -> Result<TokenResponse> {
        println!("Exchanging refresh token for access token...");

        let params = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
            ("scope", "Files.ReadWrite offline_access"),
        ];

        let response = self
            .http_client
            .post(MICROSOFT_LOGIN_URL)
            .form(&params)
            .send()
            .await
            .context("Failed to send refresh token request")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_else(|_| "No response body".into());
            return Err(anyhow::anyhow!("Token refresh failed: HTTP {}: {}", status, text));
        }

        let token_data =
            response.json::<TokenResponse>().await.context("Failed to parse token response")?;

        println!("Successfully refreshed access token");

        Ok(token_data)
    }
}

