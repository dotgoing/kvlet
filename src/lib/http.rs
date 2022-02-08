use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use std::fmt;
use std::str::FromStr;

#[derive(Debug)]
pub enum Method {
    Get,
    Post,
}

pub struct Response {
    pub status_code: u16,
    pub body: String,
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

pub fn post(id: String, state: String, url: String) -> Result<Response> {
    let res = reqwest::blocking::get("http://httpbin.org/get")?;
    let status_code = res.status().as_u16();
    let body = res.text()?;
    Ok(Response {
        status_code: status_code,
        body: body,
    })
}

pub fn get(id: &str, state: &str, url: &str) -> Result<Response> {
    let url = format!("{}?id={}&state={}", url, id, state);
    println!("url: {}", url);
    let res = reqwest::blocking::get(url)?;
    let status_code = res.status().as_u16();
    println!("Status: {}", status_code);
    let body = res.text()?;
    println!("Body:\n{}", body);
    Ok(Response {
        status_code: status_code,
        body: body,
    })
}
