use axum::{Json};
pub use axum::extract::State;
use serde::Deserialize;
use crate::state::AppState;
#[derive(Deserialize)]
pub struct DeploymentRequest{
    pub subdomain:String,
    pub repo_url:String,
}

pub async fn deploy(State(state):State<AppState>,Json(payload):Json<DeploymentRequest>)->Result<Json<&'static str>,axum::http::StatusCode>{
    state.control_plane.deploy(payload.subdomain,payload.repo_url).await.map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json("deployment started"))
} 
