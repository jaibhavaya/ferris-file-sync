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

3. **Create SQS Queue**
   ```bash
   aws sqs create-queue --queue-name ferris-file-sync-queue --endpoint-url=http://localhost:4566 --region us-east-1
   ```

4. **Set up environment**
   Create a `.env` file in the project root:
   ```
   DATABASE_URL=postgres://postgres:postgres@localhost:5433/ferris_file_sync
   QUEUE_URL=http://localhost:4566/000000000000/ferris-file-sync-queue
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

7. **Test SQS integration**
   Send a test message to the queue:
   ```bash
   aws sqs send-message --queue-url http://localhost:4566/000000000000/ferris-file-sync-queue --message-body "Hello from SQS!" --endpoint-url=http://localhost:4566 --region us-east-1
   ```
   
   You should see the message being processed in your application logs.

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