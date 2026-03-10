use axum::{extract::{State, Path}, Json, http::StatusCode};
use crate::state::AppState;

pub async fn list_projects(State(state): State<AppState>) -> Result<Json<Vec<db::models::ProjectRow>>, StatusCode> {
    state.control_plane.state.projects.list_all().await.map(Json).map_err(|e| {
        tracing::error!("Failed to list projects: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

pub async fn list_deployments(State(state): State<AppState>) -> Result<Json<Vec<db::models::DeploymentRow>>, StatusCode> {
    state.control_plane.state.deployments.list_all().await.map(Json).map_err(|e| {
        tracing::error!("Failed to list deployments: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

pub async fn get_deployment(State(state): State<AppState>, Path(id): Path<uuid::Uuid>) -> Result<Json<db::models::DeploymentRow>, StatusCode> {
    state.control_plane.state.deployments.find_by_id(&id).await.map_err(|e| {
        tracing::error!("Failed to get deployment {}: {}", id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?.ok_or(StatusCode::NOT_FOUND).map(Json)
}

pub async fn get_project_logs(State(state): State<AppState>, Path(id): Path<uuid::Uuid>) -> Result<String, StatusCode> {
    let project = state.control_plane.state.projects.find_by_id(&id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.ok_or(StatusCode::NOT_FOUND)?;
    let dep_id = project.active_deployment_id.ok_or(StatusCode::NOT_FOUND)?;
    let dep = state.control_plane.state.deployments.find_by_id(&dep_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.ok_or(StatusCode::NOT_FOUND)?;
    let container_id = dep.container_id.ok_or(StatusCode::NOT_FOUND)?;

    state.control_plane.state.runtime.get_logs(&container_id).await.map_err(|e| {
        tracing::error!("Failed to get logs for {}: {}", container_id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}
