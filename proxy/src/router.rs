use crate::state::ProxyState;

pub fn resolve_target(host:&str,state:&ProxyState)->Option<String>{
    let parts : Vec<&str>=host.split(".").collect();
    if parts.is_empty(){
        return None;
    }

    let subdomain=parts[0];

    let port=state.get_port(subdomain)?;

    Some(format!("127.0.0.1:{}",port))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_registered_subdomain_from_host_header() {
        let state = ProxyState::new();
        state.add_route("demo".to_string(), 5050);
        assert_eq!(resolve_target("demo.oxide.dev", &state), Some("127.0.0.1:5050".to_string()));
    }

    #[test]
    fn returns_none_for_unregistered_subdomain() {
        let state = ProxyState::new();
        assert_eq!(resolve_target("ghost.oxide.dev", &state), None);
    }

    #[test]
    fn returns_none_for_empty_host() {
        let state = ProxyState::new();
        assert_eq!(resolve_target("", &state), None);
    }
}
