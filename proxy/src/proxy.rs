use std::sync::Arc;
use async_trait::async_trait;
use pingora_core::prelude::*;
use pingora_core::upstreams::peer::HttpPeer;
use pingora_proxy::{http_proxy_service, ProxyHttp, Session};
use crate::router::resolve_target;
use crate::state::ProxyState;

pub fn start_proxy(state: ProxyState) {
    let mut server = Server::new(None).unwrap();
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

    fn new_ctx(&self) -> Self::CTX {
        ()
    }

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
