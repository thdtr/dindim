use dindim_core::{OpeningBalance, YearMonth};
use dindim_i18n::args;
use dindim_store::{AccountInfo, StoreError};
use iced::Task;
use iced::widget::{Id, operation};

use super::{App, Message, NewAccountForm, Picker, Screen, Tab, UnlockForm, ids};
use crate::handoff::Handoff;
use crate::session::{Notice, OpenAccount};

impl App {
    pub(super) fn update_accounts(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ShowWelcome => {
                self.leave_account_screen();
                self.screen = Screen::Welcome;
                self.refresh_accounts();
                return operation::focus(Id::new(ids::NEW_ACCOUNT));
            }
            Message::ShowBackup => {
                self.leave_account_screen();
                self.screen = Screen::Backup;
            }
            Message::ShowPreferences => {
                self.leave_account_screen();
                self.screen = Screen::Preferences;
            }
            Message::ShowAbout => {
                self.leave_account_screen();
                self.screen = Screen::About;
            }
            Message::SelectAccount(id) => return self.select_account(id),
            Message::SetTab(tab) => {
                if self.active().is_none() {
                    return Task::none();
                }
                self.tab = tab;
                self.picker = None;
                if let Some((i18n, acc)) = self.active_parts() {
                    acc.settings.notice = None;
                    acc.reset_settings(i18n);
                    acc.manager.notice = None;
                    acc.manager.renaming = None;
                    acc.manager.confirm_delete = None;
                    acc.category_menu = None;
                }
                if tab == Tab::Categories {
                    return operation::focus(Id::new(ids::CATEGORY_SEARCH));
                }
                return self.focus_amount();
            }
            Message::PrevMonth => {
                if self.active().is_some() {
                    self.month = self.month.prev();
                }
            }
            Message::NextMonth => {
                if self.active().is_some() {
                    self.month = self.month.next();
                }
            }
            Message::GoToMonth(month) => {
                self.month = month;
                if self.picker == Some(Picker::Month) {
                    self.picker = None;
                }
            }
            Message::TogglePicker(picker) => {
                if self.picker == Some(picker) {
                    self.picker = None;
                } else {
                    let today = self.today;
                    let start = match picker {
                        Picker::Month => Some(self.month),
                        Picker::EntryDate => {
                            self.active().map(|a| YearMonth::of(a.form.date_or(today)))
                        }
                        Picker::OpeningDate => self
                            .active()
                            .map(|a| YearMonth::of(a.settings.opening_date.unwrap_or(today))),
                    };
                    self.picker_view = start.unwrap_or(YearMonth::of(today));
                    self.picker = Some(picker);
                    if let Some(acc) = self.active_mut() {
                        acc.category_menu = None;
                    }
                }
            }
            Message::ClosePicker => self.picker = None,
            Message::PickerView(month) => self.picker_view = month,

            Message::NewName(v) => self.new_account.name = v,
            Message::NewPassword(v) => self.new_account.password = v,
            Message::NewPasswordAgain(v) => self.new_account.password_again = v,
            Message::NewCurrency(c) => self.new_account.currency = Some(c),
            Message::CreateAccount => return self.create_account(),
            Message::UnlockPassword(v) => {
                self.unlock.password = v;
                self.unlock.error = None;
            }
            Message::Unlock => return self.start_unlock(),
            Message::Unlocked(id, result) => return self.finish_unlock(id, result),

            Message::SettingsName(v) => self.with_settings(|s| s.name = v),
            Message::SaveName => self.save_name(),
            Message::SettingsPassword(v) => self.with_settings(|s| s.password = v),
            Message::SettingsPasswordAgain(v) => self.with_settings(|s| s.password_again = v),
            Message::SavePassword => self.save_password(false),
            Message::RemovePassword => self.save_password(true),
            Message::OpeningAmount(v) => self.with_settings(|s| s.opening_amount = v),
            Message::OpeningDate(date) => {
                self.with_settings(|s| s.opening_date = Some(date));
                self.picker = None;
            }
            Message::SaveOpening => self.save_opening(false),
            Message::ClearOpening => self.save_opening(true),
            Message::CloseAccount | Message::LockShortcut => {
                if let Screen::Account(id) = self.screen.clone() {
                    self.close_account(&id);
                    self.screen = Screen::Welcome;
                }
            }
            Message::RemoveAccount => self.remove_account(),
            _ => {}
        }
        Task::none()
    }

    fn leave_account_screen(&mut self) {
        self.picker = None;
        self.notice = None;
        if let Some(acc) = self.active_mut() {
            acc.category_menu = None;
        }
    }

    pub(super) fn select_account(&mut self, id: String) -> Task<Message> {
        self.notice = None;
        self.picker = None;
        if self.open.iter().any(|a| a.info.id == id) {
            self.screen = Screen::Account(id);
            return self.focus_amount();
        }
        let Some(info) = self.accounts.iter().find(|a| a.id == id).cloned() else {
            return Task::none();
        };
        self.unlock = UnlockForm::default();
        if info.protected {
            self.screen = Screen::Unlock(id);
            return operation::focus(Id::new(ids::PASSWORD));
        }
        match self.vault.as_ref().map(|v| v.unlock(&info.id, None)) {
            Some(result) => self.install_account(info, result),
            None => Task::none(),
        }
    }

    fn start_unlock(&mut self) -> Task<Message> {
        let Screen::Unlock(id) = self.screen.clone() else {
            return Task::none();
        };
        let Some(vault) = self.vault.clone() else {
            return Task::none();
        };
        if self.unlock.busy {
            return Task::none();
        }
        self.unlock.busy = true;
        self.unlock.error = None;
        let password = std::mem::take(&mut self.unlock.password);
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    vault.unlock(&id, Some(&password)).map(|l| (id, l))
                })
                .await
                .unwrap_or_else(|e| Err(StoreError::Io(std::io::Error::other(e))))
            },
            |result| match result {
                Ok((id, ledger)) => Message::Unlocked(id, Handoff::new(Ok(ledger))),
                Err(e) => Message::Unlocked(String::new(), Handoff::new(Err(e))),
            },
        )
    }

    fn finish_unlock(
        &mut self,
        id: String,
        result: super::StoreResult<dindim_store::Ledger>,
    ) -> Task<Message> {
        self.unlock.busy = false;
        let Some(result) = result.take() else {
            return Task::none();
        };
        let id = if id.is_empty() {
            match &self.screen {
                Screen::Unlock(current) => current.clone(),
                _ => return Task::none(),
            }
        } else {
            id
        };
        let Some(info) = self.accounts.iter().find(|a| a.id == id).cloned() else {
            return Task::none();
        };
        self.install_account(info, result)
    }

    fn install_account(
        &mut self,
        info: AccountInfo,
        result: dindim_store::Result<dindim_store::Ledger>,
    ) -> Task<Message> {
        match result.and_then(|ledger| OpenAccount::new(info.clone(), ledger, &self.i18n)) {
            Ok(account) => {
                tracing::info!(account = %info.id, "account opened");
                let id = account.info.id.clone();
                self.open.retain(|a| a.info.id != id);
                self.open.push(account);
                self.screen = Screen::Account(id.clone());
                self.tab = Tab::Entries;
                self.unlock = UnlockForm::default();
                self.prefs.last_account = Some(id);
                self.save_preferences();
                self.focus_amount()
            }
            Err(StoreError::WrongPassword) => {
                self.unlock.error = Some(self.i18n.t("error-wrong-password"));
                self.unlock.password.clear();
                self.screen = Screen::Unlock(info.id);
                operation::focus(Id::new(ids::PASSWORD))
            }
            Err(e) => {
                self.notice = Some(Notice::error(self.store_error(&e)));
                self.screen = Screen::Welcome;
                Task::none()
            }
        }
    }

    fn create_account(&mut self) -> Task<Message> {
        let form = std::mem::take(&mut self.new_account);
        if form.password != form.password_again {
            self.new_account = NewAccountForm {
                error: Some(self.i18n.t("error-passwords-differ")),
                ..form
            };
            return Task::none();
        }
        let Some(vault) = &self.vault else {
            self.new_account = form;
            return Task::none();
        };
        let currency = form
            .currency
            .unwrap_or_else(|| self.i18n.locale().default_currency());
        let password = Some(form.password.as_str()).filter(|p| !p.is_empty());
        match vault
            .create_account(&form.name, password, currency)
            .and_then(|(info, ledger)| OpenAccount::new(info, ledger, &self.i18n))
        {
            Ok(account) => {
                let id = account.info.id.clone();
                self.open.push(account);
                self.refresh_accounts();
                self.screen = Screen::Account(id.clone());
                self.tab = Tab::Entries;
                self.prefs.last_account = Some(id);
                self.save_preferences();
                self.focus_amount()
            }
            Err(e) => {
                let error = self.store_error(&e);
                self.new_account = NewAccountForm {
                    error: Some(error),
                    ..form
                };
                Task::none()
            }
        }
    }

    fn with_settings(&mut self, change: impl FnOnce(&mut crate::session::SettingsForm)) {
        if let Some(acc) = self.active_mut() {
            change(&mut acc.settings);
            acc.settings.notice = None;
        }
    }

    fn save_name(&mut self) {
        let Some(vault) = self.vault.clone() else {
            return;
        };
        let Some((i18n, acc)) = self.active_parts() else {
            return;
        };
        acc.settings.notice = Some(
            match vault.rename_account(&acc.info.id, &acc.settings.name) {
                Ok(name) => {
                    acc.info.name = name.clone();
                    acc.settings.name = name;
                    Notice::ok(i18n.t("notice-name-saved"))
                }
                Err(e) => Notice::error(crate::errors::store(i18n, &e)),
            },
        );
        self.refresh_accounts();
    }

    fn save_password(&mut self, remove: bool) {
        let Some((i18n, acc)) = self.active_parts() else {
            return;
        };
        let s = &acc.settings;
        if !remove {
            if s.password.is_empty() {
                acc.settings.notice = Some(Notice::error(i18n.t("error-password-empty")));
                return;
            }
            if s.password != s.password_again {
                acc.settings.notice = Some(Notice::error(i18n.t("error-passwords-differ")));
                return;
            }
        }
        let new_password = (!remove).then(|| s.password.clone());
        let id = acc.info.id.clone();
        match acc.ledger.set_password(new_password.as_deref()) {
            Ok(()) => {
                tracing::info!(account = %id, protected = !remove, "password changed");
                acc.info.protected = !remove;
                acc.settings.password.clear();
                acc.settings.password_again.clear();
                acc.settings.notice = Some(Notice::ok(i18n.t(if remove {
                    "notice-password-removed"
                } else {
                    "notice-password-saved"
                })));
                self.refresh_accounts();
            }
            Err(e) => {
                let text = crate::errors::store(i18n, &e);
                self.close_account(&id);
                self.screen = Screen::Welcome;
                self.notice = Some(Notice::error(text));
                self.refresh_accounts();
            }
        }
    }

    fn save_opening(&mut self, clear: bool) {
        let today = self.today;
        let Some((i18n, acc)) = self.active_parts() else {
            return;
        };
        let opening = if clear {
            Ok(None)
        } else {
            let date = acc.settings.opening_date.unwrap_or(today);
            i18n.parse_amount(&acc.settings.opening_amount, acc.currency)
                .map(|amount| Some(OpeningBalance { date, amount }))
                .map_err(|e| crate::errors::amount(i18n, e))
        };
        let result = opening.and_then(|o| {
            acc.ledger
                .set_opening_balance(o)
                .and_then(|()| acc.reload())
                .map_err(|e| crate::errors::store(i18n, &e))
        });
        acc.settings.notice = Some(match result {
            Ok(()) => {
                acc.reset_settings(i18n);
                Notice::ok(i18n.t(if clear {
                    "notice-opening-removed"
                } else {
                    "notice-opening-saved"
                }))
            }
            Err(text) => Notice::error(text),
        });
    }

    pub(super) fn close_account(&mut self, id: &str) {
        self.picker = None;
        self.open.retain(|a| a.info.id != id);
        tracing::info!(account = %id, "account closed");
    }

    fn remove_account(&mut self) {
        let Screen::Account(id) = self.screen.clone() else {
            return;
        };
        let Some(vault) = self.vault.clone() else {
            return;
        };
        let Some(acc) = self.active_mut() else {
            return;
        };
        if !acc.settings.confirm_remove {
            acc.settings.confirm_remove = true;
            return;
        }
        self.close_account(&id);
        self.notice = Some(match vault.remove_account(&id) {
            Ok(path) => Notice::ok(self.i18n.t_with(
                "notice-account-removed",
                &args! { "path" => path.display().to_string() },
            )),
            Err(e) => Notice::error(self.store_error(&e)),
        });
        if self.prefs.last_account.as_deref() == Some(id.as_str()) {
            self.prefs.last_account = None;
            self.save_preferences();
        }
        self.screen = Screen::Welcome;
        self.refresh_accounts();
    }
}
