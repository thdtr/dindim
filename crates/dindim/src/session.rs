use dindim_core::{Category, CategoryId, CategoryKind, Currency, Date, Entry, OpeningBalance};
use dindim_i18n::I18n;
use dindim_store::{AccountInfo, Ledger, Result};
use iced::widget::text_editor;

use crate::form::EntryForm;
use crate::history::History;

/// A short message shown at the top of a screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    pub ok: bool,
    pub text: String,
}

impl Notice {
    pub fn ok(text: impl Into<String>) -> Self {
        Notice {
            ok: true,
            text: text.into(),
        }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Notice {
            ok: false,
            text: text.into(),
        }
    }
}

#[derive(Default)]
pub struct SettingsForm {
    pub name: String,
    pub password: String,
    pub password_again: String,
    pub opening_amount: String,
    pub opening_date: Option<Date>,
    pub confirm_remove: bool,
    pub notice: Option<Notice>,
}

impl std::fmt::Debug for SettingsForm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SettingsForm")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone)]
pub struct CategoryManager {
    pub kind: CategoryKind,
    /// Search text, which is also the name of a new category.
    pub search: String,
    pub renaming: Option<(CategoryId, String)>,
    pub confirm_delete: Option<CategoryId>,
    pub notice: Option<Notice>,
}

impl Default for CategoryManager {
    fn default() -> Self {
        CategoryManager {
            kind: CategoryKind::Expense,
            search: String::new(),
            renaming: None,
            confirm_delete: None,
            notice: None,
        }
    }
}

pub struct OpenAccount {
    pub info: AccountInfo,
    pub ledger: Ledger,
    pub currency: Currency,
    pub entries: Vec<Entry>,
    pub categories: Vec<Category>,
    pub opening: Option<OpeningBalance>,
    pub form: EntryForm,
    pub note_editor: text_editor::Content,
    /// Highlighted row of the category menu, `None` while it is closed.
    pub category_menu: Option<usize>,
    pub search: String,
    pub settings: SettingsForm,
    pub manager: CategoryManager,
    pub history: History,
}

impl std::fmt::Debug for OpenAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenAccount")
            .field("id", &self.info.id)
            .finish_non_exhaustive()
    }
}

impl OpenAccount {
    pub fn new(info: AccountInfo, ledger: Ledger, i18n: &I18n) -> Result<Self> {
        let mut account = OpenAccount {
            info,
            currency: ledger.currency(),
            ledger,
            entries: Vec::new(),
            categories: Vec::new(),
            opening: None,
            form: EntryForm::default(),
            note_editor: text_editor::Content::new(),
            category_menu: None,
            search: String::new(),
            settings: SettingsForm::default(),
            manager: CategoryManager::default(),
            history: History::default(),
        };
        account.reload()?;
        account.reset_settings(i18n);
        Ok(account)
    }

    /// Reads everything back from the file after a change.
    pub fn reload(&mut self) -> Result<()> {
        self.entries = self.ledger.entries()?;
        self.categories = self.ledger.categories()?;
        self.opening = self.ledger.opening_balance()?;
        if let Some(id) = self.form.category
            && !self.categories.iter().any(|c| c.id == id)
        {
            self.form.category = None;
        }
        if let Some(id) = self.form.editing
            && !self.entries.iter().any(|e| e.id == id)
        {
            self.form = EntryForm::default();
            self.sync_note();
        }
        Ok(())
    }

    /// Rebuilds the note editor after the form is replaced at once.
    pub fn sync_note(&mut self) {
        self.note_editor = text_editor::Content::with_text(&self.form.note);
    }

    pub fn reset_settings(&mut self, i18n: &I18n) {
        let s = &mut self.settings;
        s.name = self.info.name.clone();
        s.password.clear();
        s.password_again.clear();
        s.opening_amount = self
            .opening
            .map(|o| i18n.amount(o.amount, self.currency))
            .unwrap_or_default();
        s.opening_date = self.opening.map(|o| o.date);
        s.confirm_remove = false;
    }

    pub fn category(&self, id: Option<CategoryId>) -> Option<&Category> {
        let id = id?;
        self.categories.iter().find(|c| c.id == id)
    }

    pub fn category_name(&self, id: Option<CategoryId>) -> Option<&str> {
        self.category(id).map(|c| c.name.as_str())
    }

    /// Rows of the category menu: matching categories, plus a final "create"
    /// row when the typed name is new.
    pub fn menu_rows(&self) -> (Vec<&Category>, bool) {
        (
            self.form.suggestions(&self.categories),
            self.form.can_create_category(&self.categories),
        )
    }

    pub fn menu_len(&self) -> usize {
        let (items, create) = self.menu_rows();
        items.len() + usize::from(create)
    }
}
