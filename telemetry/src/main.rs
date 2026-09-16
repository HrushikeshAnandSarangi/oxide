//! Telemetry ETL consumer.
//!
//! Extract: `controller`'s deployment_service/health_monitor XADD lifecycle
//! events onto the `oxide:events:deployments` Redis Stream (see
//! `common::events`).
//! Transform: this process reads them via a consumer group, parses the JSON
//! payload back into a `DeploymentEvent`.
//! Load: writes a row into Postgres' `deployment_events` table for
//! historical analytics, independent of the live Prometheus metrics.

use common::events::{DeploymentEvent, DEPLOYMENT_STREAM};
use db::telemetry_repo::TelemetryRepository;
use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::AsyncCommands;

const CONSUMER_GROUP: &str = "oxide-telemetry";
const CONSUMER_NAME: &str = "telemetry-consumer-1";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    common::logging::init();
    tracing::info!("Starting Oxide Telemetry ETL consumer...");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/oxide".to_string());
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    let pool = db::create_pool(&database_url).await?;
    let repo = TelemetryRepository::new(pool);

    let client = redis::Client::open(redis_url.as_str())?;
    let mut conn = client.get_multiplexed_async_connection().await?;

    // Idempotent: BUSYGROUP just means the group already exists from a
    // previous run, which is fine.
    let created: Result<(), redis::RedisError> = conn
        .xgroup_create_mkstream(DEPLOYMENT_STREAM, CONSUMER_GROUP, "$")
        .await;
    if let Err(e) = created {
        if !e.to_string().contains("BUSYGROUP") {
            return Err(e.into());
        }
    }

    let opts = StreamReadOptions::default()
        .group(CONSUMER_GROUP, CONSUMER_NAME)
        .count(10)
        .block(5000);

    loop {
        let reply: StreamReadReply = match conn
            .xread_options(&[DEPLOYMENT_STREAM], &[">"], &opts)
            .await
        {
            Ok(reply) => reply,
            Err(e) => {
                tracing::warn!("Error reading from Redis stream: {}. Retrying...", e);
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                continue;
            }
        };

        for stream_key in reply.keys {
            for stream_id in stream_key.ids {
                let payload: Option<String> = stream_id
                    .map
                    .get("payload")
                    .and_then(|v| redis::from_redis_value::<String>(v).ok());

                if let Some(payload) = payload {
                    match serde_json::from_str::<DeploymentEvent>(&payload) {
                        Ok(event) => {
                            if let Err(e) = repo
                                .insert_event(
                                    event.deployment_id,
                                    &event.subdomain,
                                    event.event_type.as_str(),
                                    event.status.as_deref(),
                                    event.duration_ms,
                                    event.occurred_at,
                                )
                                .await
                            {
                                tracing::error!("Failed to load event into Postgres: {}", e);
                            } else {
                                tracing::debug!(
                                    "Loaded {} event for deployment {}",
                                    event.event_type.as_str(),
                                    event.deployment_id
                                );
                            }
                        }
                        Err(e) => tracing::warn!("Skipping malformed telemetry event: {}", e),
                    }
                }

                let _: Result<i64, _> = conn
                    .xack(DEPLOYMENT_STREAM, CONSUMER_GROUP, &[stream_id.id.clone()])
                    .await;
            }
        }
    }
}
