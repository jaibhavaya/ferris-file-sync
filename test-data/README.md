# Test Data Directory

This directory is used for storing test files for local development and testing.

## Purpose

- Store sample files for uploading to mock S3
- Keep test artifacts separate from application code
- Avoid committing test data to git

## Usage

1. Create test files in this directory
2. Use them with the AWS CLI commands to upload to LocalStack S3
3. Process them with your application

Example:
```bash
# Create a test file
echo "This is a test file" > test-data/sample.txt

# Upload to mock S3
aws s3 cp test-data/sample.txt s3://ferris-file-sync-bucket/ --endpoint-url=http://localhost:4566 --region us-east-1
```

Note: All files in this directory except this README.md are gitignored.