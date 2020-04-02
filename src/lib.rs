pub mod client;
pub mod config;
pub mod error;
pub mod model;

pub type Result<T> = std::result::Result<T, error::Error>;
