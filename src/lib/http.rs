use anyhow::{anyhow, Result};
use std::fmt;
use std::str::FromStr;

#[derive(Debug)]
pub enum Method {
    Get,
    Post,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Method::Get => write!(f, "{}", "get"),
            Method::Post => write!(f, "{}", "post"),
        }
    }
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
