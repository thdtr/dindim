use crate::text::fold;
use crate::{Entry, Money};

/// A search over entries. Text matches the note or the category name,
/// ignoring case and accents. An amount, when the query reads as one, matches
/// entries of exactly that value.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntryQuery {
    text: String,
    amount: Option<Money>,
}

impl EntryQuery {
    pub fn new(text: &str, amount: Option<Money>) -> Self {
        EntryQuery {
            text: fold(text),
            amount,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn matches(&self, entry: &Entry, category_name: Option<&str>) -> bool {
        if self.text.is_empty() {
            return true;
        }
        self.amount == Some(entry.amount)
            || fold(&entry.note).contains(&self.text)
            || category_name.is_some_and(|name| fold(name).contains(&self.text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Kind;
    use jiff::civil::date;

    fn entry(note: &str, minor: i64) -> Entry {
        Entry {
            id: 1,
            date: date(2026, 11, 19),
            kind: Kind::Expense,
            amount: Money::from_minor(minor),
            category_id: Some(1),
            note: note.into(),
        }
    }

    #[test]
    fn matches_note_category_and_amount() {
        let coffee = entry("Coffee with Zoë", 450);
        assert!(EntryQuery::new("zoe", None).matches(&coffee, None));
        assert!(EntryQuery::new("dining", None).matches(&coffee, Some("Dining out")));
        assert!(EntryQuery::new("4.50", Some(Money::from_minor(450))).matches(&coffee, None));
        assert!(!EntryQuery::new("rent", None).matches(&coffee, Some("Groceries")));
        assert!(EntryQuery::new("  ", None).matches(&coffee, None));
    }
}
