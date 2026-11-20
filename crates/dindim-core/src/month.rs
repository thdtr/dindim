use jiff::civil::Date;

/// A calendar month, the unit every report is built around.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct YearMonth {
    year: i16,
    month: i8,
}

impl YearMonth {
    pub fn new(year: i16, month: i8) -> Option<Self> {
        Date::new(year, month, 1)
            .ok()
            .map(|_| YearMonth { year, month })
    }

    pub fn of(date: Date) -> Self {
        YearMonth {
            year: date.year(),
            month: date.month(),
        }
    }

    pub fn year(self) -> i16 {
        self.year
    }

    /// From 1 (January) to 12 (December).
    pub fn month(self) -> i8 {
        self.month
    }

    pub fn first_day(self) -> Date {
        Date::constant(self.year, self.month, 1)
    }

    pub fn last_day(self) -> Date {
        self.first_day().last_of_month()
    }

    pub fn contains(self, date: Date) -> bool {
        YearMonth::of(date) == self
    }

    pub fn next(self) -> Self {
        if self.month == 12 {
            YearMonth::new(self.year.saturating_add(1), 1).unwrap_or(self)
        } else {
            YearMonth {
                month: self.month + 1,
                ..self
            }
        }
    }

    pub fn prev(self) -> Self {
        if self.month == 1 {
            YearMonth::new(self.year.saturating_sub(1), 12).unwrap_or(self)
        } else {
            YearMonth {
                month: self.month - 1,
                ..self
            }
        }
    }

    /// The same month in another year, if that year is representable.
    pub fn with_year(self, year: i16) -> Option<Self> {
        YearMonth::new(year, self.month)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date;

    fn ym(y: i16, m: i8) -> YearMonth {
        YearMonth::new(y, m).unwrap()
    }

    #[test]
    fn rejects_invalid_months() {
        assert!(YearMonth::new(2026, 0).is_none());
        assert!(YearMonth::new(2026, 13).is_none());
    }

    #[test]
    fn navigates_across_years() {
        assert_eq!(ym(2026, 12).next(), ym(2027, 1));
        assert_eq!(ym(2027, 1).prev(), ym(2026, 12));
        assert_eq!(ym(2026, 5).next().prev(), ym(2026, 5));
    }

    #[test]
    fn knows_first_and_last_day() {
        assert_eq!(ym(2024, 2).last_day(), date(2024, 2, 29));
        assert_eq!(ym(2026, 12).last_day(), date(2026, 12, 31));
        assert_eq!(ym(2026, 3).first_day(), date(2026, 3, 1));
    }

    #[test]
    fn contains_only_its_own_days() {
        let m = ym(2026, 11);
        assert!(m.contains(date(2026, 11, 30)));
        assert!(!m.contains(date(2026, 12, 1)));
        assert!(!m.contains(date(2025, 11, 15)));
    }
}
