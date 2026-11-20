use std::fs;
use std::path::Path;

use dindim_i18n::Locale;
use serde::{Deserialize, Serialize};

pub const ZOOM_STEPS: [f32; 9] = [0.8, 0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.75];
pub const LOCK_CHOICES: [u32; 5] = [0, 5, 10, 15, 30];
pub const BACKUP_KEEP_CHOICES: [usize; 4] = [5, 10, 20, 50];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Language {
    #[default]
    System,
    #[serde(rename = "en-US")]
    EnUs,
    #[serde(rename = "pt-BR")]
    PtBr,
}

impl Language {
    pub const ALL: [Language; 3] = [Language::System, Language::EnUs, Language::PtBr];

    pub fn resolve(self) -> Locale {
        match self {
            Language::EnUs => Locale::EnUs,
            Language::PtBr => Locale::PtBr,
            Language::System => Locale::negotiate(sys_locale::get_locales()),
        }
    }

    pub fn fixed(self) -> Option<Locale> {
        match self {
            Language::System => None,
            Language::EnUs => Some(Locale::EnUs),
            Language::PtBr => Some(Locale::PtBr),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeChoice {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemeChoice {
    pub const ALL: [ThemeChoice; 3] = [ThemeChoice::System, ThemeChoice::Light, ThemeChoice::Dark];
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WindowState {
    pub width: f32,
    pub height: f32,
    pub x: Option<f32>,
    pub y: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Preferences {
    pub language: Language,
    pub theme: ThemeChoice,
    pub zoom: f32,
    pub window: Option<WindowState>,
    pub last_account: Option<String>,
    pub auto_backup: bool,
    pub backups_to_keep: usize,
    /// Minutes of inactivity before password protected accounts lock. Zero
    /// turns it off.
    pub lock_after_minutes: u32,
    pub check_for_updates: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Preferences {
            language: Language::System,
            theme: ThemeChoice::System,
            zoom: 1.0,
            window: None,
            last_account: None,
            auto_backup: true,
            backups_to_keep: 10,
            lock_after_minutes: 10,
            check_for_updates: false,
        }
    }
}

impl Preferences {
    /// Missing or unreadable files give the defaults; the app must start anyway.
    pub fn load(path: &Path) -> Preferences {
        let loaded = fs::read_to_string(path)
            .ok()
            .and_then(|text| match toml::from_str::<Preferences>(&text) {
                Ok(prefs) => Some(prefs),
                Err(e) => {
                    tracing::warn!(error = %e, "preferences file is invalid, using defaults");
                    None
                }
            })
            .unwrap_or_default();
        loaded.sanitized()
    }

    fn sanitized(mut self) -> Self {
        if !self.zoom.is_finite() {
            self.zoom = 1.0;
        }
        self.zoom = self
            .zoom
            .clamp(ZOOM_STEPS[0], ZOOM_STEPS[ZOOM_STEPS.len() - 1]);
        self.backups_to_keep = self.backups_to_keep.clamp(1, 500);
        self
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self).map_err(std::io::Error::other)?;
        let tmp = path.with_extension("toml.tmp");
        fs::write(&tmp, text)?;
        fs::rename(tmp, path)
    }

    pub fn zoom_in(&mut self) {
        if let Some(next) = ZOOM_STEPS.iter().find(|z| **z > self.zoom + 0.001) {
            self.zoom = *next;
        }
    }

    pub fn zoom_out(&mut self) {
        if let Some(prev) = ZOOM_STEPS.iter().rev().find(|z| **z < self.zoom - 0.001) {
            self.zoom = *prev;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_toml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("preferences.toml");
        let prefs = Preferences {
            language: Language::PtBr,
            theme: ThemeChoice::Dark,
            zoom: 1.2,
            window: Some(WindowState {
                width: 1200.0,
                height: 800.0,
                x: Some(10.0),
                y: None,
            }),
            last_account: Some("joe".into()),
            ..Preferences::default()
        };
        prefs.save(&path).unwrap();
        assert_eq!(Preferences::load(&path), prefs);
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("language = \"pt-BR\""));
    }

    #[test]
    fn bad_files_fall_back_to_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("preferences.toml");
        assert_eq!(Preferences::load(&path), Preferences::default());
        fs::write(&path, "zoom = \"huge\"").unwrap();
        assert_eq!(Preferences::load(&path), Preferences::default());
        fs::write(&path, "zoom = 40.0").unwrap();
        assert_eq!(Preferences::load(&path).zoom, 1.75);
    }

    #[test]
    fn zoom_moves_in_steps() {
        let mut prefs = Preferences::default();
        prefs.zoom_in();
        assert_eq!(prefs.zoom, 1.1);
        prefs.zoom_out();
        prefs.zoom_out();
        assert_eq!(prefs.zoom, 0.9);
        prefs.zoom = 0.8;
        prefs.zoom_out();
        assert_eq!(prefs.zoom, 0.8);
    }
}
