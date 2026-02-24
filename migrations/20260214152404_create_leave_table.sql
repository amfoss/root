-- Leave table for tracking leaves
CREATE TABLE Leave (
        leave_id SERIAL PRIMARY KEY,
        discord_id VARCHAR(255) NOT NULL REFERENCES Member(discord_id) ON DELETE CASCADE,
        from_date DATE DEFAULT CURRENT_DATE NOT NULL,
        duration INT DEFAULT 1 NOT NULL,
        reason TEXT NOT NULL,
        applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
        approved_by VARCHAR(255) REFERENCES Member(discord_id) ON DELETE SET NULL,
        CHECK (approved_by IS NULL OR approved_by <> discord_id),
        CHECK (duration > 0),
        UNIQUE (from_date, discord_id)
);
