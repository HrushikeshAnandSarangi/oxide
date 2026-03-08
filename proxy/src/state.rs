
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


