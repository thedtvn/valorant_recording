use std::path::{Path, PathBuf};
use reqwest::header;
use crate::Lockfile;

pub fn logfile_path() -> PathBuf {
    let env_local = std::env::var("LocalAppData").unwrap();
    let log_file_path = Path::new(&env_local);
    log_file_path.join("Riot Games/Riot Client/Config/lockfile")
}

pub fn local_client(lockfile: &Lockfile) -> reqwest::Client {
    let mut headers = header::HeaderMap::new();
    headers.insert("Authorization", lockfile.auth_header().parse().unwrap());
    reqwest::ClientBuilder::new().danger_accept_invalid_certs(true).default_headers(headers).build().unwrap()
}