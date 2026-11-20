use jiff::civil::Date;

use crate::Money;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Expense,
    Income,
    /// Money moved into the emergency fund.
    ReserveDeposit,
    /// Money taken out of the emergency fund.
    ReserveWithdrawal,
}

impl Kind {
    pub const ALL: [Kind; 4] = [
        Kind::Expense,
        Kind::Income,
        Kind::ReserveDeposit,
        Kind::ReserveWithdrawal,
    ];

    /// Stable identifier used in storage and CSV files.
    pub fn code(self) -> &'static str {
        match self {
            Kind::Expense => "expense",
            Kind::Income => "income",
            Kind::ReserveDeposit => "reserve_deposit",
            Kind::ReserveWithdrawal => "reserve_withdrawal",
        }
    }

    pub fn from_code(code: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|k| k.code() == code)
    }

    pub fn category_kind(self) -> Option<CategoryKind> {
        match self {
            Kind::Expense => Some(CategoryKind::Expense),
            Kind::Income => Some(CategoryKind::Income),
            Kind::ReserveDeposit | Kind::ReserveWithdrawal => None,
        }
    }

    pub fn is_reserve(self) -> bool {
        self.category_kind().is_none()
    }

    /// Effect on available cash: income and withdrawals add, the rest subtracts.
    pub fn cash_effect(self, amount: Money) -> Money {
        match self {
            Kind::Income | Kind::ReserveWithdrawal => amount,
            Kind::Expense | Kind::ReserveDeposit => -amount,
        }
    }

    /// Effect on the emergency fund balance.
    pub fn reserve_effect(self, amount: Money) -> Money {
        match self {
            Kind::ReserveDeposit => amount,
            Kind::ReserveWithdrawal => -amount,
            Kind::Expense | Kind::Income => Money::ZERO,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CategoryKind {
    Expense,
    Income,
}

impl CategoryKind {
    pub fn code(self) -> &'static str {
        match self {
            CategoryKind::Expense => "expense",
            CategoryKind::Income => "income",
        }
    }

    pub fn from_code(code: &str) -> Option<CategoryKind> {
        match code {
            "expense" => Some(CategoryKind::Expense),
            "income" => Some(CategoryKind::Income),
            _ => None,
        }
    }
}

pub type CategoryId = i64;
pub type EntryId = i64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Category {
    pub id: CategoryId,
    pub name: String,
    pub kind: CategoryKind,
}

/// Cash on hand when the account started being tracked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpeningBalance {
    pub date: Date,
    pub amount: Money,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub id: EntryId,
    pub date: Date,
    pub kind: Kind,
    /// Always positive; the direction comes from `kind`.
    pub amount: Money,
    pub category_id: Option<CategoryId>,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewEntry {
    pub date: Date,
    pub kind: Kind,
    pub amount: Money,
    pub category_id: Option<CategoryId>,
    pub note: String,
}

impl NewEntry {
    pub fn with_id(self, id: EntryId) -> Entry {
        Entry {
            id,
            date: self.date,
            kind: self.kind,
            amount: self.amount,
            category_id: self.category_id,
            note: self.note,
        }
    }
}

impl From<Entry> for NewEntry {
    fn from(entry: Entry) -> Self {
        NewEntry {
            date: entry.date,
            kind: entry.kind,
            amount: entry.amount,
            category_id: entry.category_id,
            note: entry.note,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_round_trip() {
        for kind in Kind::ALL {
            assert_eq!(Kind::from_code(kind.code()), Some(kind));
        }
        assert_eq!(Kind::from_code("???"), None);
        for kind in [CategoryKind::Expense, CategoryKind::Income] {
            assert_eq!(CategoryKind::from_code(kind.code()), Some(kind));
        }
    }

    #[test]
    fn effects_have_the_right_sign() {
        let m = Money::from_minor(1000);
        assert_eq!(Kind::Income.cash_effect(m), m);
        assert_eq!(Kind::ReserveWithdrawal.cash_effect(m), m);
        assert_eq!(Kind::Expense.cash_effect(m), -m);
        assert_eq!(Kind::ReserveDeposit.cash_effect(m), -m);
        assert_eq!(Kind::ReserveDeposit.reserve_effect(m), m);
        assert_eq!(Kind::ReserveWithdrawal.reserve_effect(m), -m);
        assert_eq!(Kind::Expense.reserve_effect(m), Money::ZERO);
    }

    #[test]
    fn only_expense_and_income_have_categories() {
        assert_eq!(Kind::Expense.category_kind(), Some(CategoryKind::Expense));
        assert_eq!(Kind::Income.category_kind(), Some(CategoryKind::Income));
        assert!(Kind::ReserveDeposit.is_reserve());
        assert!(!Kind::Income.is_reserve());
    }
}
