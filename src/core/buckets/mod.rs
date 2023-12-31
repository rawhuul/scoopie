mod manifest;
mod metadata;
mod query;
mod sync;

pub use manifest::Manifest;
pub use metadata::write_default_metadata;
pub use query::*;
pub use sync::*;

use std::{collections::HashMap, fmt, format, write};

use console::style;
use serde::{Deserialize, Serialize};

pub type AppName = String;
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Bucket(HashMap<AppName, Manifest>);

pub type BucketName = String;
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Buckets(HashMap<BucketName, Bucket>);

impl Buckets {
    pub fn get_app_from(&self, app_name: &str, bucket_name: &str) -> Option<Manifest> {
        self.0.get(bucket_name)?.0.get(app_name).cloned()
    }

    pub fn get_app(&self, app_name: &str) -> Option<Manifest> {
        self.0
            .values()
            .flat_map(|bucket| bucket.0.get(app_name))
            .next()
            .cloned()
    }
}

impl fmt::Display for Buckets {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .flat_map(|(bucket_name, bucket)| {
                    let bucket_name = style(bucket_name).blue();

                    bucket.0.iter().map(move |(app_name, manifest)| {
                        let app = style(app_name).green();
                        let version = style(format!("v{}", manifest.version)).magenta();
                        let desc = style(&manifest.description).white();
                        format!("{app}/{bucket_name}  {version}\n  {desc}\n")
                    })
                })
                .collect::<String>()
        )
    }
}
