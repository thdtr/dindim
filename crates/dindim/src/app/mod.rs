//! Application state, messages and the update loop (Elm architecture). The
//! handlers live in one submodule per area.

mod accounts;
mod categories;
mod data;
mod entries;
mod system;
#[cfg(test)]
mod tests;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use dindim_core::{CategoryId, CategoryKind, Currency, Date, EntryId, Kind, YearMonth};
use dindim_i18n::I18n;
use dindim_store::backup::BackupContents;
use dindim_store::csv::CsvRow;
use dindim_store::{AccountInfo, Ledger, StoreError, Vault};
use iced::keyboard::{self, Key, key::Named};
use iced::widget::{Id, operation, text_editor};
use iced::{Element, Event, Subscription, Task, Theme, event, window};

use crate::handoff::Handoff;
use crate::preferences::{Language, Preferences, ThemeChoice};
use crate::session::{Notice, OpenAccount};
use crate::updates::Release;
use crate::{errors, theme, views};

pub mod ids {
    pub const AMOUNT: &str = "amount";
    pub const CATEGORY: &str = "category";
    pub const NOTE: &str = "note";
    pub const ENTRY_SEARCH: &str = "entry-search";
    pub const CATEGORY_SEARCH: &str = "category-search";
    pub const RENAME: &str = "rename-category";
    pub const NEW_ACCOUNT: &str = "new-account";
    pub const PASSWORD: &str = "password";
}

const TOAST_DURATION: Duration = Duration::from_secs(4);
const AUTO_BACKUP_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Entries,
    Report,
    Categories,
    Settings,
}

impl Tab {
    pub const ALL: [Tab; 4] = [Tab::Entries, Tab::Report, Tab::Categories, Tab::Settings];
}

/// Floating panels; only one is open at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Picker {
    EntryDate,
    OpeningDate,
    Month,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    Welcome,
    Unlock(String),
    Account(String),
    Backup,
    Preferences,
    About,
}

#[derive(Default)]
pub struct NewAccountForm {
    pub name: String,
    pub password: String,
    pub password_again: String,
    pub currency: Option<Currency>,
    pub error: Option<String>,
}

#[derive(Default)]
pub struct UnlockForm {
    pub password: String,
    pub error: Option<String>,
    pub busy: bool,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub text: String,
    pub ok: bool,
    expires: Instant,
}

pub type StoreResult<T> = Handoff<dindim_store::Result<T>>;

#[derive(Debug, Clone)]
pub enum Message {
    ShowWelcome,
    ShowBackup,
    ShowPreferences,
    ShowAbout,
    SelectAccount(String),
    SetTab(Tab),
    PrevMonth,
    NextMonth,
    GoToMonth(YearMonth),
    TogglePicker(Picker),
    ClosePicker,
    PickerView(YearMonth),

    NewName(String),
    NewPassword(String),
    NewPasswordAgain(String),
    NewCurrency(Currency),
    CreateAccount,
    UnlockPassword(String),
    Unlock,
    Unlocked(String, StoreResult<Ledger>),

    SetKind(Kind),
    Amount(String),
    CategoryQuery(String),
    ToggleCategoryMenu,
    CloseCategoryMenu,
    MenuMove(i32),
    CategorySubmit,
    PickCategory(CategoryId),
    CreateCategoryInline,
    Note(text_editor::Action),
    EntryDate(Date),
    SubmitEntry,
    EditEntry(EntryId),
    CancelEdit,
    DeleteEntry,
    Search(String),

    ManagerKind(CategoryKind),
    ManagerSearch(String),
    CreateCategory,
    StartRename(CategoryId),
    RenameText(String),
    SaveRename,
    CancelRename,
    AskDeleteCategory(CategoryId),
    ConfirmDeleteCategory,
    CancelDeleteCategory,

    SettingsName(String),
    SaveName,
    SettingsPassword(String),
    SettingsPasswordAgain(String),
    SavePassword,
    RemovePassword,
    OpeningAmount(String),
    OpeningDate(Date),
    SaveOpening,
    ClearOpening,
    CloseAccount,
    RemoveAccount,

    ExportCsv,
    ExportCsvTo(Option<PathBuf>),
    CsvWritten(PathBuf, StoreResult<()>),
    ImportCsv,
    ImportCsvFrom(Option<PathBuf>),
    CsvRead(String, StoreResult<Vec<CsvRow>>),
    ExportBackup,
    ExportBackupTo(Option<PathBuf>),
    BackupWritten {
        path: PathBuf,
        count: usize,
        automatic: bool,
        result: StoreResult<()>,
    },
    ImportBackup,
    ImportBackupFrom(Option<PathBuf>),
    BackupRead(StoreResult<BackupContents>),
    OpenDataFolder,

    SetLanguage(Language),
    SetTheme(ThemeChoice),
    SetLockAfter(u32),
    SetAutoBackup(bool),
    SetBackupsToKeep(usize),
    SetCheckUpdates(bool),
    ZoomIn,
    ZoomOut,
    ZoomReset,
    UpdateFound(Option<Release>),
    OpenUrl(String),

    Undo,
    Redo,
    FocusNext(bool),
    Escape,
    NewEntryShortcut,
    FindShortcut,
    LockShortcut,
    Activity,
    Tick,
    SystemTheme(iced::theme::Mode),
    Window(window::Event),
    Quit,
}

/// Everything `App::new` needs from the outside world.
#[derive(Debug, Clone)]
pub struct Config {
    pub data_dir: PathBuf,
    /// Where preferences are saved; `None` keeps them in memory only.
    pub preferences_file: Option<PathBuf>,
    pub preferences: Preferences,
    pub logs_dir: Option<PathBuf>,
    pub today: Date,
    /// Talk to the operating system (theme, update check). Off in tests.
    pub system: bool,
}

pub struct App {
    pub i18n: I18n,
    pub prefs: Preferences,
    preferences_file: Option<PathBuf>,
    pub logs_dir: Option<PathBuf>,
    pub vault: Option<Vault>,
    pub accounts: Vec<AccountInfo>,
    pub open: Vec<OpenAccount>,
    pub screen: Screen,
    pub tab: Tab,
    pub month: YearMonth,
    pub today: Date,
    pub picker: Option<Picker>,
    pub picker_view: YearMonth,
    pub new_account: NewAccountForm,
    pub unlock: UnlockForm,
    pub notice: Option<Notice>,
    pub toast: Option<Toast>,
    pub update: Option<Release>,
    pub system_dark: bool,
    pub busy: bool,
    last_activity: Instant,
    last_backup_check: Option<Instant>,
}

impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("screen", &self.screen)
            .field("tab", &self.tab)
            .finish_non_exhaustive()
    }
}

impl App {
    pub fn new(config: Config) -> (Self, Task<Message>) {
        let locale = config.preferences.language.resolve();
        let mut app = App {
            i18n: I18n::new(locale),
            prefs: config.preferences,
            preferences_file: config.preferences_file,
            logs_dir: config.logs_dir,
            vault: None,
            accounts: Vec::new(),
            open: Vec::new(),
            screen: Screen::Welcome,
            tab: Tab::Entries,
            month: YearMonth::of(config.today),
            today: config.today,
            picker: None,
            picker_view: YearMonth::of(config.today),
            new_account: NewAccountForm::default(),
            unlock: UnlockForm::default(),
            notice: None,
            toast: None,
            update: None,
            system_dark: false,
            busy: false,
            last_activity: Instant::now(),
            last_backup_check: None,
        };
        tracing::info!(
            version = env!("CARGO_PKG_VERSION"),
            os = std::env::consts::OS,
            locale = locale.tag(),
            "dindim started"
        );

        match Vault::open(&config.data_dir) {
            Ok((vault, damaged)) => {
                if let Some(path) = damaged {
                    app.notice = Some(Notice::error(app.i18n.t_with(
                        "notice-account-list-damaged",
                        &dindim_i18n::args! { "path" => path.display().to_string() },
                    )));
                }
                app.vault = Some(vault);
                app.refresh_accounts();
            }
            Err(e) => {
                tracing::error!(error = %e, path = %config.data_dir.display(), "cannot open data folder");
                app.notice = Some(Notice::error(app.i18n.t("error-data-folder")));
            }
        }

        let mut tasks = vec![app.open_startup_account()];
        if config.system {
            tasks.push(iced::system::theme().map(Message::SystemTheme));
            if app.prefs.check_for_updates {
                tasks.push(system::check_for_updates());
            }
        }
        (app, Task::batch(tasks))
    }

    /// Reopens the last account used, or the only one, when no password is needed.
    fn open_startup_account(&mut self) -> Task<Message> {
        let last = self
            .prefs
            .last_account
            .as_ref()
            .and_then(|id| self.accounts.iter().find(|a| &a.id == id));
        let pick = match (last, self.accounts.as_slice()) {
            (Some(info), _) if !info.protected => Some(info.id.clone()),
            (None, [only]) if !only.protected => Some(only.id.clone()),
            _ => None,
        };
        match pick {
            Some(id) => self.select_account(id),
            None => Task::none(),
        }
    }

    pub fn title(&self) -> String {
        match self.active() {
            Some(acc) => format!("{} · dindim", acc.info.name),
            None => "dindim".into(),
        }
    }

    pub fn theme(&self) -> Theme {
        theme::theme(self.is_dark())
    }

    pub fn is_dark(&self) -> bool {
        match self.prefs.theme {
            ThemeChoice::System => self.system_dark,
            ThemeChoice::Light => false,
            ThemeChoice::Dark => true,
        }
    }

    pub fn scale_factor(&self) -> f32 {
        self.prefs.zoom
    }

    pub fn view(&self) -> Element<'_, Message> {
        views::root(self)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let keys = event::listen_with(|event, status, _window| {
            let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event else {
                return match event {
                    Event::Mouse(iced::mouse::Event::ButtonPressed(_))
                    | Event::Mouse(iced::mouse::Event::WheelScrolled { .. }) => {
                        Some(Message::Activity)
                    }
                    _ => None,
                };
            };
            let captured = status == event::Status::Captured;
            if modifiers.command() {
                let shortcut = match key.as_ref() {
                    Key::Character("z") if modifiers.shift() => Some(Message::Redo),
                    Key::Character("Z") => Some(Message::Redo),
                    Key::Character("z") => Some(Message::Undo),
                    Key::Character("y") => Some(Message::Redo),
                    Key::Character("n") => Some(Message::NewEntryShortcut),
                    Key::Character("f") => Some(Message::FindShortcut),
                    Key::Character("l") => Some(Message::LockShortcut),
                    Key::Character(",") => Some(Message::ShowPreferences),
                    Key::Character("q") => Some(Message::Quit),
                    Key::Character("=" | "+") => Some(Message::ZoomIn),
                    Key::Character("-") => Some(Message::ZoomOut),
                    Key::Character("0") => Some(Message::ZoomReset),
                    Key::Character(c @ ("1" | "2" | "3" | "4")) => {
                        let index = c.parse::<usize>().unwrap_or(1) - 1;
                        Some(Message::SetTab(Tab::ALL[index]))
                    }
                    _ => None,
                };
                return shortcut.or(Some(Message::Activity));
            }
            Some(match key {
                Key::Named(Named::Tab) => Message::FocusNext(!modifiers.shift()),
                Key::Named(Named::Escape) => Message::Escape,
                Key::Named(Named::PageUp) => Message::PrevMonth,
                Key::Named(Named::PageDown) => Message::NextMonth,
                Key::Named(Named::ArrowDown) if !captured => Message::MenuMove(1),
                Key::Named(Named::ArrowUp) if !captured => Message::MenuMove(-1),
                _ => Message::Activity,
            })
        });
        let mut subs = vec![
            keys,
            iced::time::every(Duration::from_secs(15)).map(|_| Message::Tick),
            window::events().map(|(_, event)| Message::Window(event)),
            iced::system::theme_changes().map(Message::SystemTheme),
        ];
        if self.toast.is_some() {
            subs.push(iced::time::every(Duration::from_millis(500)).map(|_| Message::Tick));
        }
        Subscription::batch(subs)
    }

    pub fn active(&self) -> Option<&OpenAccount> {
        match &self.screen {
            Screen::Account(id) => self.open.iter().find(|a| &a.info.id == id),
            _ => None,
        }
    }

    fn active_mut(&mut self) -> Option<&mut OpenAccount> {
        match &self.screen {
            Screen::Account(id) => self.open.iter_mut().find(|a| &a.info.id == id),
            _ => None,
        }
    }

    /// The open account together with the translator, for handlers that need both.
    fn active_parts(&mut self) -> Option<(&I18n, &mut OpenAccount)> {
        let App {
            i18n, open, screen, ..
        } = self;
        match screen {
            Screen::Account(id) => open
                .iter_mut()
                .find(|a| &a.info.id == id)
                .map(|acc| (&*i18n, acc)),
            _ => None,
        }
    }

    fn refresh_accounts(&mut self) {
        if let Some(vault) = &self.vault {
            match vault.accounts() {
                Ok(list) => self.accounts = list,
                Err(e) => self.notice = Some(Notice::error(errors::store(&self.i18n, &e))),
            }
        }
    }

    fn focus_amount(&self) -> Task<Message> {
        if self.tab == Tab::Entries && self.active().is_some() {
            operation::focus(Id::new(ids::AMOUNT))
        } else {
            Task::none()
        }
    }

    pub fn toast(&mut self, text: impl Into<String>, ok: bool) {
        self.toast = Some(Toast {
            text: text.into(),
            ok,
            expires: Instant::now() + TOAST_DURATION,
        });
    }

    fn store_error(&self, error: &StoreError) -> String {
        errors::store(&self.i18n, error)
    }

    fn save_preferences(&self) {
        if let Some(path) = &self.preferences_file
            && let Err(e) = self.prefs.save(path)
        {
            tracing::warn!(error = %e, "could not save preferences");
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        if !matches!(
            message,
            Message::Tick
                | Message::Window(_)
                | Message::SystemTheme(_)
                | Message::UpdateFound(_)
                | Message::Unlocked(..)
                | Message::BackupWritten { .. }
                | Message::CsvWritten(..)
                | Message::CsvRead(..)
                | Message::BackupRead(_)
        ) {
            self.last_activity = Instant::now();
        }
        match message {
            Message::ShowWelcome
            | Message::ShowBackup
            | Message::ShowPreferences
            | Message::ShowAbout
            | Message::SelectAccount(_)
            | Message::SetTab(_)
            | Message::PrevMonth
            | Message::NextMonth
            | Message::GoToMonth(_)
            | Message::TogglePicker(_)
            | Message::ClosePicker
            | Message::PickerView(_)
            | Message::NewName(_)
            | Message::NewPassword(_)
            | Message::NewPasswordAgain(_)
            | Message::NewCurrency(_)
            | Message::CreateAccount
            | Message::UnlockPassword(_)
            | Message::Unlock
            | Message::Unlocked(..)
            | Message::SettingsName(_)
            | Message::SaveName
            | Message::SettingsPassword(_)
            | Message::SettingsPasswordAgain(_)
            | Message::SavePassword
            | Message::RemovePassword
            | Message::OpeningAmount(_)
            | Message::OpeningDate(_)
            | Message::SaveOpening
            | Message::ClearOpening
            | Message::CloseAccount
            | Message::RemoveAccount
            | Message::LockShortcut => self.update_accounts(message),

            Message::SetKind(_)
            | Message::Amount(_)
            | Message::CategoryQuery(_)
            | Message::ToggleCategoryMenu
            | Message::CloseCategoryMenu
            | Message::MenuMove(_)
            | Message::CategorySubmit
            | Message::PickCategory(_)
            | Message::CreateCategoryInline
            | Message::Note(_)
            | Message::EntryDate(_)
            | Message::SubmitEntry
            | Message::EditEntry(_)
            | Message::CancelEdit
            | Message::DeleteEntry
            | Message::Search(_)
            | Message::NewEntryShortcut
            | Message::FindShortcut
            | Message::Undo
            | Message::Redo => self.update_entries(message),

            Message::ManagerKind(_)
            | Message::ManagerSearch(_)
            | Message::CreateCategory
            | Message::StartRename(_)
            | Message::RenameText(_)
            | Message::SaveRename
            | Message::CancelRename
            | Message::AskDeleteCategory(_)
            | Message::ConfirmDeleteCategory
            | Message::CancelDeleteCategory => self.update_categories(message),

            Message::ExportCsv
            | Message::ExportCsvTo(_)
            | Message::CsvWritten(..)
            | Message::ImportCsv
            | Message::ImportCsvFrom(_)
            | Message::CsvRead(..)
            | Message::ExportBackup
            | Message::ExportBackupTo(_)
            | Message::BackupWritten { .. }
            | Message::ImportBackup
            | Message::ImportBackupFrom(_)
            | Message::BackupRead(_)
            | Message::OpenDataFolder => self.update_data(message),

            Message::SetLanguage(_)
            | Message::SetTheme(_)
            | Message::SetLockAfter(_)
            | Message::SetAutoBackup(_)
            | Message::SetBackupsToKeep(_)
            | Message::SetCheckUpdates(_)
            | Message::ZoomIn
            | Message::ZoomOut
            | Message::ZoomReset
            | Message::UpdateFound(_)
            | Message::OpenUrl(_)
            | Message::FocusNext(_)
            | Message::Escape
            | Message::Activity
            | Message::Tick
            | Message::SystemTheme(_)
            | Message::Window(_)
            | Message::Quit => self.update_system(message),
        }
    }
}
