-- Leave table for tracking leaves
CREATE TABLE Leave (
        leave_id SERIAL PRIMARY KEY,
        discord_id VARCHAR(255) REFERENCES Member(discord_id) ON DELETE CASCADE,
        from_date DATE DEFAULT CURRENT_DATE,
        duration INT DEFAULT 1,
        reason TEXT NOT NULL,
        applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        approved_by VARCHAR(255) REFERENCES Member(discord_id),
        CHECK (approved_by IS NULL OR approved_by <> discord_id),
        UNIQUE (from_date, discord_id)
);
