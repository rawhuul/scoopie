mod bucket;
mod config;
mod error;
mod info;
mod init;
mod install;
mod list;
mod locate;
mod nuke;
mod query;
mod remove;

use argh::FromArgs;

use info::InfoCommand;
use init::InitCommand;
use install::InstallCommand;
use list::ListCommand;
use locate::LocateCommand;
use nuke::NukeCommand;
use query::QueryCommand;
use remove::RemoveCommand;

use crate::bucket::data::BucketData;

#[derive(FromArgs, PartialEq, Debug)]
/// Scoopie, your favorite package manager
struct Scoopie {
    #[argh(subcommand)]
    cmd: Command,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum Command {
    Info(InfoCommand),
    Init(InitCommand),
    Install(InstallCommand),
    List(ListCommand),
    Locate(LocateCommand),
    Nuke(NukeCommand),
    Query(QueryCommand),
    Remove(RemoveCommand),
}

fn main() {
    let cmd: Scoopie = argh::from_env();
    println!("{:?}", cmd);

    match cmd.cmd {
        Command::Install(args) => InstallCommand::from(args),
        Command::Remove(_) => todo!(),
        Command::Query(query) => match BucketData::try_from(query) {
            Ok(results) => println!("{results}"),
            Err(e) => eprintln!("{e}"),
        },
        Command::Locate(_) => todo!(),
        Command::Info(_) => todo!(),
        Command::Init(config) => match InitCommand::from(config) {
            Ok(x) => println!("{x}"),
            Err(e) => eprintln!("{e}"),
        },
        Command::List(_) => todo!(),
        Command::Nuke(_) => match NukeCommand::from() {
            Ok(_) => println!("👋🏻 Goodbye, Scoopie has been deleted!"),
            Err(e) => eprintln!("{e}"),
        },
    }
}
