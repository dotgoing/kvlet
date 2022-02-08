use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use serde::Serialize;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub enum Method {
    Get,
    Post,
    None,
}

pub struct Response {
    pub id: String,
    pub status_code: u16,
    pub body: String,
}

#[derive(Serialize)]
struct PostBody {
    id: String,
    state: String,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Method::Get => write!(f, "{}", "get"),
            Method::Post => write!(f, "{}", "post"),
            Method::None => write!(f, "{}", ""),
        }
    }
}

impl FromStr for Method {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "get" => Ok(Method::Get),
            "post" => Ok(Method::Post),
            "" => Ok(Method::None),
            _ => Err(anyhow!(format!("method not supported {}", s))),
        }
    }
}

pub fn post(id: &str, state: &str, url: &str) -> Result<Response> {
    let url = format!("{}?id={}&state={}", url, id, state);
    println!("POST {}", url);
    let client = Client::new();
    let res = client
        .post(url)
        .json(&PostBody {
            id: id.to_string(),
            state: state.to_string(),
        })
        .send()?;
    let status_code = res.status().as_u16();
    println!("Status: {}", status_code);
    let body = res.text()?;
    println!("Body:\n{}", body);
    Ok(Response {
        id: id.to_string(),
        status_code: status_code,
        body: body,
    })
}

pub fn get(id: &str, state: &str, url: &str) -> Result<Response> {
    let url = format!("{}?id={}&state={}", url, id, state);
    println!("GET {}", url);
    let client = Client::new();
    let res = client.get(url).send()?;
    let status_code = res.status().as_u16();
    println!("Status: {}", status_code);
    let body = res.text()?;
    println!("Body:\n{}", body);
    Ok(Response {
        id: id.to_string(),
        status_code: status_code,
        body: body,
    })
}
