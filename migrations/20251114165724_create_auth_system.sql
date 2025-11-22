-- Create authentication system tables

-- Custom type for user roles
CREATE TYPE role_type AS ENUM ('Admin', 'Member', 'Bot');

-- Add role column to Member table
ALTER TABLE Member ADD COLUMN role role_type NOT NULL DEFAULT 'Member';

-- Make member fields nullable for GitHub-only registrations
ALTER TABLE Member ALTER COLUMN roll_no DROP NOT NULL;
ALTER TABLE Member ALTER COLUMN sex DROP NOT NULL;
ALTER TABLE Member ALTER COLUMN year DROP NOT NULL;
ALTER TABLE Member ALTER COLUMN hostel DROP NOT NULL;
ALTER TABLE Member ALTER COLUMN mac_address DROP NOT NULL;
ALTER TABLE Member ALTER COLUMN discord_id DROP NOT NULL;
ALTER TABLE Member ALTER COLUMN group_id DROP NOT NULL;

-- Add updated_at column to Member table
ALTER TABLE Member ADD COLUMN updated_at TIMESTAMP NOT NULL DEFAULT NOW();

-- Sessions table: stores session tokens for logged-in users
CREATE TABLE IF NOT EXISTS Sessions (
    session_id SERIAL PRIMARY KEY,
    member_id INTEGER NOT NULL REFERENCES Member(member_id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- API Keys table: stores hashed API keys for bot authentication
CREATE TABLE IF NOT EXISTS ApiKeys (
    api_key_id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    key_hash TEXT NOT NULL,
    created_by INTEGER REFERENCES Member(member_id) ON DELETE SET NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMP
);

-- Few critical indexes
CREATE INDEX idx_sessions_token_hash ON Sessions(token_hash);
CREATE INDEX idx_apikeys_key_hash ON ApiKeys(key_hash);

-- Trigger to update timestamp on Users table
CREATE TRIGGER update_users_timestamp
BEFORE UPDATE ON Member
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();
