#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod errors;
mod form;
mod handoff;
mod history;
mod icons;
mod instance;
mod logging;
mod paths;
mod preferences;
mod session;
mod theme;
mod updates;
mod views;
mod widget;

use dindim_i18n::I18n;
use iced::{Point, Size, window};

use crate::app::{App, Config};
use crate::instance::{InstanceLock, LockError};
use crate::paths::Paths;
use crate::preferences::Preferences;

macro_rules! font {
    ($file:literal) => {
        include_bytes!(concat!("../../../assets/fonts/", $file)).as_slice()
    };
}

pub const FONTS: [&[u8]; 8] = [
    font!("Figtree-Regular.ttf"),
    font!("Figtree-Medium.ttf"),
    font!("Figtree-SemiBold.ttf"),
    font!("Figtree-Bold.ttf"),
    font!("FigtreeTabular-SemiBold.ttf"),
    font!("FigtreeTabular-Bold.ttf"),
    font!("BricolageGrotesque-SemiBold.ttf"),
    font!("BricolageGrotesque-ExtraBold.ttf"),
];

const DEFAULT_SIZE: Size = Size::new(1280.0, 840.0);
const MIN_SIZE: Size = Size::new(1100.0, 680.0);

fn main() -> iced::Result {
    let Some(paths) = Paths::resolve() else {
        eprintln!("dindim: could not find a home folder for app data");
        std::process::exit(1);
    };
    let _log_guard = logging::init(&paths.logs);
    let preferences = Preferences::load(&paths.preferences_file());

    let _lock = match InstanceLock::acquire(&paths.lock_file()) {
        Ok(lock) => lock,
        Err(LockError::AlreadyRunning) => {
            tracing::info!("another instance is already running");
            let i18n = I18n::new(preferences.language.resolve());
            rfd::MessageDialog::new()
                .set_title("dindim")
                .set_description(i18n.t("dialog-already-running"))
                .set_level(rfd::MessageLevel::Info)
                .show();
            return Ok(());
        }
        Err(LockError::Io(e)) => {
            tracing::warn!(error = %e, "could not take the instance lock, continuing");
            InstanceLock::acquire(&std::env::temp_dir().join("dindim-fallback.lock"))
                .unwrap_or_else(|_| std::process::exit(1))
        }
    };

    let window_settings = window_settings(&preferences);
    let config = Config {
        data_dir: paths.data.clone(),
        preferences_file: Some(paths.preferences_file()),
        preferences,
        logs_dir: Some(paths.logs.clone()),
        today: jiff::ZonedDateTime::now().date(),
        system: true,
    };

    iced::application(move || App::new(config.clone()), App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .scale_factor(App::scale_factor)
        .subscription(App::subscription)
        .settings(iced::Settings {
            fonts: FONTS
                .iter()
                .map(|f| std::borrow::Cow::Borrowed(*f))
                .collect(),
            default_font: theme::SANS,
            ..Default::default()
        })
        .window(window_settings)
        .exit_on_close_request(false)
        .antialiasing(true)
        .run()
}

fn window_settings(preferences: &Preferences) -> window::Settings {
    let saved = preferences.window;
    let size = saved
        .map(|w| Size::new(w.width.max(MIN_SIZE.width), w.height.max(MIN_SIZE.height)))
        .unwrap_or(DEFAULT_SIZE);
    let position = match saved.and_then(|w| w.x.zip(w.y)) {
        Some((x, y)) => window::Position::Specific(Point::new(x, y)),
        None => window::Position::Centered,
    };
    window::Settings {
        size,
        position,
        min_size: Some(MIN_SIZE),
        icon: window::icon::from_file_data(
            include_bytes!("../../../assets/icons/dindim-256.png"),
            None,
        )
        .ok(),
        platform_specific: platform_settings(),
        ..Default::default()
    }
}

#[cfg(target_os = "linux")]
fn platform_settings() -> window::settings::PlatformSpecific {
    window::settings::PlatformSpecific {
        application_id: "dindim".into(),
        ..Default::default()
    }
}

#[cfg(not(target_os = "linux"))]
fn platform_settings() -> window::settings::PlatformSpecific {
    window::settings::PlatformSpecific::default()
}
