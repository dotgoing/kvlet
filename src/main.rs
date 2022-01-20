use anyhow::Result;
use clap::Parser;

mod lib;
use lib::*;

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
    #[clap(short, long)]
    method: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    match opts.subcmd {
        SubCommand::Get(ref args) => get(args),
        SubCommand::Set(ref args) => set(args),
        SubCommand::List(ref args) => list(args),
    };
    Ok(())
}

fn get(arg: &Get) {
    println!("{:?}", arg);
    let records = lib::get(&arg.key).unwrap();
    println!("{:?}", records);
}

fn set(arg: &Set) {
    println!("{:?}", arg);
    lib::set(&arg.key, &arg.state).unwrap();
}

fn list(arg: &List) {
    println!("{:?}", arg);
    let records = lib::list(arg.num.unwrap_or_else(|| 10)).unwrap();
    println!("{:?}", records);
}
