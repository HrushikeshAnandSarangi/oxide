-- Historical event log fed by the telemetry crate's Redis Streams consumer.
-- Kept as a flat append-only log rather than pre-aggregated so it supports
-- arbitrary analytical queries later without committing to a rollup shape.
CREATE TABLE deployment_events (
    id UUID PRIMARY KEY,
    deployment_id UUID NOT NULL,
    subdomain TEXT NOT NULL,
    event_type TEXT NOT NULL,
    status TEXT,
    duration_ms BIGINT,
    occurred_at TIMESTAMPTZ NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_deployment_events_deployment_id ON deployment_events(deployment_id);
CREATE INDEX idx_deployment_events_event_type ON deployment_events(event_type);
CREATE INDEX idx_deployment_events_occurred_at ON deployment_events(occurred_at);
