-- Add migration script here
ALTER table statusupdatehistory
RENAME COLUMN is_updated TO is_sent;
