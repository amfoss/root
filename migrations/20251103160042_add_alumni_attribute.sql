-- Add migration script here
ALTER TABLE Member
ADD COLUMN is_alumni BOOLEAN DEFAULT FALSE;

-- Creating a view 'active_members' to filter out alumni
CREATE VIEW active_members AS
SELECT * FROM Member
WHERE is_alumni = FALSE;