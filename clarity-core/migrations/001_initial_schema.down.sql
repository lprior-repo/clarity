-- Down migration for initial schema
-- This reverses the initial schema creation

DROP INDEX IF EXISTS idx_interviews_spec_name;
DROP INDEX IF EXISTS idx_users_email;
DROP INDEX IF EXISTS idx_beads_created_by;
DROP INDEX IF EXISTS idx_beads_priority;
DROP INDEX IF EXISTS idx_beads_type;
DROP INDEX IF EXISTS idx_beads_status;
DROP TABLE IF EXISTS specs;
DROP TABLE IF EXISTS interviews;
DROP TABLE IF EXISTS beads;
DROP TABLE IF EXISTS users;
