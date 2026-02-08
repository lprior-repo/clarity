-- Initial schema for Clarity application
-- This migration creates the core tables for users, beads, interviews, and specs

-- Create custom types
CREATE TYPE user_role AS ENUM ('admin', 'user');
CREATE TYPE bead_status AS ENUM ('open', 'in_progress', 'blocked', 'deferred', 'closed');
CREATE TYPE bead_type AS ENUM ('feature', 'bugfix', 'refactor', 'test', 'docs');

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role user_role NOT NULL DEFAULT 'user',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Beads table
CREATE TABLE beads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(500) NOT NULL,
    description TEXT,
    status bead_status NOT NULL DEFAULT 'open',
    priority INTEGER NOT NULL DEFAULT 2 CHECK (priority BETWEEN 1 AND 3),
    bead_type bead_type NOT NULL DEFAULT 'feature',
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Interviews table
CREATE TABLE interviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    spec_name VARCHAR(255) NOT NULL,
    questions JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Specs table
CREATE TABLE specs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) UNIQUE NOT NULL,
    description TEXT,
    schema JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for common queries
CREATE INDEX idx_beads_status ON beads(status);
CREATE INDEX idx_beads_type ON beads(bead_type);
CREATE INDEX idx_beads_priority ON beads(priority);
CREATE INDEX idx_beads_created_by ON beads(created_by);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_interviews_spec_name ON interviews(spec_name);
