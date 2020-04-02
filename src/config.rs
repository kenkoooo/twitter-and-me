use crate::Result;

use serde::Deserialize;
use std::fs::File;
use std::io::Read;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub key: String,
    pub secret: String,
}

pub fn read_json_file<T: serde::de::DeserializeOwned>(json_file: &str) -> Result<T> {
    let mut f = File::open(json_file)?;
    let mut buffer = String::new();
    f.read_to_string(&mut buffer)?;

    let config = serde_json::from_str(&buffer)?;
    Ok(config)
}

pub fn write_json_file<T: serde::Serialize>(obj: T, json_file: &str) -> Result<()> {
    log::info!("Writing to {} ...", json_file);
    let file = File::create(json_file)?;
    serde_json::to_writer(file, &obj)?;
    Ok(())
}
