use dindim_i18n::args;
use iced::Task;
use iced::widget::{Id, operation};

use super::{App, Message, ids};
use crate::errors;
use crate::history::Change;
use crate::session::{CategoryManager, Notice};

impl App {
    pub(super) fn update_categories(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ManagerKind(kind) => {
                self.with_manager(|m| {
                    m.kind = kind;
                    m.renaming = None;
                    m.confirm_delete = None;
                });
                return operation::focus(Id::new(ids::CATEGORY_SEARCH));
            }
            Message::ManagerSearch(v) => self.with_manager(|m| m.search = v),
            Message::CreateCategory => return self.create_category(),
            Message::StartRename(id) => {
                if let Some(acc) = self.active_mut() {
                    let name = acc.category_name(Some(id)).unwrap_or_default().to_string();
                    acc.manager.renaming = Some((id, name));
                    acc.manager.confirm_delete = None;
                    acc.manager.notice = None;
                }
                return Task::batch([
                    operation::focus(Id::new(ids::RENAME)),
                    operation::select_all(Id::new(ids::RENAME)),
                ]);
            }
            Message::RenameText(v) => self.with_manager(|m| {
                if let Some((_, name)) = &mut m.renaming {
                    *name = v;
                }
            }),
            Message::SaveRename => self.rename_category(),
            Message::CancelRename => self.with_manager(|m| m.renaming = None),
            Message::AskDeleteCategory(id) => self.with_manager(|m| {
                m.confirm_delete = Some(id);
                m.renaming = None;
            }),
            Message::ConfirmDeleteCategory => self.delete_category(),
            Message::CancelDeleteCategory => self.with_manager(|m| m.confirm_delete = None),
            _ => {}
        }
        Task::none()
    }

    fn with_manager(&mut self, change: impl FnOnce(&mut CategoryManager)) {
        if let Some(acc) = self.active_mut() {
            change(&mut acc.manager);
            acc.manager.notice = None;
        }
    }

    fn create_category(&mut self) -> Task<Message> {
        let Some((i18n, acc)) = self.active_parts() else {
            return Task::none();
        };
        let kind = acc.manager.kind;
        let name = acc.manager.search.trim().to_string();
        if name.is_empty() {
            acc.manager.notice = Some(Notice::error(i18n.t("error-category-name-needed")));
            return operation::focus(Id::new(ids::CATEGORY_SEARCH));
        }
        let result = acc
            .ledger
            .add_category(&name, kind)
            .and_then(|c| acc.reload().map(|()| c));
        acc.manager.notice = Some(match result {
            Ok(c) => {
                acc.manager.search.clear();
                let text = i18n.t_with(
                    "notice-category-created",
                    &args! { "name" => c.name.as_str() },
                );
                acc.history.record(Change::CategoryAdded(c));
                Notice::ok(text)
            }
            Err(e) => Notice::error(errors::store(i18n, &e)),
        });
        operation::focus(Id::new(ids::CATEGORY_SEARCH))
    }

    fn rename_category(&mut self) {
        let Some((i18n, acc)) = self.active_parts() else {
            return;
        };
        let Some((id, name)) = acc.manager.renaming.clone() else {
            return;
        };
        let Some(before) = acc.category(Some(id)).cloned() else {
            return;
        };
        let result = acc
            .ledger
            .rename_category(id, &name)
            .and_then(|after| acc.reload().map(|()| after));
        acc.manager.notice = Some(match result {
            Ok(after) => {
                acc.manager.renaming = None;
                let text = i18n.t_with(
                    "notice-category-renamed",
                    &args! { "old" => before.name.as_str(), "new" => after.name.as_str() },
                );
                if before.name != after.name {
                    acc.history
                        .record(Change::CategoryRenamed { before, after });
                }
                Notice::ok(text)
            }
            Err(e) => Notice::error(errors::store(i18n, &e)),
        });
    }

    fn delete_category(&mut self) {
        let Some((i18n, acc)) = self.active_parts() else {
            return;
        };
        let Some(id) = acc.manager.confirm_delete.take() else {
            return;
        };
        let Some(category) = acc.category(Some(id)).cloned() else {
            return;
        };
        let result = acc
            .ledger
            .delete_category(id)
            .and_then(|entries| acc.reload().map(|()| entries));
        acc.manager.notice = Some(match result {
            Ok(entries) => {
                let text = i18n.t_with(
                    "notice-category-deleted",
                    &args! { "name" => category.name.as_str(), "count" => entries.len() },
                );
                acc.history
                    .record(Change::CategoryDeleted { category, entries });
                Notice::ok(text)
            }
            Err(e) => Notice::error(errors::store(i18n, &e)),
        });
    }
}
