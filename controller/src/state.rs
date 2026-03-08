use std::sync::Arc;

use builder::Builder;
use db::deployment_repo::DeploymentRepository;
use proxy::state::ProxyState;
use runtime::runtime::Runtime;



pub struct ControlState{
    pub builder:Arc<Builder>,
    pub runtime:Arc<Runtime>,
    pub proxy:Arc<ProxyState>,
    pub deployments:Arc<DeploymentRepository>,
}

