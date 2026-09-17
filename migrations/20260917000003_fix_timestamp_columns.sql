-- Every created_at column was declared TIMESTAMP (no timezone) but every
-- Rust call site writes/reads it as chrono::DateTime<Utc>, which sqlx maps
-- to TIMESTAMPTZ. That mismatch made every SELECT decoding one of these
-- columns fail at runtime (only surfaced once someone actually ran a full
-- create-project-then-deploy flow). Existing values were always written as
-- UTC (via Utc::now()), so reinterpreting them as UTC on conversion is
-- correct.
ALTER TABLE projects
  ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC';

ALTER TABLE deployments
  ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC';

ALTER TABLE project_env_vars
  ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC';
