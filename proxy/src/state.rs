
use std::sync::Arc;

use dashmap::DashMap;

#[derive(Clone)]
pub struct ProxyState{
    routes:Arc<DashMap<String,u16>>,
}

impl ProxyState {
    pub fn new()->Self{
        Self{
            routes:Arc::new(DashMap::new()),
        }
    }

    pub fn add_route(&self,subdomain:String,port:u16){
        self.routes.insert(subdomain, port);
    }

    pub fn remove_route(&self,subdomain:&str){
        self.routes.remove(subdomain);
    }

    pub fn get_port(&self,subdomain:&str)->Option<u16>{
        self.routes.get(subdomain).map(|v|*v)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_then_get_route_roundtrips() {
        let state = ProxyState::new();
        state.add_route("demo1".to_string(), 4001);
        assert_eq!(state.get_port("demo1"), Some(4001));
    }

    #[test]
    fn unknown_subdomain_returns_none() {
        let state = ProxyState::new();
        assert_eq!(state.get_port("nope"), None);
    }

    #[test]
    fn remove_route_clears_it() {
        let state = ProxyState::new();
        state.add_route("demo2".to_string(), 4002);
        state.remove_route("demo2");
        assert_eq!(state.get_port("demo2"), None);
    }

    #[test]
    fn add_route_overwrites_existing_port() {
        let state = ProxyState::new();
        state.add_route("demo3".to_string(), 4003);
        state.add_route("demo3".to_string(), 4099);
        assert_eq!(state.get_port("demo3"), Some(4099));
    }
}


