use anyhow::Result;
use clap::Parser;
use log::{error, info, warn};
use reqwest::Url;
use tabled::Table;

mod lib;
use lib::*;

/// A redis-like tool for storing key-value pairs, and notify url if specified.
#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "sean")]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug)]
enum SubCommand {
    Get(Get),
    Set(Set),
    Env(Env),
    List(List),
}

/// Get value(state) by key
#[derive(Parser, Debug)]
struct Get {
    /// Specify the key, it should be unique
    #[clap(short, long)]
    key: String,
    /// Specify the notify url, which will be notified on set command
    #[clap(short, long,parse(try_from_str=parse_url))]
    url: Option<String>,
    /// Specify the notify method, default to POST
    #[clap(short, long, parse(try_from_str=parse_method))]
    method: Option<Method>,
}

/// Show env varialbes used by kvlet, show location of kvlet.db and kvlet.log
#[derive(Parser, Debug)]
struct Env {}

/// List key-value pairs
#[derive(Parser, Debug)]
struct List {
    /// Show only the latest n keys
    #[clap(short, long)]
    num: Option<usize>,
    /// Filter by value(state)
    #[clap(short, long)]
    state: Option<String>,
}

/// Set (key, value) pair, if url is specified, the url will be notified
#[derive(Parser, Debug)]
struct Set {
    /// Key should be unique
    #[clap(short, long)]
    key: String,
    /// State can be any string, (running, done, fail etc)
    #[clap(short, long)]
    state: String,
    /// Specify the notify url, which will be notified on set command
    #[clap(short, long,parse(try_from_str=parse_url))]
    url: Option<String>,
    /// Specify the notify method, default to POST
    #[clap(short, long, parse(try_from_str=parse_method))]
    method: Option<Method>,
}

fn parse_method(s: &str) -> Result<Method> {
    s.parse()
}

fn parse_url(s: &str) -> Result<String> {
    let _url: Url = s.parse()?;
    Ok(s.into())
}

fn main() -> Result<()> {
    lib::config_log();
    let opts: Opts = Opts::parse();
    let result = match opts.subcmd {
        SubCommand::Get(ref args) => get(args),
        SubCommand::Set(ref args) => set(args),
        SubCommand::Env(_) => show_env(),
        SubCommand::List(ref args) => list(args),
    };
    if let Err(e) = result {
        error!("{}", e);
    }
    Ok(())
}

fn get(arg: &Get) -> Result<()> {
    let notify = match (arg.method, &arg.url) {
        (Some(method), Some(url)) => Some(Notify {
            method: method,
            url: url.to_string(),
        }),
        (None, Some(url)) => Some(Notify {
            method: Method::Post,
            url: url.to_string(),
        }),
        _ => None,
    };

    match lib::get(&arg.key, notify) {
        Ok(Some(r)) => {
            info!("get : {} {}", &arg.key, &r.state);
            println!("{}", &r.state);
            Ok(())
        }
        Ok(None) => {
            warn!("no record found {}", arg.key);
            Ok(())
        }
        Err(e) => {
            error!("{}", e);
            Err(e)
        }
    }
}

fn set(arg: &Set) -> Result<()> {
    let notify = match (arg.method, &arg.url) {
        (Some(method), Some(url)) => Some(Notify {
            method: method,
            url: url.to_string(),
        }),
        (None, Some(url)) => Some(Notify {
            method: Method::Post,
            url: url.to_string(),
        }),
        _ => None,
    };
    lib::set(InRecord {
        id: arg.key.to_string(),
        state: arg.state.to_string(),
        notify,
    })?;
    Ok(())
}

fn list(arg: &List) -> Result<()> {
    let lines = arg.num.unwrap_or_else(|| 10);
    let records = lib::list(lines, &arg.state)?;
    let table = Table::new(records).to_string();
    println!("{}", table);
    Ok(())
}

use std::env;
use std::path::Path;
fn show_env() -> Result<()> {
    let db_name = "kvlet.db";
    match env::var("KVLET_DB_PATH") {
        Ok(dir) => println!(
            "KVLET_DB_PATH={} , kvlet.db is saved in {:?}",
            &dir,
            Path::new(&dir).join(db_name)
        ),
        _ => println!("KVLET_DB_PATH=. , kvlet.db is saved in ./{}", db_name),
    };
    let log_name = "kvlet.log";
    match env::var("KVLET_LOG_PATH") {
        Ok(dir) => println!(
            "KVLET_LOG_PATH={} , kvlet.log is saved in {:?}",
            &dir,
            Path::new(&dir).join(log_name)
        ),
        _ => println!(
            "KVLET_LOG_PATH=. , kvlet.log is saved in ./log/{}",
            log_name
        ),
    }
    Ok(())
}
