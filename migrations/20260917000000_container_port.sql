-- get_running_deployments() and the health monitor both need to know which
-- port a running deployment's container is listening on, but no column ever
-- stored it. Add it so proxy-route recovery on restart actually works.
ALTER TABLE deployments
  ADD COLUMN container_port INTEGER;
