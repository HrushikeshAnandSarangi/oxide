use axum::{Router,routing::{get,post}};

use crate::state::AppState;
use crate::handlers::{deploy::deploy,health::health,project::create_project, history::{list_projects, list_deployments, get_deployment, get_project_logs}};

pub fn create_router(state:AppState)->Router{
    Router::new()
        .route("/health",get(health))
        .route("/deploy", post(deploy))
        .route("/project",post(create_project))
        .route("/projects",get(list_projects))
        .route("/projects/:id/logs",get(get_project_logs))
        .route("/deployments",get(list_deployments))
        .route("/deployments/:id",get(get_deployment))
        .with_state(state)
}
