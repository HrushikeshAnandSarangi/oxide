use chrono::{DateTime, Utc};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::OxideError;

pub const DEPLOYMENT_STREAM: &str = "oxide:events:deployments";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    DeploymentStarted,
    BuildStarted,
    BuildCompleted,
    BuildFailed,
    ContainerStarted,
    RouteActivated,
    DeploymentCompleted,
    HealthCheckFailed,
    DeploymentCrashed,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::DeploymentStarted => "DeploymentStarted",
            EventType::BuildStarted => "BuildStarted",
            EventType::BuildCompleted => "BuildCompleted",
            EventType::BuildFailed => "BuildFailed",
            EventType::ContainerStarted => "ContainerStarted",
            EventType::RouteActivated => "RouteActivated",
            EventType::DeploymentCompleted => "DeploymentCompleted",
            EventType::HealthCheckFailed => "HealthCheckFailed",
            EventType::DeploymentCrashed => "DeploymentCrashed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentEvent {
    pub deployment_id: Uuid,
    pub subdomain: String,
    pub event_type: EventType,
    pub status: Option<String>,
    pub duration_ms: Option<i64>,
    pub occurred_at: DateTime<Utc>,
}

impl DeploymentEvent {
    pub fn new(deployment_id: Uuid, subdomain: impl Into<String>, event_type: EventType) -> Self {
        Self {
            deployment_id,
            subdomain: subdomain.into(),
            event_type,
            status: None,
            duration_ms: None,
            occurred_at: Utc::now(),
        }
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn with_duration_ms(mut self, duration_ms: i64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }
}

/// Publishes deployment lifecycle events onto a Redis Stream so a separate
/// consumer (the `telemetry` crate) can transform and load them into
/// Postgres for historical analytics, independent of the live Prometheus
/// metrics scraped from `controller::metrics`.
#[derive(Clone)]
pub struct EventPublisher {
    conn: redis::aio::ConnectionManager,
}

impl EventPublisher {
    pub async fn connect(redis_url: &str) -> Result<Self, OxideError> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| OxideError::Internal(format!("invalid redis url: {e}")))?;
        let conn = client
            .get_connection_manager()
            .await
            .map_err(|e| OxideError::Internal(format!("failed to connect to redis: {e}")))?;
        Ok(Self { conn })
    }

    pub async fn publish(&self, event: &DeploymentEvent) -> Result<(), OxideError> {
        let payload = serde_json::to_string(event)
            .map_err(|e| OxideError::Internal(format!("failed to serialize event: {e}")))?;

        let mut conn = self.conn.clone();
        let _: String = conn
            .xadd(DEPLOYMENT_STREAM, "*", &[("payload", payload.as_str())])
            .await
            .map_err(|e| OxideError::Internal(format!("failed to publish event: {e}")))?;
        Ok(())
    }
}
