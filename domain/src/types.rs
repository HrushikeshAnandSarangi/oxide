use uuid::Uuid;

pub type ProjectId=Uuid;
pub type DeploymentId=Uuid;

#[derive(Debug,Clone)]
pub struct Subdomain(String);

impl Subdomain {
    pub fn new(value: String)->Result<Self,String>{
        if value.is_empty() || value.contains(' '){
            return Err("Invalid Subdomain".into());
        }

        Ok(Self(value))
    }

    pub fn value(&self)->&str{
        &self.0
    }
    
}
