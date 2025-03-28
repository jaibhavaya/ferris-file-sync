mod config;
mod db;

use aws_config::BehaviorVersion;
use aws_sdk_sqs::{Client, Error};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();

    // Load configuration
    let config = config::Config::from_env().expect("Failed to load config");

    // Set up database connection
    let pool = db::connect(&config.database_url).await.expect("Failed to connect to database");

    // Run migrations
    db::run_migrations(&pool).await.expect("Failed to run migrations");

    // Load AWS credentials and create SQS client
    let aws_config = aws_config::defaults(BehaviorVersion::latest()).load().await;
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
                println!("Processing message: {:?}", message);

                if let Some(body) = &message.body {
                    println!("Message body: {}", body);

                    // TODO: Add your file sync logic here
                    // This is where you'd parse the message and perform actions
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
