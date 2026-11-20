use std::collections::HashMap;

use crate::model::{CategoryId, Entry, Kind, OpeningBalance};
use crate::{Money, YearMonth};

/// Share of a category in the total of its kind. `category` is `None` for
/// entries without a category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CategoryShare {
    pub category: Option<CategoryId>,
    pub total: Money,
    pub percent: u8,
}

/// The one-line story of a month.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Headline {
    NoEntries,
    NoIncome { expenses: Money },
    Surplus { amount: Money },
    BreakEven,
    Deficit { amount: Money },
}

impl Headline {
    pub fn tone(self) -> Tone {
        match self {
            Headline::Surplus { .. } => Tone::Good,
            Headline::NoEntries | Headline::BreakEven => Tone::Neutral,
            Headline::NoIncome { .. } | Headline::Deficit { .. } => Tone::Bad,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Good,
    Neutral,
    Bad,
}

/// How this month's spending compares to the previous month.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpendingChange {
    More(Money),
    Less(Money),
    Same,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonthReport {
    pub month: YearMonth,
    pub income: Money,
    pub expenses: Money,
    pub reserve_deposits: Money,
    pub reserve_withdrawals: Money,
    /// Emergency fund balance at the end of the month.
    pub reserve_balance: Money,
    pub expenses_by_category: Vec<CategoryShare>,
    pub income_by_category: Vec<CategoryShare>,
    /// Spending in the previous month, if it had any entries at all.
    pub previous_expenses: Option<Money>,
    /// Cash at the end of the month: opening balance plus every movement so far.
    pub balance: Money,
    pub entry_count: usize,
}

impl MonthReport {
    pub fn build(month: YearMonth, entries: &[Entry], opening: Option<OpeningBalance>) -> Self {
        let in_month: Vec<&Entry> = entries.iter().filter(|e| month.contains(e.date)).collect();
        let total_of = |kind: Kind| -> Money {
            in_month
                .iter()
                .filter(|e| e.kind == kind)
                .map(|e| e.amount)
                .sum()
        };

        let last_day = month.last_day();
        let until_month_end = || entries.iter().filter(move |e| e.date <= last_day);
        let reserve_balance = until_month_end()
            .map(|e| e.kind.reserve_effect(e.amount))
            .sum();
        let opening_amount = opening
            .filter(|o| o.date <= last_day)
            .map(|o| o.amount)
            .unwrap_or_default();
        let balance = opening_amount
            + until_month_end()
                .map(|e| e.kind.cash_effect(e.amount))
                .sum::<Money>();

        let prev = month.prev();
        let previous_expenses = entries.iter().any(|e| prev.contains(e.date)).then(|| {
            entries
                .iter()
                .filter(|e| e.kind == Kind::Expense && prev.contains(e.date))
                .map(|e| e.amount)
                .sum()
        });

        MonthReport {
            month,
            income: total_of(Kind::Income),
            expenses: total_of(Kind::Expense),
            reserve_deposits: total_of(Kind::ReserveDeposit),
            reserve_withdrawals: total_of(Kind::ReserveWithdrawal),
            reserve_balance,
            expenses_by_category: shares(&in_month, Kind::Expense),
            income_by_category: shares(&in_month, Kind::Income),
            previous_expenses,
            balance,
            entry_count: in_month.len(),
        }
    }

    /// Income minus expenses. The emergency fund is left out on purpose.
    pub fn left_over(&self) -> Money {
        self.income - self.expenses
    }

    /// Net amount moved into (positive) or out of (negative) the fund.
    pub fn reserve_net(&self) -> Money {
        self.reserve_deposits - self.reserve_withdrawals
    }

    pub fn headline(&self) -> Headline {
        let left = self.left_over();
        if self.entry_count == 0 {
            Headline::NoEntries
        } else if self.income.is_zero() && self.expenses.is_positive() {
            Headline::NoIncome {
                expenses: self.expenses,
            }
        } else if left.is_positive() {
            Headline::Surplus { amount: left }
        } else if left.is_zero() {
            Headline::BreakEven
        } else {
            Headline::Deficit { amount: left.abs() }
        }
    }

    /// Expenses as a rounded percentage of income, when there is both.
    pub fn spending_ratio(&self) -> Option<u32> {
        if !self.income.is_positive() || !self.expenses.is_positive() {
            return None;
        }
        let income = i128::from(self.income.minor());
        let pct = (i128::from(self.expenses.minor()) * 100 + income / 2) / income;
        u32::try_from(pct).ok()
    }

    pub fn spending_change(&self) -> Option<SpendingChange> {
        let diff = self.expenses - self.previous_expenses?;
        Some(if diff.is_positive() {
            SpendingChange::More(diff)
        } else if diff.is_negative() {
            SpendingChange::Less(diff.abs())
        } else {
            SpendingChange::Same
        })
    }
}

fn shares(entries: &[&Entry], kind: Kind) -> Vec<CategoryShare> {
    let mut totals: HashMap<Option<CategoryId>, Money> = HashMap::new();
    for e in entries.iter().filter(|e| e.kind == kind) {
        *totals.entry(e.category_id).or_default() += e.amount;
    }
    let grand_total: Money = totals.values().sum();
    let mut out: Vec<CategoryShare> = totals
        .into_iter()
        .map(|(category, total)| CategoryShare {
            category,
            total,
            percent: total.percent_of(grand_total),
        })
        .collect();
    out.sort_by(|a, b| {
        b.total
            .cmp(&a.total)
            .then_with(|| a.category.is_none().cmp(&b.category.is_none()))
            .then_with(|| a.category.cmp(&b.category))
    });
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonthTotals {
    pub month: YearMonth,
    pub income: Money,
    pub expenses: Money,
}

/// The twelve months of a year side by side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YearReport {
    pub year: i16,
    pub months: Vec<MonthTotals>,
}

impl YearReport {
    pub fn build(year: i16, entries: &[Entry]) -> Self {
        let months = (1..=12)
            .filter_map(|m| YearMonth::new(year, m))
            .map(|month| {
                let sum = |kind: Kind| -> Money {
                    entries
                        .iter()
                        .filter(|e| e.kind == kind && month.contains(e.date))
                        .map(|e| e.amount)
                        .sum()
                };
                MonthTotals {
                    month,
                    income: sum(Kind::Income),
                    expenses: sum(Kind::Expense),
                }
            })
            .collect();
        YearReport { year, months }
    }

    pub fn income(&self) -> Money {
        self.months.iter().map(|m| m.income).sum()
    }

    pub fn expenses(&self) -> Money {
        self.months.iter().map(|m| m.expenses).sum()
    }

    /// Largest monthly value, used to scale the chart.
    pub fn peak(&self) -> Money {
        self.months
            .iter()
            .flat_map(|m| [m.income, m.expenses])
            .max()
            .unwrap_or(Money::ZERO)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::{Date, date};

    fn e(id: i64, date: Date, kind: Kind, dollars: i64, category: Option<i64>) -> Entry {
        Entry {
            id,
            date,
            kind,
            amount: Money::from_minor(dollars * 100),
            category_id: category,
            note: String::new(),
        }
    }

    fn usd(dollars: i64) -> Money {
        Money::from_minor(dollars * 100)
    }

    fn nov() -> YearMonth {
        YearMonth::new(2026, 11).unwrap()
    }

    fn sample() -> Vec<Entry> {
        vec![
            e(1, date(2026, 10, 5), Kind::Expense, 500, Some(1)),
            e(2, date(2026, 10, 6), Kind::ReserveDeposit, 1000, None),
            e(3, date(2026, 11, 1), Kind::Income, 3000, Some(3)),
            e(4, date(2026, 11, 2), Kind::Expense, 600, Some(1)),
            e(5, date(2026, 11, 3), Kind::Expense, 300, Some(2)),
            e(6, date(2026, 11, 4), Kind::Expense, 100, None),
            e(7, date(2026, 11, 5), Kind::ReserveDeposit, 400, None),
            e(8, date(2026, 11, 6), Kind::ReserveWithdrawal, 100, None),
            e(9, date(2026, 12, 1), Kind::Expense, 999, Some(1)),
            e(10, date(2026, 12, 1), Kind::ReserveDeposit, 999, None),
        ]
    }

    #[test]
    fn totals_only_count_the_month() {
        let r = MonthReport::build(nov(), &sample(), None);
        assert_eq!(r.income, usd(3000));
        assert_eq!(r.expenses, usd(1000));
        assert_eq!(r.left_over(), usd(2000));
        assert_eq!(r.reserve_net(), usd(300));
        assert_eq!(r.entry_count, 6);
    }

    #[test]
    fn reserve_balance_accumulates_until_month_end() {
        let r = MonthReport::build(nov(), &sample(), None);
        assert_eq!(r.reserve_balance, usd(1300));
    }

    #[test]
    fn categories_are_ranked_with_percentages() {
        let r = MonthReport::build(nov(), &sample(), None);
        let got: Vec<_> = r
            .expenses_by_category
            .iter()
            .map(|s| (s.category, s.total, s.percent))
            .collect();
        assert_eq!(
            got,
            vec![
                (Some(1), usd(600), 60),
                (Some(2), usd(300), 30),
                (None, usd(100), 10)
            ]
        );
        assert_eq!(r.income_by_category.len(), 1);
        assert_eq!(r.income_by_category[0].percent, 100);
    }

    #[test]
    fn headline_tells_the_story() {
        let r = MonthReport::build(nov(), &sample(), None);
        assert_eq!(r.headline(), Headline::Surplus { amount: usd(2000) });
        assert_eq!(r.headline().tone(), Tone::Good);
        assert_eq!(r.spending_ratio(), Some(33));

        let over = [
            e(1, date(2026, 11, 1), Kind::Income, 100, None),
            e(2, date(2026, 11, 2), Kind::Expense, 150, None),
        ];
        assert_eq!(
            MonthReport::build(nov(), &over, None).headline(),
            Headline::Deficit { amount: usd(50) }
        );

        let no_income = [e(1, date(2026, 11, 2), Kind::Expense, 80, None)];
        assert_eq!(
            MonthReport::build(nov(), &no_income, None).headline(),
            Headline::NoIncome { expenses: usd(80) }
        );

        let even = [
            e(1, date(2026, 11, 1), Kind::Income, 100, None),
            e(2, date(2026, 11, 2), Kind::Expense, 100, None),
        ];
        assert_eq!(
            MonthReport::build(nov(), &even, None).headline(),
            Headline::BreakEven
        );

        let empty = MonthReport::build(nov(), &[], None);
        assert_eq!(empty.headline(), Headline::NoEntries);
        assert_eq!(empty.spending_ratio(), None);
    }

    #[test]
    fn compares_with_previous_month() {
        let r = MonthReport::build(nov(), &sample(), None);
        assert_eq!(r.spending_change(), Some(SpendingChange::More(usd(500))));

        let only_nov = [e(1, date(2026, 11, 2), Kind::Expense, 80, None)];
        assert_eq!(
            MonthReport::build(nov(), &only_nov, None).spending_change(),
            None
        );

        let less = [
            e(1, date(2026, 10, 2), Kind::Expense, 100, None),
            e(2, date(2026, 11, 2), Kind::Expense, 40, None),
        ];
        assert_eq!(
            MonthReport::build(nov(), &less, None).spending_change(),
            Some(SpendingChange::Less(usd(60)))
        );
    }

    #[test]
    fn balance_counts_opening_and_every_movement_until_month_end() {
        let opening = OpeningBalance {
            date: date(2026, 10, 1),
            amount: usd(100),
        };
        let r = MonthReport::build(nov(), &sample(), Some(opening));
        assert_eq!(r.balance, usd(300));

        let later = OpeningBalance {
            date: date(2027, 1, 1),
            ..opening
        };
        assert_eq!(
            MonthReport::build(nov(), &sample(), Some(later)).balance,
            usd(200)
        );
    }

    #[test]
    fn year_report_has_twelve_months() {
        let y = YearReport::build(2026, &sample());
        assert_eq!(y.months.len(), 12);
        assert_eq!(y.months[10].income, usd(3000));
        assert_eq!(y.months[9].expenses, usd(500));
        assert_eq!(y.expenses(), usd(2499));
        assert_eq!(y.income(), usd(3000));
        assert_eq!(y.peak(), usd(3000));
        assert_eq!(YearReport::build(2020, &sample()).peak(), Money::ZERO);
    }
}
