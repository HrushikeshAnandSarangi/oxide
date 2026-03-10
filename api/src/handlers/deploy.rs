use axum::{Json};
pub use axum::extract::State;
use serde::{Deserialize, Serialize};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct DeploymentRequest{
    pub subdomain:String,
}

#[derive(Serialize)]
pub struct DeploymentResponse {
    pub message: String,
    pub deployment_id: String,
}

pub async fn deploy(State(state):State<AppState>,Json(payload):Json<DeploymentRequest>) -> Result<(axum::http::StatusCode, Json<DeploymentResponse>), axum::http::StatusCode> {
    let deployment_id = state.control_plane.deploy(payload.subdomain).await.map_err(|e| {
        tracing::error!("Deployment triggering failed: {}", e);
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((
        axum::http::StatusCode::ACCEPTED,
        Json(DeploymentResponse {
            message: "deployment queued".to_string(),
            deployment_id: deployment_id.to_string(),
        })
    ))
} 
