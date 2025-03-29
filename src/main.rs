mod config;
mod db;
mod messages;
mod onedrive;

use anyhow::{Context, Result};
use aws_config::BehaviorVersion;
use aws_sdk_sqs::{Client, Error as SqsError};
use chrono::Utc;
use std::{cmp::min, time::Duration};

use crate::messages::{parse_message, MessageType};

#[tokio::main]
async fn main() -> Result<(), SqsError> {
    dotenv::dotenv().ok();

    // Load configuration
    let config = config::Config::from_env().expect("Failed to load config");

    // Set up database connection
    let pool = db::connect(&config.database_url).await.expect("Failed to connect to database");

    // Run migrations
    db::run_migrations(&pool).await.expect("Failed to run migrations");

    // Load AWS credentials and create SQS client
    let mut aws_config_builder = aws_config::defaults(BehaviorVersion::latest())
        .region(aws_types::region::Region::new(config.aws_region.clone()));

    // Use local endpoint if specified (for development with LocalStack)
    if let Some(endpoint) = &config.s3_endpoint {
        aws_config_builder = aws_config_builder.endpoint_url(endpoint.clone());
    }

    let aws_config = aws_config_builder.load().await;
    let client = Client::new(&aws_config);

    println!("Ferris File Sync SQS Consumer starting...");
    println!("Listening for messages on queue: {}", config.queue_url);

    loop {
        let recv_message_output = client
            .receive_message()
            .queue_url(&config.queue_url)
            .wait_time_seconds(20) // Long polling
            .max_number_of_messages(10)
            .send()
            .await?;

        if let Some(messages) = recv_message_output.messages {
            for message in messages {
                println!("Processing message ID: {}", message.message_id().unwrap_or("unknown"));

                if let Some(body) = &message.body {
                    // Process the message
                    match process_message(body, &pool, &config).await {
                        Ok(_) => println!("Message processed successfully"),
                        Err(e) => println!("Error processing message: {}", e),
                    }
                }

                // Delete the message from the queue after processing
                if let Some(receipt_handle) = &message.receipt_handle {
                    client
                        .delete_message()
                        .queue_url(&config.queue_url)
                        .receipt_handle(receipt_handle)
                        .send()
                        .await?;

                    println!("Message deleted from queue");
                }
            }
        } else {
            println!("No messages received. Waiting...");
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}

/// Process an SQS message based on its type
async fn process_message(
    message_body: &str,
    pool: &sqlx::PgPool,
    config: &config::Config,
) -> Result<(), anyhow::Error> {
    // Parse the message
    let message = parse_message(message_body).context("Failed to parse message")?;

    // Initialize OneDrive client if needed
    let onedrive_client = onedrive::OneDriveClient::new(
        pool.clone(),
        config.encryption_key.clone(),
        config.onedrive_client_id.clone(),
        config.onedrive_client_secret.clone(),
    );

    // Handle different message types
    match message {
        MessageType::OneDriveAuthorization { payload } => {
            println!(
                "Handling OneDrive authorization for owner: {}, user: {}",
                payload.owner_id, payload.user_id
            );

            // First store the refresh token
            db::onedrive::save_refresh_token(
                pool,
                payload.owner_id,
                payload.user_id,
                &payload.refresh_token,
                &config.encryption_key,
            )
            .await
            .context("Failed to save OneDrive refresh token")?;

            println!("OneDrive refresh token saved for owner: {}", payload.owner_id);

            // Immediately validate the token by trying to get an access token
            match onedrive_client.get_access_token(payload.owner_id).await {
                Ok(access_token) => {
                    println!("Successfully validated refresh token and obtained access token");
                    println!("Access token: {}...", &access_token[0..min(20, access_token.len())]);
                    println!("OneDrive integration is now ready for use");
                }
                Err(e) => {
                    println!("Warning: Saved refresh token, but token validation failed: {}", e);
                    println!("The refresh token may be invalid or expired");
                    // We don't return an error here as we've saved the token, but log the warning
                }
            }
        }

        MessageType::FileSync { payload } => {
            println!("Handling file sync request for owner: {}", payload.owner_id);
            println!("  - Source: s3://{}/{}", payload.bucket, payload.key);
            println!("  - Destination: {}", payload.destination);

            // First, check if we have a valid integration with OneDrive by trying to get an access token
            match onedrive_client.get_access_token(payload.owner_id).await {
                Ok(access_token) => {
                    println!("Successfully obtained access token: {}...", &access_token[0..20]);
                    // TODO: Implement file sync logic once token refresh is working
                    // 1. Download the file from S3
                    // 2. Upload the file to OneDrive
                    // 3. Update the file status in the database
                }
                Err(e) => {
                    println!("Error getting access token: {}", e);
                    return Err(anyhow::anyhow!("Failed to get OneDrive access token: {}", e));
                }
            }
        }
    }

    Ok(())
}
