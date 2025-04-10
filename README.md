# Ferris File Sync

*When files need to be elsewhere...*

A Rust-powered entity that watches and waits. It moves what needs to be moved.
No traces. No failures. No questions.

## Local Development Setup

1. **Install Rust and Cargo** (if not already installed)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source "$HOME/.cargo/env"
   ```

2. **Start PostgreSQL and LocalStack**
   ```bash
   docker-compose up -d
   ```

3. **Create SQS Queue and S3 Bucket**
   ```bash
   # Create SQS queue
   aws sqs create-queue --queue-name ferris-file-sync-queue --endpoint-url=http://localhost:4566 --region us-east-1
   
   # Create S3 bucket
   aws s3 mb s3://ferris-file-sync-bucket --endpoint-url=http://localhost:4566 --region us-east-1
   ```

4. **Set up environment**
   Create a `.env` file in the project root:
   ```
   DATABASE_URL=postgres://postgres:postgres@localhost:5433/ferris_file_sync
   QUEUE_URL=http://localhost:4566/000000000000/ferris-file-sync-queue
   S3_BUCKET=ferris-file-sync-bucket
   S3_ENDPOINT=http://localhost:4566
   ENCRYPTION_KEY=your-secret-key-for-token-encryption
   ONEDRIVE_CLIENT_ID=your-microsoft-app-client-id
   ONEDRIVE_CLIENT_SECRET=your-microsoft-app-client-secret
   ```

5. **Handle initial database setup**

   There are two options to handle SQLx's compile-time database checks:

   **Option A**: Create the database schema first (recommended)
   ```bash
   # Install SQLx CLI
   cargo install sqlx-cli

   # Create database and run migrations
   sqlx database create --database-url postgres://postgres:postgres@localhost:5433/ferris_file_sync
   sqlx migrate run --database-url postgres://postgres:postgres@localhost:5433/ferris_file_sync
   ```

   **Option B**: Disable compile-time checks
   Add the `offline` feature to sqlx in Cargo.toml and use `query_as_unchecked!` instead of `query_as!` in the code.

6. **Build and run**
   ```bash
   cargo build
   cargo run
   ```

7. **Test SQS, S3, and OneDrive integration**
   
   **Upload a test file to S3:**
   ```bash
   # Create a test file in the test-data directory (which is gitignored)
   mkdir -p test-data
   echo "This is a test file for S3 to OneDrive sync" > test-data/test-file.txt
   
   # Upload the file to S3
   aws s3 cp test-data/test-file.txt s3://ferris-file-sync-bucket/ --endpoint-url=http://localhost:4566 --region us-east-1
   ```
   
   **List files in the bucket:**
   ```bash
   aws s3 ls s3://ferris-file-sync-bucket/ --endpoint-url=http://localhost:4566 --region us-east-1
   ```
   
   **Testing OneDrive Integration:**
   
   The integration with OneDrive requires two steps:
   
   1. **First, store a refresh token by sending an authorization message:**
   ```bash
   # OneDrive authorization message
   aws sqs send-message \
     --queue-url http://localhost:4566/000000000000/ferris-file-sync-queue \
     --message-body '{
       "event_type": "onedrive_authorization",
       "payload": {
         "refresh_token": "example_refresh_token",
         "owner_id": 123,
         "user_id": 456,
         "timestamp": "2025-03-29T12:00:00Z"
       }
     }' \
     --endpoint-url=http://localhost:4566 \
     --region us-east-1
   ```
   
   2. **Then test the token refresh by sending a file sync message:**
   ```bash
   # File sync message
   aws sqs send-message \
     --queue-url http://localhost:4566/000000000000/ferris-file-sync-queue \
     --message-body '{
       "event_type": "file_sync",
       "payload": {
         "bucket": "ferris-file-sync-bucket",
         "key": "test-file.txt",
         "destination": "/Documents/",
         "owner_id": 123,
         "user_id": 456,
         "timestamp": "2025-03-29T12:00:00Z"
       }
     }' \
     --endpoint-url=http://localhost:4566 \
     --region us-east-1
   ```
   
   To correctly test with Microsoft, you'll need to:
   
   1. Register an application in the [Azure Portal](https://portal.azure.com/#blade/Microsoft_AAD_RegisteredApps/ApplicationsListBlade)
   2. Create a client secret in the Azure Portal under "Certificates & secrets"
   3. Update both `ONEDRIVE_CLIENT_ID` and `ONEDRIVE_CLIENT_SECRET` in your `.env` file
   4. Obtain a real refresh token via OAuth flow (temporarily uncomment the get_access_token test in the OneDriveAuthorization handler)
   
   For local testing, you'll see error messages when trying to refresh the token since "example_refresh_token" is not valid.

8. **Add test data**
   Connect to the database:
   ```bash
   psql -h localhost -p 5433 -U postgres -d ferris_file_sync
   # Password: postgres
   ```

   Insert test data:
   ```sql
   INSERT INTO files (name)
   VALUES
     ('important_document.pdf'),
     ('quarterly_report.xlsx'),
     ('profile_picture.jpg');
   ```