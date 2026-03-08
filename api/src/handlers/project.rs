use axum::{extract::State,Json,http::StatusCode,};
use serde::{Deserialize,Serialize};

use crate::state::AppState;

#[derive(Deserialize)]
pub struct CreateProjectRequest{
    pub name:String,
    pub subdomain:String,
}

#[derive(Serialize)]
pub struct CreateProjectResponse{
    pub message:String,
}

pub async fn create_project(State(state):State<AppState>,Json(payload): Json<CreateProjectRequest>)->Result<Json<CreateProjectResponse>,StatusCode>{
    tracing::info!("Creating project: {} ({})",payload.name,payload.subdomain);
    Ok(Json(CreateProjectResponse{
        message:"Project created".to_string(),
    }))
}
