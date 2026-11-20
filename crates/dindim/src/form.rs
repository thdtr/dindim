//! The quick entry form, free of any UI code.
//!
//! Built for the keyboard: `45.90`, Tab, `gro`, Tab, `milk`, Enter. Text typed
//! in the category field and left without picking counts as the best match.

use dindim_core::text::fold;
use dindim_core::{
    AmountError, Category, CategoryId, Currency, Date, Entry, EntryId, Kind, NewEntry,
};
use dindim_i18n::I18n;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CategoryMatch<'a> {
    /// Nothing typed: the entry has no category.
    Empty,
    Found(&'a Category),
    NotFound,
}

/// Categories whose name contains `typed`, best matches first: exact name,
/// then names that start with it, then a word that starts with it, then the rest.
pub fn rank<'a>(typed: &str, options: &[&'a Category]) -> Vec<&'a Category> {
    let typed = fold(typed);
    if typed.is_empty() {
        return options.to_vec();
    }
    let mut scored: Vec<(u8, &Category)> = options
        .iter()
        .filter_map(|c| {
            let name = fold(&c.name);
            let score = if name == typed {
                0
            } else if name.starts_with(&typed) {
                1
            } else if name.split_whitespace().any(|w| w.starts_with(&typed)) {
                2
            } else if name.contains(&typed) {
                3
            } else {
                return None;
            };
            Some((score, *c))
        })
        .collect();
    scored.sort_by_key(|(score, _)| *score);
    scored.into_iter().map(|(_, c)| c).collect()
}

pub fn match_category<'a>(typed: &str, options: &[&'a Category]) -> CategoryMatch<'a> {
    if fold(typed).is_empty() {
        return CategoryMatch::Empty;
    }
    rank(typed, options)
        .first()
        .map_or(CategoryMatch::NotFound, |c| CategoryMatch::Found(c))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormError {
    Amount(AmountError),
    AmountNotPositive,
    UnknownCategory(String),
}

impl FormError {
    pub fn message(&self, i18n: &I18n) -> String {
        match self {
            FormError::Amount(e) => crate::errors::amount(i18n, *e),
            FormError::AmountNotPositive => i18n.t("error-amount-not-positive"),
            FormError::UnknownCategory(name) => i18n.t_with(
                "error-unknown-category",
                &dindim_i18n::args! { "name" => name.as_str() },
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryForm {
    pub kind: Kind,
    pub amount: String,
    pub category: Option<CategoryId>,
    /// Text typed in the category field without picking an item.
    pub category_query: String,
    pub note: String,
    /// `None` means today.
    pub date: Option<Date>,
    pub editing: Option<EntryId>,
    pub confirm_delete: bool,
    pub error: Option<FormError>,
}

impl Default for EntryForm {
    fn default() -> Self {
        EntryForm {
            kind: Kind::Expense,
            amount: String::new(),
            category: None,
            category_query: String::new(),
            note: String::new(),
            date: None,
            editing: None,
            confirm_delete: false,
            error: None,
        }
    }
}

impl EntryForm {
    /// Categories that fit the chosen kind, in alphabetical order.
    pub fn options<'a>(&self, categories: &'a [Category]) -> Vec<&'a Category> {
        let mut options: Vec<&Category> = match self.kind.category_kind() {
            Some(kind) => categories.iter().filter(|c| c.kind == kind).collect(),
            None => Vec::new(),
        };
        options.sort_by_cached_key(|c| fold(&c.name));
        options
    }

    /// Items shown in the category menu for what is typed now.
    pub fn suggestions<'a>(&self, categories: &'a [Category]) -> Vec<&'a Category> {
        rank(&self.category_query, &self.options(categories))
    }

    /// Whether the menu should offer to create a category from the typed text.
    pub fn can_create_category(&self, categories: &[Category]) -> bool {
        let typed = fold(&self.category_query);
        !typed.is_empty()
            && self.kind.category_kind().is_some()
            && !self
                .options(categories)
                .iter()
                .any(|c| fold(&c.name) == typed)
    }

    pub fn matched<'a>(&self, categories: &'a [Category]) -> CategoryMatch<'a> {
        if self.kind.category_kind().is_none() {
            return CategoryMatch::Empty;
        }
        let options = self.options(categories);
        if let Some(id) = self.category
            && let Some(c) = options.iter().find(|c| c.id == id)
        {
            return CategoryMatch::Found(c);
        }
        match_category(&self.category_query, &options)
    }

    pub fn pick_category(&mut self, id: CategoryId) {
        self.category = Some(id);
        self.category_query.clear();
        self.error = None;
    }

    /// When leaving the category field, typed text that matches a category
    /// becomes the chosen one.
    pub fn resolve_query(&mut self, categories: &[Category]) {
        if self.category_query.trim().is_empty() {
            self.category_query.clear();
            return;
        }
        if let CategoryMatch::Found(c) =
            match_category(&self.category_query, &self.options(categories))
        {
            let id = c.id;
            self.pick_category(id);
        }
    }

    pub fn set_kind(&mut self, kind: Kind) {
        if self.kind.category_kind() != kind.category_kind() {
            self.category = None;
            self.category_query.clear();
        }
        self.kind = kind;
        self.error = None;
    }

    pub fn date_or(&self, today: Date) -> Date {
        self.date.unwrap_or(today)
    }

    pub fn build(
        &self,
        categories: &[Category],
        today: Date,
        i18n: &I18n,
        currency: Currency,
    ) -> Result<NewEntry, FormError> {
        let amount = i18n
            .parse_amount(&self.amount, currency)
            .map_err(FormError::Amount)?;
        if !amount.is_positive() {
            return Err(FormError::AmountNotPositive);
        }
        let category_id = match self.matched(categories) {
            CategoryMatch::Empty => None,
            CategoryMatch::Found(c) => Some(c.id),
            CategoryMatch::NotFound => {
                return Err(FormError::UnknownCategory(
                    self.category_query.trim().to_string(),
                ));
            }
        };
        Ok(NewEntry {
            date: self.date_or(today),
            kind: self.kind,
            amount,
            category_id,
            note: self.note.trim().to_string(),
        })
    }

    pub fn load(entry: &Entry, categories: &[Category], i18n: &I18n, currency: Currency) -> Self {
        let category = entry
            .category_id
            .filter(|id| categories.iter().any(|c| c.id == *id));
        EntryForm {
            kind: entry.kind,
            amount: i18n.amount(entry.amount, currency),
            category,
            note: entry.note.clone(),
            date: Some(entry.date),
            editing: Some(entry.id),
            ..EntryForm::default()
        }
    }

    /// After saving, clears what changes with every purchase and keeps the
    /// kind and the date for the next one.
    pub fn after_save(&mut self) {
        let keep_date = if self.editing.is_some() {
            None
        } else {
            self.date
        };
        *self = EntryForm {
            kind: self.kind,
            date: keep_date,
            ..EntryForm::default()
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dindim_core::{CategoryKind, Money};
    use dindim_i18n::Locale;

    fn cats() -> Vec<Category> {
        let c = |id, name: &str, kind| Category {
            id,
            name: name.into(),
            kind,
        };
        vec![
            c(1, "Groceries", CategoryKind::Expense),
            c(2, "Health & Fitness", CategoryKind::Expense),
            c(3, "Home bills", CategoryKind::Expense),
            c(4, "Gas", CategoryKind::Expense),
            c(5, "Salary", CategoryKind::Income),
            c(6, "Café", CategoryKind::Expense),
        ]
    }

    fn today() -> Date {
        Date::constant(2026, 11, 19)
    }

    fn i18n() -> I18n {
        I18n::new(Locale::EnUs)
    }

    fn build(form: &EntryForm) -> Result<NewEntry, FormError> {
        form.build(&cats(), today(), &i18n(), Currency::USD)
    }

    fn found(typed: &str, kind: Kind) -> Option<i64> {
        let form = EntryForm {
            kind,
            category_query: typed.into(),
            ..Default::default()
        };
        match form.matched(&cats()) {
            CategoryMatch::Found(c) => Some(c.id),
            _ => None,
        }
    }

    #[test]
    fn matches_typed_text_loosely() {
        assert_eq!(found("groceries", Kind::Expense), Some(1));
        assert_eq!(found("g", Kind::Expense), Some(4));
        assert_eq!(found("gr", Kind::Expense), Some(1));
        assert_eq!(found("fit", Kind::Expense), Some(2));
        assert_eq!(found("bill", Kind::Expense), Some(3));
        assert_eq!(found("cafe", Kind::Expense), Some(6));
        assert_eq!(found("ocer", Kind::Expense), Some(1));
        assert_eq!(found("sal", Kind::Expense), None);
        assert_eq!(found("sal", Kind::Income), Some(5));
    }

    #[test]
    fn suggestions_are_ranked_and_creation_is_offered_for_new_names() {
        let cats = cats();
        let mut form = EntryForm::default();
        let names = |f: &EntryForm| -> Vec<String> {
            f.suggestions(&cats)
                .iter()
                .map(|c| c.name.clone())
                .collect()
        };
        assert_eq!(names(&form).len(), 5);
        assert!(!form.can_create_category(&cats));
        form.category_query = "h".into();
        assert_eq!(names(&form), ["Health & Fitness", "Home bills"]);
        assert!(form.can_create_category(&cats));
        form.category_query = "gas".into();
        assert!(!form.can_create_category(&cats));
        form.set_kind(Kind::ReserveDeposit);
        form.category_query = "anything".into();
        assert!(!form.can_create_category(&cats));
    }

    #[test]
    fn leaving_the_field_resolves_typed_text() {
        let cats = cats();
        let mut form = EntryForm {
            category_query: "groc".into(),
            ..Default::default()
        };
        form.resolve_query(&cats);
        assert_eq!(form.category, Some(1));
        assert!(form.category_query.is_empty());

        let mut form = EntryForm {
            category_query: "xyz".into(),
            ..Default::default()
        };
        form.resolve_query(&cats);
        assert_eq!(form.category, None);
        assert_eq!(form.category_query, "xyz");
    }

    #[test]
    fn builds_a_valid_entry() {
        let yesterday = today().yesterday().unwrap();
        let form = EntryForm {
            amount: "45.90".into(),
            category: Some(1),
            note: "  milk  ".into(),
            date: Some(yesterday),
            ..Default::default()
        };
        let entry = build(&form).unwrap();
        assert_eq!(entry.amount, Money::from_minor(4590));
        assert_eq!(entry.category_id, Some(1));
        assert_eq!(entry.note, "milk");
        assert_eq!(entry.date, yesterday);
    }

    #[test]
    fn explains_what_is_wrong() {
        let base = EntryForm {
            amount: "10".into(),
            ..Default::default()
        };
        assert_eq!(
            build(&EntryForm {
                amount: String::new(),
                ..base.clone()
            }),
            Err(FormError::Amount(AmountError::Empty))
        );
        assert_eq!(
            build(&EntryForm {
                amount: "0".into(),
                ..base.clone()
            }),
            Err(FormError::AmountNotPositive)
        );
        assert_eq!(
            build(&EntryForm {
                category_query: "xyz".into(),
                ..base
            }),
            Err(FormError::UnknownCategory("xyz".into()))
        );
    }

    #[test]
    fn reserve_ignores_category_and_switching_kind_clears_it() {
        let form = EntryForm {
            kind: Kind::ReserveDeposit,
            amount: "200".into(),
            category_query: "whatever".into(),
            ..Default::default()
        };
        assert_eq!(build(&form).unwrap().category_id, None);

        let mut form = EntryForm {
            category: Some(1),
            ..Default::default()
        };
        form.set_kind(Kind::Expense);
        assert_eq!(form.category, Some(1));
        form.set_kind(Kind::Income);
        assert_eq!(form.category, None);
    }

    #[test]
    fn load_and_after_save() {
        let entry = Entry {
            id: 7,
            date: Date::constant(2026, 11, 2),
            kind: Kind::Expense,
            amount: Money::from_minor(123_456),
            category_id: Some(2),
            note: "vitamins".into(),
        };
        let mut form = EntryForm::load(&entry, &cats(), &i18n(), Currency::USD);
        assert_eq!(form.amount, "1,234.56");
        assert_eq!(build(&form).unwrap().with_id(7), entry);

        form.after_save();
        assert_eq!(form.editing, None);
        assert!(form.amount.is_empty() && form.date.is_none());

        let mut quick = EntryForm {
            amount: "5".into(),
            date: Some(entry.date),
            kind: Kind::Income,
            ..Default::default()
        };
        quick.after_save();
        assert_eq!(quick.date, Some(entry.date), "date carries over");
        assert_eq!(quick.kind, Kind::Income);
    }
}
