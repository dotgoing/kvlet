use anyhow::Result;
use clap::Parser;
use tabled::Table;

mod lib;
use lib::Method;

/// A tool to delete the oldest files
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

/// Delete files according the given args
#[derive(Parser, Debug)]
struct Get {
    /// Specify the key, it should be unique
    #[clap(short, long)]
    key: String,
}

/// Delete files according the given args
#[derive(Parser, Debug)]
struct List {
    /// List the latest n items
    #[clap(short, long)]
    num: Option<usize>,
    /// Specify the state
    #[clap(short, long)]
    state: Option<String>,
}

/// Show files which can be deleted according the given args
#[derive(Parser, Debug)]
struct Set {
    /// Specify the key, it should be unique
    #[clap(short, long)]
    key: String,
    /// Specify the state
    #[clap(short, long)]
    state: String,
    #[clap(short, long)]
    url: Option<String>,
    #[clap(short, long, parse(try_from_str=parse_method))]
    method: Option<Method>,
}

fn parse_method(s: &str) -> Result<Method> {
    s.parse()
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let result = match opts.subcmd {
        SubCommand::Get(ref args) => get(args),
        SubCommand::Set(ref args) => set(args),
        SubCommand::List(ref args) => list(args),
    };
    if let Err(e) = result {
        println!("{}", e);
    }
    Ok(())
}

fn get(arg: &Get) -> Result<()> {
    let records = lib::get(&arg.key)?;
    let table = Table::new(records).to_string();
    println!("{}", table);
    Ok(())
}

fn set(arg: &Set) -> Result<()> {
    let get = Some("get".to_string());
    // let get = &get;
    match (&arg.method, &arg.url) {
        (Some(d), Some(k)) => 3,
        _ => 3,
    };
    lib::set(&arg.key, &arg.state)?;
    Ok(())
}

fn list(arg: &List) -> Result<()> {
    let records = lib::list(arg.num.unwrap_or_else(|| 10))?;
    let table = Table::new(records).to_string();
    println!("{}", table);
    Ok(())
}
