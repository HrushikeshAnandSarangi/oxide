use axum::{Router,routing::{get,post}};

use crate::state::AppState;
use crate::handlers::{deploy::deploy,health::health,project::create_project};

pub fn create_router(state:AppState)->Router{
    Router::new()
        .route("/health",get(health))
        .route("/deploy", get(deploy))
        .route("/project",post(create_project))
        .with_state(state)
}
