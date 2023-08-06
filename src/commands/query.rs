use argh::FromArgs;

use super::prelude::*;
use crate::core::bucket::*;
use crate::error::ScoopieError;

#[derive(FromArgs, PartialEq, Debug)]
/// Search available apps from buckets (supports full-text search)
#[argh(subcommand, name = "query")]
pub struct QueryCommand {
    #[argh(positional)]
    query: String,
}

impl ExecuteCommand for QueryCommand {
    fn exec(&self) -> Result<(), ScoopieError> {
        let term = self.query.trim();

        let res = match term.contains(" ") {
            true => {
                let query = term.split_whitespace().collect::<Vec<&str>>().join(" AND ");
                Bucket::query(QueryKind::FULLTEXT, format!("{query}*"))
            }
            false => Bucket::query(QueryKind::KEYWORD, format!("{term}*")),
        }?;

        println!("{}", res);
        Ok(())
    }
}
