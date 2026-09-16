use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result};
use uuid::Uuid;

pub struct TelemetryRepository {
    pool: PgPool,
}

impl TelemetryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_event(
        &self,
        deployment_id: Uuid,
        subdomain: &str,
        event_type: &str,
        status: Option<&str>,
        duration_ms: Option<i64>,
        occurred_at: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO deployment_events(id, deployment_id, subdomain, event_type, status, duration_ms, occurred_at)
            VALUES($1,$2,$3,$4,$5,$6,$7)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(deployment_id)
        .bind(subdomain)
        .bind(event_type)
        .bind(status)
        .bind(duration_ms)
        .bind(occurred_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn average_build_duration_ms(&self) -> Result<Option<f64>, sqlx::Error> {
        use sqlx::Row;
        let row = sqlx::query(
            r#"
            SELECT AVG(duration_ms)::float8 AS avg_ms
            FROM deployment_events
            WHERE event_type = 'BuildCompleted' AND duration_ms IS NOT NULL
            "#,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row.try_get::<Option<f64>, _>("avg_ms").unwrap_or(None))
    }
}
