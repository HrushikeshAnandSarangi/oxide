-- Add Vercel-style deployment config columns to projects
ALTER TABLE projects 
  ADD COLUMN repo_url TEXT,
  ADD COLUMN install_command TEXT,
  ADD COLUMN build_command TEXT,
  ADD COLUMN run_command TEXT,
  ADD COLUMN root_directory TEXT DEFAULT '/';

-- Fix typo from previous migration (subdomaine -> subdomain)
-- NOTE: SQLite/Postgres sometimes don't like renaming columns if there's an index, but this is early dev.
-- We checked init.sql, the column was indeed `subdomain` but project_repo.rs had a typo in INSERT `subdomaine`.
-- So no DB level rename needed, just Rust code fix.

-- Create project environment variables table for AES-GCM encrypted secrets
CREATE TABLE project_env_vars (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    key TEXT NOT NULL,
    value_encrypted BYTEA NOT NULL,
    nonce BYTEA NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Index for fast lookup by project
CREATE INDEX idx_project_env_vars_project_id ON project_env_vars(project_id);
