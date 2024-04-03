use std::path::{Path, PathBuf};

pub fn logfile_path() -> PathBuf {
    let env_local = std::env::var("LocalAppData").unwrap();
    let log_file_path = Path::new(&env_local);
    log_file_path.join("Riot Games/Riot Client/Config/lockfile")
}