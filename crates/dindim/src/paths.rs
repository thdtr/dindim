use std::path::PathBuf;

use directories::ProjectDirs;

/// Where dindim keeps its files. Setting `DINDIM_HOME` puts everything under
/// one folder, which is handy for testing.
#[derive(Debug, Clone)]
pub struct Paths {
    pub data: PathBuf,
    pub config: PathBuf,
    pub logs: PathBuf,
}

impl Paths {
    pub fn resolve() -> Option<Paths> {
        if let Some(home) = std::env::var_os("DINDIM_HOME").filter(|v| !v.is_empty()) {
            let home = PathBuf::from(home);
            return Some(Paths {
                data: home.join("data"),
                config: home.join("config"),
                logs: home.join("logs"),
            });
        }
        let dirs = ProjectDirs::from("", "", "dindim")?;
        let logs = dirs
            .state_dir()
            .unwrap_or_else(|| dirs.data_local_dir())
            .join("logs");
        Some(Paths {
            data: dirs.data_dir().to_path_buf(),
            config: dirs.config_dir().to_path_buf(),
            logs,
        })
    }

    pub fn preferences_file(&self) -> PathBuf {
        self.config.join("preferences.toml")
    }

    pub fn lock_file(&self) -> PathBuf {
        self.data.join("dindim.lock")
    }
}
