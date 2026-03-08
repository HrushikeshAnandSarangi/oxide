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
