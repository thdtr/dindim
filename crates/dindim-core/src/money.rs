use std::iter::Sum;
use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

/// An amount of money in the minor units of the account currency (cents for
/// USD). Integers keep every sum exact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Money(i64);

impl Money {
    pub const ZERO: Money = Money(0);

    pub const fn from_minor(units: i64) -> Self {
        Money(units)
    }

    pub const fn minor(self) -> i64 {
        self.0
    }

    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    pub const fn is_negative(self) -> bool {
        self.0 < 0
    }

    pub const fn is_positive(self) -> bool {
        self.0 > 0
    }

    pub const fn abs(self) -> Self {
        Money(self.0.abs())
    }

    /// Rounded percentage (0 to 100) of `total` that `self` represents.
    pub fn percent_of(self, total: Money) -> u8 {
        if total.0 <= 0 || self.0 <= 0 {
            return 0;
        }
        let pct = (i128::from(self.0) * 100 + i128::from(total.0) / 2) / i128::from(total.0);
        pct.clamp(0, 100) as u8
    }
}

impl Add for Money {
    type Output = Money;
    fn add(self, rhs: Money) -> Money {
        Money(self.0 + rhs.0)
    }
}

impl AddAssign for Money {
    fn add_assign(&mut self, rhs: Money) {
        self.0 += rhs.0;
    }
}

impl Sub for Money {
    type Output = Money;
    fn sub(self, rhs: Money) -> Money {
        Money(self.0 - rhs.0)
    }
}

impl SubAssign for Money {
    fn sub_assign(&mut self, rhs: Money) {
        self.0 -= rhs.0;
    }
}

impl Neg for Money {
    type Output = Money;
    fn neg(self) -> Money {
        Money(-self.0)
    }
}

impl Sum for Money {
    fn sum<I: Iterator<Item = Money>>(iter: I) -> Money {
        iter.fold(Money::ZERO, Add::add)
    }
}

impl<'a> Sum<&'a Money> for Money {
    fn sum<I: Iterator<Item = &'a Money>>(iter: I) -> Money {
        iter.copied().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic_is_exact() {
        let a = Money::from_minor(1000);
        let b = Money::from_minor(250);
        assert_eq!((a + b).minor(), 1250);
        assert_eq!((a - b).minor(), 750);
        assert_eq!((-a).minor(), -1000);
        assert_eq!([a, b].iter().sum::<Money>().minor(), 1250);
    }

    #[test]
    fn percent_is_rounded_and_safe() {
        let total = Money::from_minor(30_000);
        assert_eq!(Money::from_minor(10_000).percent_of(total), 33);
        assert_eq!(Money::from_minor(20_000).percent_of(total), 67);
        assert_eq!(Money::from_minor(1_000).percent_of(Money::ZERO), 0);
        assert_eq!(Money::from_minor(-1_000).percent_of(total), 0);
    }
}
