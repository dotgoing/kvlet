use anyhow::{anyhow, Result};
use std::str::FromStr;
#[derive(Debug)]
pub enum Method {
    Get,
    Post,
}

impl FromStr for Method {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "get" => Ok(Method::Get),
            "post" => Ok(Method::Post),
            _ => Err(anyhow!(format!("method can only be get or post {}", s))),
        }
    }
}
