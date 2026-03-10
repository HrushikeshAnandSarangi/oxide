use axum::{extract::State,Json,http::StatusCode,};
use serde::{Deserialize,Serialize};

use crate::state::AppState;

#[derive(Deserialize)]
pub struct CreateProjectRequest{
    pub name:String,
    pub subdomain:String,
    pub repo_url: Option<String>,
    pub install_command: Option<String>,
    pub build_command: Option<String>,
    pub run_command: Option<String>,
    pub root_directory: Option<String>,
    pub env_vars: Option<std::collections::HashMap<String, String>>,
}

#[derive(Serialize)]
pub struct CreateProjectResponse{
    pub message:String,
    pub project_id: String,
}

pub async fn create_project(State(state):State<AppState>,Json(payload): Json<CreateProjectRequest>)->Result<Json<CreateProjectResponse>,StatusCode>{
    tracing::info!("Creating project: {} ({}) with repo: {:?}",payload.name,payload.subdomain, payload.repo_url);
    
    // We construct the project domain object
    let subdomain = domain::types::Subdomain::new(payload.subdomain).map_err(|_| StatusCode::BAD_REQUEST)?;
    let project = domain::Project::new(
        payload.name, 
        subdomain,
        payload.repo_url,
        payload.install_command,
        payload.build_command,
        payload.run_command,
        payload.root_directory
    );

    state.control_plane.create_project(&project, payload.env_vars).await.map_err(|e| {
        tracing::error!("Failed to save project: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(CreateProjectResponse{
        message: "Project created".to_string(),
        project_id: project.id.to_string(),
    }))
}
