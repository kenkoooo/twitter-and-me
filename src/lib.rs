pub mod client;
pub mod error;
pub mod io;
pub mod model;

pub type Result<T> = std::result::Result<T, error::Error>;
