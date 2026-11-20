use dindim_core::YearMonth;
use iced::Task;
use iced::widget::{Id, operation};

use super::{App, Message, Screen, Tab, ids};
use crate::errors;
use crate::form::EntryForm;
use crate::history::Change;

impl App {
    pub(super) fn update_entries(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SetKind(kind) => {
                self.with_form(|f| f.set_kind(kind));
                self.close_category_menu();
                return self.focus_amount();
            }
            Message::Amount(v) => self.with_form(|f| f.amount = v),
            Message::CategoryQuery(v) => {
                self.with_form(|f| {
                    f.category_query = v;
                    f.category = None;
                });
                if let Some(acc) = self.active_mut() {
                    acc.category_menu = Some(0);
                }
                self.picker = None;
            }
            Message::ToggleCategoryMenu => {
                self.picker = None;
                if let Some(acc) = self.active_mut() {
                    acc.category_menu = match acc.category_menu {
                        Some(_) => None,
                        None => Some(0),
                    };
                }
                return operation::focus(Id::new(ids::CATEGORY));
            }
            Message::CloseCategoryMenu => self.close_category_menu(),
            Message::MenuMove(delta) => {
                if let Some(acc) = self.active_mut() {
                    let len = acc.menu_len();
                    if let (Some(current), true) = (acc.category_menu, len > 0) {
                        let next = (current as i64 + i64::from(delta)).rem_euclid(len as i64);
                        acc.category_menu = Some(next as usize);
                    }
                }
            }
            Message::CategorySubmit => {
                let choice = self.active().and_then(|acc| {
                    let index = acc.category_menu?;
                    let (items, create) = acc.menu_rows();
                    match items.get(index) {
                        Some(c) => Some(Some(c.id)),
                        None if create && index == items.len() => Some(None),
                        None => None,
                    }
                });
                return match choice {
                    Some(Some(id)) => self.update_entries(Message::PickCategory(id)),
                    Some(None) => self.update_entries(Message::CreateCategoryInline),
                    None => self.submit_entry(),
                };
            }
            Message::PickCategory(id) => {
                self.with_form(|f| f.pick_category(id));
                self.close_category_menu();
                return operation::focus(Id::new(ids::NOTE));
            }
            Message::CreateCategoryInline => return self.create_category_inline(),
            Message::Note(action) => {
                if let Some(acc) = self.active_mut() {
                    acc.note_editor.perform(action);
                    let text = acc.note_editor.text();
                    acc.form.note = text.trim_end_matches('\n').replace('\n', " ");
                    acc.form.error = None;
                    acc.form.confirm_delete = false;
                }
            }
            Message::EntryDate(date) => {
                let today = self.today;
                self.with_form(|f| f.date = (date != today).then_some(date));
                self.picker = None;
                return self.focus_amount();
            }
            Message::SubmitEntry => return self.submit_entry(),
            Message::EditEntry(id) => {
                if let Some((i18n, acc)) = self.active_parts() {
                    if acc.form.editing == Some(id) {
                        acc.form = EntryForm::default();
                    } else if let Some(entry) = acc.entries.iter().find(|e| e.id == id) {
                        acc.form = EntryForm::load(entry, &acc.categories, i18n, acc.currency);
                    }
                    acc.sync_note();
                    acc.category_menu = None;
                }
                self.picker = None;
                return self.focus_amount();
            }
            Message::CancelEdit => {
                if let Some(acc) = self.active_mut() {
                    acc.form = EntryForm::default();
                    acc.sync_note();
                }
                return self.focus_amount();
            }
            Message::DeleteEntry => return self.delete_entry(),
            Message::Search(v) => {
                if let Some(acc) = self.active_mut() {
                    acc.search = v;
                }
            }
            Message::NewEntryShortcut => {
                if self.active().is_none() {
                    return Task::none();
                }
                self.tab = Tab::Entries;
                self.picker = None;
                if let Some(acc) = self.active_mut() {
                    acc.form = EntryForm {
                        kind: acc.form.kind,
                        ..EntryForm::default()
                    };
                    acc.sync_note();
                    acc.category_menu = None;
                }
                return self.focus_amount();
            }
            Message::FindShortcut => {
                if self.active().is_none() {
                    return Task::none();
                }
                return match self.tab {
                    Tab::Categories => operation::focus(Id::new(ids::CATEGORY_SEARCH)),
                    _ => {
                        self.tab = Tab::Entries;
                        operation::focus(Id::new(ids::ENTRY_SEARCH))
                    }
                };
            }
            Message::Undo => self.step_history(true),
            Message::Redo => self.step_history(false),
            _ => {}
        }
        Task::none()
    }

    fn with_form(&mut self, change: impl FnOnce(&mut EntryForm)) {
        if let Some(acc) = self.active_mut() {
            change(&mut acc.form);
            acc.form.error = None;
            acc.form.confirm_delete = false;
        }
    }

    pub(super) fn close_category_menu(&mut self) {
        if let Some(acc) = self.active_mut() {
            acc.category_menu = None;
        }
    }

    /// Typed text in the category field becomes the matching category.
    pub(super) fn resolve_category_query(&mut self) {
        if let Some(acc) = self.active_mut() {
            acc.category_menu = None;
            if !acc.form.category_query.is_empty() {
                acc.form.resolve_query(&acc.categories);
            }
        }
    }

    fn create_category_inline(&mut self) -> Task<Message> {
        let Some((i18n, acc)) = self.active_parts() else {
            return Task::none();
        };
        let Some(kind) = acc.form.kind.category_kind() else {
            return Task::none();
        };
        let name = acc.form.category_query.trim().to_string();
        match acc.ledger.add_category(&name, kind) {
            Ok(category) => {
                acc.history.record(Change::CategoryAdded(category.clone()));
                if let Err(e) = acc.reload() {
                    acc.form.error = None;
                    tracing::error!(error = %e, "reload failed");
                }
                acc.form.pick_category(category.id);
                acc.category_menu = None;
                operation::focus(Id::new(ids::NOTE))
            }
            Err(e) => {
                let text = errors::store(i18n, &e);
                acc.category_menu = None;
                self.toast(text, false);
                Task::none()
            }
        }
    }

    fn submit_entry(&mut self) -> Task<Message> {
        let today = self.today;
        let mut saved_month = None;
        let mut failure = None;
        if let Some((i18n, acc)) = self.active_parts() {
            acc.category_menu = None;
            match acc.form.build(&acc.categories, today, i18n, acc.currency) {
                Err(e) => acc.form.error = Some(e),
                Ok(new) => {
                    let result = match acc.form.editing {
                        Some(id) => {
                            let before = acc.entries.iter().find(|e| e.id == id).cloned();
                            acc.ledger
                                .update_entry(&new.clone().with_id(id))
                                .map(|after| {
                                    before.map(|before| Change::EntryUpdated { before, after })
                                })
                        }
                        None => acc
                            .ledger
                            .add_entry(&new)
                            .map(|entry| Some(Change::EntryAdded(entry))),
                    };
                    match result.and_then(|change| acc.reload().map(|()| change)) {
                        Ok(change) => {
                            if let Some(change) = change {
                                acc.history.record(change);
                            }
                            acc.form.after_save();
                            acc.sync_note();
                            saved_month = Some(YearMonth::of(new.date));
                        }
                        Err(e) => failure = Some(errors::store(i18n, &e)),
                    }
                }
            }
        }
        if let Some(text) = failure {
            self.toast(text, false);
        }
        // Jump to the month the entry landed in, so it is visible right away.
        if let Some(month) = saved_month {
            self.month = month;
        }
        self.focus_amount()
    }

    fn delete_entry(&mut self) -> Task<Message> {
        let mut failure = None;
        if let Some((i18n, acc)) = self.active_parts() {
            let Some(id) = acc.form.editing else {
                return Task::none();
            };
            if !acc.form.confirm_delete {
                acc.form.confirm_delete = true;
                return Task::none();
            }
            let entry = acc.entries.iter().find(|e| e.id == id).cloned();
            match acc.ledger.delete_entry(id).and_then(|()| acc.reload()) {
                Ok(()) => {
                    if let Some(entry) = entry {
                        acc.history.record(Change::EntryDeleted(entry));
                    }
                    acc.form = EntryForm::default();
                    acc.sync_note();
                }
                Err(e) => failure = Some(errors::store(i18n, &e)),
            }
        }
        match failure {
            Some(text) => self.toast(text, false),
            None => {
                let text = self.i18n.t("toast-entry-deleted");
                self.toast(text, true);
            }
        }
        self.focus_amount()
    }

    fn step_history(&mut self, undo: bool) {
        if !matches!(self.screen, Screen::Account(_)) {
            return;
        }
        let outcome = self.active_parts().map(|(i18n, acc)| {
            let step = if undo {
                acc.history.undo(&mut acc.ledger)
            } else {
                acc.history.redo(&mut acc.ledger)
            };
            let result = step.map(|r| r.and_then(|()| acc.reload()));
            if result.is_some() {
                acc.form = EntryForm {
                    kind: acc.form.kind,
                    ..EntryForm::default()
                };
                acc.sync_note();
                acc.manager.renaming = None;
                acc.manager.confirm_delete = None;
            }
            result.map(|r| r.map_err(|e| errors::store(i18n, &e)))
        });
        match outcome.flatten() {
            Some(Ok(())) => {
                let text = self
                    .i18n
                    .t(if undo { "toast-undone" } else { "toast-redone" });
                self.toast(text, true);
            }
            Some(Err(text)) => self.toast(text, false),
            None => {
                let text = self.i18n.t(if undo {
                    "toast-nothing-to-undo"
                } else {
                    "toast-nothing-to-redo"
                });
                self.toast(text, true);
            }
        }
    }
}
