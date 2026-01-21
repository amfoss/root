ALTER TABLE statusbreaks ADD COLUMN member_id INT;

ALTER TABLE statusbreaks 
ADD CONSTRAINT fk_statusbreaks_member 
FOREIGN KEY (member_id) 
REFERENCES member(member_id) 
ON DELETE CASCADE;

-- Make year nullable since member-specific breaks don't need it
ALTER TABLE statusbreaks ALTER COLUMN year DROP NOT NULL;

-- Add index for member_id lookups
CREATE INDEX idx_statusbreaks_member_id ON statusbreaks(member_id);

