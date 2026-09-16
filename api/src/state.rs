use controller::main_controller::ControlPlane;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub control_plane: Arc<ControlPlane>,
}
