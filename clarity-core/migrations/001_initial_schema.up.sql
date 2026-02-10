-- Initial schema for Clarity application (SQLite)
-- This migration creates the core tables for users, beads, interviews, and specs

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'user', -- 'admin' or 'user'
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Beads table
CREATE TABLE IF NOT EXISTS beads (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'open', -- 'open', 'in_progress', 'blocked', 'deferred', 'closed'
    priority INTEGER NOT NULL DEFAULT 2 CHECK (priority BETWEEN 1 AND 3),
    bead_type TEXT NOT NULL DEFAULT 'feature', -- 'feature', 'bugfix', 'refactor', 'test', 'docs'
    created_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Interviews table
CREATE TABLE IF NOT EXISTS interviews (
    id TEXT PRIMARY KEY,
    spec_name TEXT NOT NULL,
    questions TEXT NOT NULL DEFAULT '[]', -- JSON array
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Specs table
CREATE TABLE IF NOT EXISTS specs (
    id TEXT PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    description TEXT,
    schema TEXT NOT NULL DEFAULT '{}', -- JSON object
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS idx_beads_status ON beads(status);
CREATE INDEX IF NOT EXISTS idx_beads_type ON beads(bead_type);
CREATE INDEX IF NOT EXISTS idx_beads_priority ON beads(priority);
CREATE INDEX IF NOT EXISTS idx_beads_created_by ON beads(created_by);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_interviews_spec_name ON interviews(spec_name);
