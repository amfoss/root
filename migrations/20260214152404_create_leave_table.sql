-- Leave table for tracking leaves
CREATE TABLE Leave (
        leave_id SERIAL PRIMARY KEY,
        discord_id VARCHAR(255) REFERENCES Member(discord_id) ON DELETE CASCADE,
        date DATE DEFAULT CURRENT_DATE,
        duration INT DEFAULt 1,
        reason TEXT NOT NULL,
        approved_by VARCHAR(255) REFERENCES Member(discord_id),
        CHECK (approved_by IS NULL OR approved_by <> discord_id),
        UNIQUE (date, discord_id)
);
