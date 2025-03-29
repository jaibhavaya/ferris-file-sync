CREATE TABLE IF NOT EXISTS onedrive_integrations (
    id SERIAL PRIMARY KEY,
    owner_id UUID NOT NULL,
    encrypted_token TEXT NOT NULL,    -- Base64-encoded encrypted access token
    token_expires_at TIMESTAMPTZ NOT NULL, -- When the token expires
    drive_id TEXT,                    -- OneDrive drive ID (can be null)
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Unique index on owner_id to ensure one integration per owner
CREATE UNIQUE INDEX idx_onedrive_integrations_owner ON onedrive_integrations(owner_id);

-- Index for finding active integrations
CREATE INDEX idx_onedrive_integrations_active ON onedrive_integrations(is_active);

-- Create a function to automatically update the updated_at timestamp
CREATE OR REPLACE FUNCTION trigger_set_timestamp()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Add trigger to automatically update updated_at on row update
CREATE TRIGGER set_timestamp
BEFORE UPDATE ON onedrive_integrations
FOR EACH ROW
EXECUTE FUNCTION trigger_set_timestamp();