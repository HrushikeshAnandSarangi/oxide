pub mod config;
pub mod error;
pub mod logging;

pub type Result<T>=std::result::Result<T,error::OxideError>;


