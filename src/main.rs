use anyhow::Result;
use clap::Parser;
use log::error;
use reqwest::Url;
use tabled::Table;

mod lib;
use lib::*;

/// A redis-like tool for storing key-value pairs, and notify url if specified.
/// 1. kvlet.db will be store in KVLET_DB_PATH, KVLET_DB_PATH default to current dir(.).
/// 2. kvlet.log will be store in KVLET_LOG_PATH, KVLET_LOG_PATH default to current dir(./log).
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
    List(List),
}

/// Get value(state) by key
#[derive(Parser, Debug)]
struct Get {
    /// Specify the key, it should be unique
    #[clap(short, long)]
    key: String,
}

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
    #[clap(short, long,parse(try_from_str=parse_url))]
    url: Option<String>,
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

#[tokio::main]
async fn main() -> Result<()> {
    lib::config_log();
    let opts: Opts = Opts::parse();
    let result = match opts.subcmd {
        SubCommand::Get(ref args) => get(args),
        SubCommand::Set(ref args) => set(args),
        SubCommand::List(ref args) => list(args),
    };
    if let Err(e) = result {
        error!("{}", e);
    }
    Ok(())
}

fn get(arg: &Get) -> Result<()> {
    match lib::get(&arg.key) {
        Ok(Some(r)) => println!("{}", &r.state),
        Ok(None) => println!(""),
        Err(e) => println!("{}", e),
    }
    Ok(())
}

fn set(arg: &Set) -> Result<()> {
    let notify = match (arg.method, &arg.url) {
        (Some(method), Some(url)) => Some(Notify {
            method: method,
            url: url.to_string(),
        }),
        (None, Some(url)) => Some(Notify {
            method: Method::Get,
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
