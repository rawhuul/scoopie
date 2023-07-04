mod db;
mod sync;

use std::todo;

use argh::FromArgs;
use sync::Repo;

#[derive(FromArgs, PartialEq, Debug)]
/// Install specified app or Update app(s)
#[argh(subcommand, name = "install")]
pub struct InstallCommand {
    #[argh(positional)]
    app: Option<String>,

    #[argh(switch, short = 'S')]
    /// sync all repos
    sync: bool,

    #[argh(switch, short = 'a')]
    /// update all apps
    update_all: bool,
}

impl InstallCommand {
    pub fn from(args: InstallCommand) {
        if args.sync {
            println!("{:?}", Repo::fetch());
        } else {
            todo!()
        }
    }
}