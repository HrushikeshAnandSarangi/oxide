use serde::Deserialize;

#[derive(Debug,Deserialize,Clone)]
pub struct ServerConfig{
    pub host: String,
    pub port: u16,
}

#[derive(Debug,Deserialize,Clone)]
pub struct DatabaseConfig{
    pub url:String,
}

#[derive(Debug,Deserialize,Clone)]
pub struct Settings{
    pub server: ServerConfig,
    pub database:DatabaseConfig,
}

pub fn load()->crate::Result<Settings>{
    let builder=config::Config::builder().add_source(config::File::with_name("config").required(false)).add_source(config::Environment::with_prefix("OXIDE"));

    builder.build().map_err(|e|crate::error::OxideError::Config(e.to_string()))?.try_deserialize().map_err(|e|crate::error::OxideError::Config(e.to_string()))
}
