use dindim_core::{Category, Entry, EntryId};
use dindim_store::{Ledger, Result};

const LIMIT: usize = 200;

/// A change that can be undone and redone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    EntryAdded(Entry),
    EntryUpdated {
        before: Entry,
        after: Entry,
    },
    EntryDeleted(Entry),
    CategoryAdded(Category),
    CategoryRenamed {
        before: Category,
        after: Category,
    },
    CategoryDeleted {
        category: Category,
        entries: Vec<EntryId>,
    },
}

impl Change {
    fn undo(&self, ledger: &mut Ledger) -> Result<()> {
        match self {
            Change::EntryAdded(entry) => ledger.delete_entry(entry.id),
            Change::EntryUpdated { before, .. } => ledger.update_entry(before).map(drop),
            Change::EntryDeleted(entry) => ledger.restore_entry(entry),
            Change::CategoryAdded(category) => ledger.delete_category(category.id).map(drop),
            Change::CategoryRenamed { before, .. } => {
                ledger.rename_category(before.id, &before.name).map(drop)
            }
            Change::CategoryDeleted { category, entries } => {
                ledger.restore_category(category, entries)
            }
        }
    }

    fn redo(&self, ledger: &mut Ledger) -> Result<()> {
        match self {
            Change::EntryAdded(entry) => ledger.restore_entry(entry),
            Change::EntryUpdated { after, .. } => ledger.update_entry(after).map(drop),
            Change::EntryDeleted(entry) => ledger.delete_entry(entry.id),
            Change::CategoryAdded(category) => ledger.restore_category(category, &[]),
            Change::CategoryRenamed { after, .. } => {
                ledger.rename_category(after.id, &after.name).map(drop)
            }
            Change::CategoryDeleted { category, .. } => {
                ledger.delete_category(category.id).map(drop)
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct History {
    undo: Vec<Change>,
    redo: Vec<Change>,
}

impl History {
    pub fn record(&mut self, change: Change) {
        self.redo.clear();
        self.undo.push(change);
        if self.undo.len() > LIMIT {
            self.undo.remove(0);
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }

    /// Reverts the latest change. `None` when there is nothing to undo.
    pub fn undo(&mut self, ledger: &mut Ledger) -> Option<Result<()>> {
        let change = self.undo.pop()?;
        Some(match change.undo(ledger) {
            Ok(()) => {
                self.redo.push(change);
                Ok(())
            }
            Err(e) => {
                self.clear();
                Err(e)
            }
        })
    }

    pub fn redo(&mut self, ledger: &mut Ledger) -> Option<Result<()>> {
        let change = self.redo.pop()?;
        Some(match change.redo(ledger) {
            Ok(()) => {
                self.undo.push(change);
                Ok(())
            }
            Err(e) => {
                self.clear();
                Err(e)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dindim_core::{CategoryKind, Currency, Date, Kind, Money, NewEntry};

    fn new_entry(category_id: Option<i64>) -> NewEntry {
        NewEntry {
            date: Date::constant(2026, 11, 19),
            kind: Kind::Expense,
            amount: Money::from_minor(500),
            category_id,
            note: "lunch".into(),
        }
    }

    #[test]
    fn undoes_and_redoes_entry_changes() {
        let mut ledger = Ledger::in_memory(Currency::USD).unwrap();
        let mut history = History::default();

        let added = ledger.add_entry(&new_entry(None)).unwrap();
        history.record(Change::EntryAdded(added.clone()));
        let mut edited = added.clone();
        edited.amount = Money::from_minor(900);
        ledger.update_entry(&edited).unwrap();
        history.record(Change::EntryUpdated {
            before: added.clone(),
            after: edited.clone(),
        });

        history.undo(&mut ledger).unwrap().unwrap();
        assert_eq!(ledger.entries().unwrap(), vec![added.clone()]);
        history.undo(&mut ledger).unwrap().unwrap();
        assert!(ledger.entries().unwrap().is_empty());
        assert!(history.undo(&mut ledger).is_none());

        history.redo(&mut ledger).unwrap().unwrap();
        history.redo(&mut ledger).unwrap().unwrap();
        assert_eq!(ledger.entries().unwrap(), vec![edited]);
        assert!(!history.can_redo());
    }

    #[test]
    fn undoing_a_category_delete_brings_its_entries_back() {
        let mut ledger = Ledger::in_memory(Currency::USD).unwrap();
        let mut history = History::default();
        let rent = ledger.add_category("Rent", CategoryKind::Expense).unwrap();
        let entry = ledger.add_entry(&new_entry(Some(rent.id))).unwrap();

        let entries = ledger.delete_category(rent.id).unwrap();
        history.record(Change::CategoryDeleted {
            category: rent.clone(),
            entries,
        });
        history.undo(&mut ledger).unwrap().unwrap();
        assert_eq!(ledger.entries().unwrap()[0].category_id, Some(rent.id));

        history.redo(&mut ledger).unwrap().unwrap();
        assert_eq!(ledger.entries().unwrap()[0].id, entry.id);
        assert_eq!(ledger.entries().unwrap()[0].category_id, None);
    }

    #[test]
    fn a_new_change_clears_redo() {
        let mut ledger = Ledger::in_memory(Currency::USD).unwrap();
        let mut history = History::default();
        let a = ledger.add_entry(&new_entry(None)).unwrap();
        history.record(Change::EntryAdded(a));
        history.undo(&mut ledger).unwrap().unwrap();
        assert!(history.can_redo());
        let b = ledger.add_entry(&new_entry(None)).unwrap();
        history.record(Change::EntryAdded(b));
        assert!(!history.can_redo());
    }
}
