use crate::state::AppState;
use axum::Json;
use axum::extract::Path;
pub use axum::extract::State;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct DeploymentRequest {
    pub subdomain: String,
}

#[derive(Serialize)]
pub struct DeploymentResponse {
    pub message: String,
    pub deployment_id: String,
}

pub async fn deploy(
    State(state): State<AppState>,
    Json(payload): Json<DeploymentRequest>,
) -> Result<(axum::http::StatusCode, Json<DeploymentResponse>), axum::http::StatusCode> {
    let deployment_id = state
        .control_plane
        .deploy(payload.subdomain)
        .await
        .map_err(|e| {
            tracing::error!("Deployment triggering failed: {}", e);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((
        axum::http::StatusCode::ACCEPTED,
        Json(DeploymentResponse {
            message: "deployment queued".to_string(),
            deployment_id: deployment_id.to_string(),
        }),
    ))
}

/// Gracefully tears the deployment down (stop + remove its container,
/// remove its proxy route if it's the active one) and deletes the record.
pub async fn delete_deployment(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, StatusCode> {
    state
        .control_plane
        .delete_deployment(id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete deployment {}: {}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(StatusCode::NO_CONTENT)
}

/// One-command rollback to the project's previous built artifact.
pub async fn rollback(
    State(state): State<AppState>,
    Json(payload): Json<DeploymentRequest>,
) -> Result<(StatusCode, Json<DeploymentResponse>), (StatusCode, String)> {
    let deployment_id = state
        .control_plane
        .rollback(payload.subdomain)
        .await
        .map_err(|e| {
            tracing::error!("Rollback failed: {}", e);
            (StatusCode::BAD_REQUEST, e.to_string())
        })?;

    Ok((
        StatusCode::ACCEPTED,
        Json(DeploymentResponse {
            message: "rollback started".to_string(),
            deployment_id: deployment_id.to_string(),
        }),
    ))
}
