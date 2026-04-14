-- Add member_roles table to support multiple roles per member
CREATE TABLE MemberRoles (
    discord_id VARCHAR(255) PRIMARY KEY, -- Use the Discord ID as the key
    roles TEXT[] NOT NULL                -- Store roles as a PostgreSQL Array
);