-- Live build/deploy output (nix build + docker image build), appended
-- line-by-line as a deployment progresses, so the dashboard can show "how
-- it's building" rather than only the final outcome.
ALTER TABLE deployments
  ADD COLUMN build_log TEXT;
