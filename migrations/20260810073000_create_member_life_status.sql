CREATE TABLE IF NOT EXISTS MemberLifeStatus (
    member_id INT REFERENCES Member(member_id) ON DELETE CASCADE PRIMARY KEY,
    lives INT NOT NULL DEFAULT 3,
    recovery_streak INT NOT NULL DEFAULT 0,
    is_probation BOOLEAN NOT NULL DEFAULT FALSE,
    last_reset_month INT NOT NULL,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
