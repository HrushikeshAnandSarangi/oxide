use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

pub async fn list_projects(
    State(state): State<AppState>,
) -> Result<Json<Vec<db::models::ProjectRow>>, StatusCode> {
    state
        .control_plane
        .state
        .projects
        .list_all()
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("Failed to list projects: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

pub async fn list_deployments(
    State(state): State<AppState>,
) -> Result<Json<Vec<db::models::DeploymentRow>>, StatusCode> {
    state
        .control_plane
        .state
        .deployments
        .list_all()
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("Failed to list deployments: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

pub async fn get_deployment(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<db::models::DeploymentRow>, StatusCode> {
    state
        .control_plane
        .state
        .deployments
        .find_by_id(&id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get deployment {}: {}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)
        .map(Json)
}

#[derive(serde::Serialize)]
pub struct DeploymentLogsResponse {
    pub status: String,
    pub build_log: Option<String>,
    pub container_log: Option<String>,
}

/// Build/image-build output (nix build + docker build, appended live as the
/// deployment progresses) plus, once a container exists, its runtime logs —
/// the full "how is this building/running" picture for one deployment.
pub async fn get_deployment_logs(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<DeploymentLogsResponse>, StatusCode> {
    let deployment = state
        .control_plane
        .state
        .deployments
        .find_by_id(&id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get deployment {}: {}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let build_log = state
        .control_plane
        .state
        .deployments
        .get_build_log(&id)
        .await
        .unwrap_or(None);

    let container_log = if let Some(container_id) = &deployment.container_id {
        state
            .control_plane
            .state
            .runtime
            .get_logs(container_id)
            .await
            .ok()
    } else {
        None
    };

    Ok(Json(DeploymentLogsResponse {
        status: deployment.status,
        build_log,
        container_log,
    }))
}

pub async fn get_project_logs(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<String, StatusCode> {
    let project = state
        .control_plane
        .state
        .projects
        .find_by_id(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let dep_id = project.active_deployment_id.ok_or(StatusCode::NOT_FOUND)?;
    let dep = state
        .control_plane
        .state
        .deployments
        .find_by_id(&dep_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let container_id = dep.container_id.ok_or(StatusCode::NOT_FOUND)?;

    state
        .control_plane
        .state
        .runtime
        .get_logs(&container_id)
        .await
        .map_err(|e: common::error::OxideError| {
            tracing::error!("Failed to get logs for {}: {}", container_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}
