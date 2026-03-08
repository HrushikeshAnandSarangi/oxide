use std::sync::Arc;
use controller::main_controller::ControlPlane;


#[derive(Clone)]
pub struct AppState{
    pub control_plane:Arc<ControlPlane>,
}
