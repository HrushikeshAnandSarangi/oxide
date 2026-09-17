use crate::router::resolve_target;
use crate::state::ProxyState;
use async_trait::async_trait;
use pingora_core::prelude::*;
use pingora_core::server::configuration::ServerConf;
use pingora_core::upstreams::peer::HttpPeer;
use pingora_proxy::{ProxyHttp, Session, http_proxy_service};
use std::sync::Arc;

pub fn start_proxy(state: ProxyState) {
    // Pingora's own default grace period on SIGTERM is 300s — it always
    // sleeps for the full configured duration on shutdown, regardless of
    // whether connections have already drained (see pingora-core's
    // server/mod.rs). Left at the default, `docker stop`/systemd's usual
    // ~10s timeout would always end in a SIGKILL instead of ever finishing
    // gracefully. 10s keeps a real (if short) drain window while still
    // fitting inside typical orchestrator shutdown timeouts.
    let conf = ServerConf {
        grace_period_seconds: Some(10),
        graceful_shutdown_timeout_seconds: Some(5),
        ..Default::default()
    };
    let mut server = Server::new_with_opt_and_conf(None, conf);
    server.bootstrap();

    let state = Arc::new(state);
    let mut proxy = http_proxy_service(&server.configuration, OxideProxy { state });
    proxy.add_tcp("0.0.0.0:8000");
    server.add_service(proxy);
    server.run_forever();
}

struct OxideProxy {
    state: Arc<ProxyState>,
}

#[async_trait]
impl ProxyHttp for OxideProxy {
    type CTX = ();

    fn new_ctx(&self) -> Self::CTX {}

    async fn upstream_peer(
        &self,
        session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let host = session
            .req_header()
            .headers
            .get("host")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let target = resolve_target(host, &self.state)
            .ok_or_else(|| Error::new(ErrorType::HTTPStatus(404)))?;

        let peer = HttpPeer::new(target, false, host.to_string());
        Ok(Box::new(peer))
    }
}
