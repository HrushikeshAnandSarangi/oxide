-- Explicit per-project opt-in: generate a Nix flake from a Dockerfile-only
-- repo at build time. Off by default — a deliberate choice by whoever
-- deploys the project, not automatic just because a Dockerfile exists.
ALTER TABLE projects
  ADD COLUMN auto_generate_flake BOOLEAN NOT NULL DEFAULT false;
