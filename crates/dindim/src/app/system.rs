use std::time::{Duration, Instant};

use dindim_i18n::I18n;
use iced::widget::operation;
use iced::{Task, window};

use super::{App, Message, Screen, Tab};
use crate::form::EntryForm;
use crate::preferences::WindowState;
use crate::updates;

pub(super) fn check_for_updates() -> Task<Message> {
    Task::perform(
        async {
            tokio::task::spawn_blocking(|| updates::check(env!("CARGO_PKG_VERSION")))
                .await
                .ok()
                .flatten()
        },
        Message::UpdateFound,
    )
}

impl App {
    pub(super) fn update_system(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SetLanguage(language) => {
                self.prefs.language = language;
                self.i18n = I18n::new(language.resolve());
                let i18n = &self.i18n;
                for acc in &mut self.open {
                    acc.reset_settings(i18n);
                }
                self.save_preferences();
            }
            Message::SetTheme(choice) => {
                self.prefs.theme = choice;
                self.save_preferences();
            }
            Message::SetLockAfter(minutes) => {
                self.prefs.lock_after_minutes = minutes;
                self.save_preferences();
            }
            Message::SetAutoBackup(on) => {
                self.prefs.auto_backup = on;
                self.last_backup_check = None;
                self.save_preferences();
                return self.maybe_auto_backup();
            }
            Message::SetBackupsToKeep(keep) => {
                self.prefs.backups_to_keep = keep;
                self.save_preferences();
            }
            Message::SetCheckUpdates(on) => {
                self.prefs.check_for_updates = on;
                self.save_preferences();
                if on {
                    return check_for_updates();
                }
                self.update = None;
            }
            Message::ZoomIn => {
                self.prefs.zoom_in();
                self.save_preferences();
            }
            Message::ZoomOut => {
                self.prefs.zoom_out();
                self.save_preferences();
            }
            Message::ZoomReset => {
                self.prefs.zoom = 1.0;
                self.save_preferences();
            }
            Message::UpdateFound(release) => {
                if let Some(r) = &release {
                    tracing::info!(version = %r.version, "update available");
                }
                self.update = release;
            }
            Message::OpenUrl(url) => {
                if let Err(e) = open::that_detached(&url) {
                    tracing::warn!(error = %e, "could not open link");
                }
            }

            Message::FocusNext(forward) => {
                self.resolve_category_query();
                return if forward {
                    operation::focus_next()
                } else {
                    operation::focus_previous()
                };
            }
            Message::Escape => return self.escape(),
            Message::Activity => {}
            Message::Tick => return self.tick(),
            Message::SystemTheme(mode) => {
                self.system_dark = mode == iced::theme::Mode::Dark;
            }
            Message::Window(event) => match event {
                window::Event::Resized(size) => {
                    let mut state = self.prefs.window.unwrap_or(WindowState {
                        width: size.width,
                        height: size.height,
                        x: None,
                        y: None,
                    });
                    state.width = size.width;
                    state.height = size.height;
                    self.prefs.window = Some(state);
                }
                window::Event::Moved(point) => {
                    if let Some(state) = &mut self.prefs.window {
                        state.x = Some(point.x);
                        state.y = Some(point.y);
                    }
                }
                window::Event::CloseRequested => return self.quit(),
                _ => {}
            },
            Message::Quit => return self.quit(),
            _ => {}
        }
        Task::none()
    }

    fn quit(&mut self) -> Task<Message> {
        self.save_preferences();
        tracing::info!("dindim closed");
        iced::exit()
    }

    /// One Esc closes one thing: a panel, then a pending edit, then the form.
    fn escape(&mut self) -> Task<Message> {
        if self.picker.take().is_some() {
            return Task::none();
        }
        let on_entries = self.tab == Tab::Entries;
        if let Some(acc) = self.active_mut() {
            acc.settings.confirm_remove = false;
            if acc.category_menu.take().is_some() {
                return Task::none();
            }
            if acc.manager.renaming.is_some() || acc.manager.confirm_delete.is_some() {
                acc.manager.renaming = None;
                acc.manager.confirm_delete = None;
                return Task::none();
            }
            if on_entries {
                acc.form = EntryForm {
                    kind: acc.form.kind,
                    ..EntryForm::default()
                };
                acc.sync_note();
            }
        }
        self.focus_amount()
    }

    fn tick(&mut self) -> Task<Message> {
        if self
            .toast
            .as_ref()
            .is_some_and(|t| Instant::now() >= t.expires)
        {
            self.toast = None;
        }
        let today = jiff::ZonedDateTime::now().date();
        if today != self.today {
            let following = self.month == dindim_core::YearMonth::of(self.today);
            self.today = today;
            if following {
                self.month = dindim_core::YearMonth::of(today);
            }
        }
        self.lock_if_idle();
        self.maybe_auto_backup()
    }

    pub(super) fn lock_if_idle(&mut self) {
        let minutes = self.prefs.lock_after_minutes;
        if minutes == 0
            || self.last_activity.elapsed() < Duration::from_secs(u64::from(minutes) * 60)
        {
            return;
        }
        let locked: Vec<String> = self
            .open
            .iter()
            .filter(|a| a.info.protected)
            .map(|a| a.info.id.clone())
            .collect();
        if locked.is_empty() {
            return;
        }
        for id in &locked {
            self.close_account(id);
        }
        tracing::info!(accounts = locked.len(), "locked after inactivity");
        if let Screen::Account(id) = &self.screen
            && locked.contains(id)
        {
            self.screen = Screen::Unlock(id.clone());
            self.unlock = Default::default();
        }
        let text = self.i18n.t("toast-locked-idle");
        self.toast(text, true);
    }
}
