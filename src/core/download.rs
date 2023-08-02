use std::path::PathBuf;

use rayon::prelude::*;
use tokio::runtime::Runtime;
use trauma::{download::Download, downloader::DownloaderBuilder};
use url::Url;

use {
    super::{bucket::*, config::*, verify::Hash},
    crate::error::*,
};

#[derive(Debug)]
pub enum DownloadStatus {
    Success,
}

#[derive(Debug)]
struct Metadata(String, Hash, Url);

impl Metadata {
    #[inline]
    fn exists(&self) -> Result<bool, ScoopieError> {
        Ok(Config::cache_dir()?.join(&self.0).exists())
    }
}

#[derive(Debug)]
struct DownloadEntry {
    app_name: String,
    version: String,
    metadata: Vec<Metadata>,
}

trait FetchFromBucket<T> {
    type Error;

    fn fetch(app_name: T) -> Result<Self, Self::Error>
    where
        Self: Sized;

    fn fetch_from(app_name: T, bucket_name: T) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl FetchFromBucket<&str> for DownloadEntry {
    type Error = ScoopieError;

    fn fetch(app_name: &str) -> Result<Self, Self::Error> {
        let res = Bucket::query(QueryKind::APP, app_name.into())?;

        let manifest = res
            .entries()
            .par_iter()
            .find_map_first(|(_, entries)| entries.par_iter().find_first(|entry| entry.app_name == app_name).map(|entry| entry.manifest.to_owned()))
            .ok_or_else(|| ScoopieError::Download(DownloadError::NoAppFound(app_name.into())))?;

        let app_name: String = app_name.into();
        let version = manifest.version.clone();

        let metadata: Vec<Metadata> = manifest
            .url()
            .into_par_iter()
            .zip(manifest.hash().into_par_iter())
            .map(|(url, hash)| {
                let file = format!("{}_{}{}{}", app_name, version, url.path(), url.fragment().unwrap_or("")).replace("/", "_");

                Metadata(file, hash, url)
            })
            .collect();

        Ok(Self { app_name, version, metadata })
    }

    fn fetch_from(app_name: &str, bucket_name: &str) -> Result<Self, Self::Error> {
        let res = Bucket::query(QueryKind::APP, app_name.into())?;

        let manifest = res
            .entries()
            .get(bucket_name)
            .map(|entries| entries.par_iter().find_first(|x| x.app_name == app_name))
            .flatten()
            .map(|entry| entry.manifest.to_owned())
            .ok_or_else(|| ScoopieError::Download(DownloadError::NoAppFoundInBucket(app_name.into(), bucket_name.into())))?;

        let app_name: String = app_name.into();
        let version = manifest.version.clone();

        let metadata: Vec<Metadata> = manifest
            .url()
            .into_par_iter()
            .zip(manifest.hash().into_par_iter())
            .map(|(url, hash)| {
                let file = format!("{}_{}{}{}", app_name, version, url.path(), url.fragment().unwrap_or("")).replace("/", "_");

                Metadata(file, hash, url)
            })
            .collect();

        Ok(Self { app_name, version, metadata })
    }
}

impl DownloadEntry {
    fn get(&self) -> Result<Vec<Download>, ScoopieError> {
        self.metadata
            .par_iter()
            .filter_map(|m| match m.exists() {
                Ok(true) => None,
                Ok(false) => Some(Ok(Download::new(&m.2, &m.0))),
                Err(err) => Some(Err(err)),
            })
            .collect()
    }

    fn verify(&self) -> Result<bool, ScoopieError> {
        let download_dir = Config::cache_dir()?;
        Ok(self.metadata.par_iter().map(|m| m.1.verify(&PathBuf::from(download_dir.join(&m.0))).unwrap()).all(|v| v == true))
    }
}

pub struct Downloader {
    item: DownloadEntry,
    max_retries: u32,
    concurrent: usize,
    download_dir: PathBuf,
}

pub trait Builder<T> {
    type Error;
    fn build_for(app_name: T) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl Builder<&str> for Downloader {
    type Error = ScoopieError;

    fn build_for(app_name: &str) -> Result<Self, Self::Error> {
        let query = app_name.trim().to_lowercase();

        let item = match app_name.split_once('/') {
            Some((bucket, app)) => DownloadEntry::fetch_from(app, bucket),
            None => DownloadEntry::fetch(&query),
        }?;

        let download_cfg = Config::read()?.download();
        let max_retries = download_cfg.max_retries;
        let concurrent = download_cfg.concurrent_downloads;

        let download_dir = Config::cache_dir()?;

        println!("Found: {} v{}", item.app_name, item.version);

        Ok(Self {
            item,
            max_retries,
            concurrent,
            download_dir,
        })
    }
}

impl Downloader {
    pub fn download(&self, verify: bool) -> Result<DownloadStatus, ScoopieError> {
        let dm = DownloaderBuilder::new()
            .concurrent_downloads(self.concurrent)
            .retries(self.max_retries)
            .directory(self.download_dir.to_path_buf())
            .build();

        let downloads = self.item.get()?;

        if !downloads.is_empty() {
            let rt = Runtime::new().unwrap();
            let _s = rt.block_on(dm.download(&downloads));
            let verified = if verify { self.item.verify()? } else { false };
            println!("{verified}");
        } else {
            println!("\"{} v{}\" already in cache", self.item.app_name, self.item.version);
        }

        Ok(DownloadStatus::Success)
    }
}
