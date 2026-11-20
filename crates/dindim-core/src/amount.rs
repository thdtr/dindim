//! Reading amounts typed by people and writing them back as plain numbers.

use thiserror::Error;

use crate::{Currency, Money};

/// Decimal and digit group separators of a locale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumberStyle {
    pub decimal: char,
    pub group: char,
}

impl NumberStyle {
    pub const POINT: NumberStyle = NumberStyle {
        decimal: '.',
        group: ',',
    };
    pub const COMMA: NumberStyle = NumberStyle {
        decimal: ',',
        group: '.',
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum AmountError {
    #[error("amount is empty")]
    Empty,
    #[error("amount contains invalid characters")]
    Invalid,
    #[error("amount has more decimal places than the currency allows")]
    TooManyDecimals,
    #[error("amount is too large")]
    Overflow,
}

const SYMBOLS: &str = "$€£¥₹₩";

/// Parses `12`, `1,234.56`, `$9.90` or sums like `10 + 5.50`.
///
/// Without the locale decimal separator, a single group separator followed by
/// up to `minor_units` digits is read as a decimal point, so `12,50` works in
/// a `POINT` locale and `12.50` works in a `COMMA` one.
pub fn parse(input: &str, currency: Currency, style: NumberStyle) -> Result<Money, AmountError> {
    if input.trim().is_empty() {
        return Err(AmountError::Empty);
    }
    input
        .split('+')
        .try_fold(0_i64, |acc, term| {
            let value = parse_term(term, currency, style)?;
            acc.checked_add(value).ok_or(AmountError::Overflow)
        })
        .map(Money::from_minor)
}

fn strip_symbol(term: &str) -> (&str, bool) {
    let symbol_len: usize = term
        .chars()
        .take_while(|c| c.is_alphabetic() || SYMBOLS.contains(*c))
        .take(4)
        .map(char::len_utf8)
        .sum();
    (term[symbol_len..].trim_start(), symbol_len > 0)
}

fn parse_term(raw: &str, currency: Currency, style: NumberStyle) -> Result<i64, AmountError> {
    let (s, had_symbol) = strip_symbol(raw.trim());
    if s.is_empty() {
        return Err(if had_symbol {
            AmountError::Invalid
        } else {
            AmountError::Empty
        });
    }
    if !s
        .chars()
        .all(|c| c.is_ascii_digit() || c == '.' || c == ',')
    {
        return Err(AmountError::Invalid);
    }

    let minor_units = usize::from(currency.minor_units);
    let (int_part, frac_part) = if let Some(pos) = s.rfind(style.decimal) {
        (&s[..pos], &s[pos + style.decimal.len_utf8()..])
    } else if let Some(pos) = s.rfind(style.group) {
        let frac = &s[pos + style.group.len_utf8()..];
        let single = s.matches(style.group).count() == 1;
        if single && minor_units > 0 && (1..=minor_units).contains(&frac.len()) {
            (&s[..pos], frac)
        } else {
            (s, "")
        }
    } else {
        (s, "")
    };

    if int_part.contains(style.decimal) || frac_part.contains(['.', ',']) {
        return Err(AmountError::Invalid);
    }
    if frac_part.len() > minor_units {
        return Err(AmountError::TooManyDecimals);
    }
    let digits: String = int_part.chars().filter(char::is_ascii_digit).collect();
    if digits.is_empty() && frac_part.is_empty() {
        return Err(AmountError::Invalid);
    }

    let major: i64 = if digits.is_empty() {
        0
    } else {
        digits.parse().map_err(|_| AmountError::Overflow)?
    };
    let minor: i64 = if frac_part.is_empty() {
        0
    } else {
        let padded = format!("{frac_part:0<minor_units$}");
        padded.parse().map_err(|_| AmountError::Invalid)?
    };
    major
        .checked_mul(currency.scale())
        .and_then(|v| v.checked_add(minor))
        .ok_or(AmountError::Overflow)
}

/// Writes `money` without a currency symbol, ready to go back into a text
/// field: `1,234.56` or `1.234,56`.
pub fn format(money: Money, currency: Currency, style: NumberStyle) -> String {
    let abs = money.minor().unsigned_abs();
    let scale = currency.scale().unsigned_abs();
    let sign = if money.is_negative() { "-" } else { "" };
    let major = group_thousands(abs / scale, style.group);
    if currency.minor_units == 0 {
        format!("{sign}{major}")
    } else {
        let width = usize::from(currency.minor_units);
        format!("{sign}{major}{}{:0width$}", style.decimal, abs % scale)
    }
}

fn group_thousands(n: u64, separator: char) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(separator);
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const JPY: Currency = Currency {
        code: "JPY",
        minor_units: 0,
    };

    fn point(s: &str) -> Result<i64, AmountError> {
        parse(s, Currency::USD, NumberStyle::POINT).map(Money::minor)
    }

    fn comma(s: &str) -> Result<i64, AmountError> {
        parse(s, Currency::EUR, NumberStyle::COMMA).map(Money::minor)
    }

    #[test]
    fn reads_point_decimal_input() {
        assert_eq!(point("12"), Ok(1200));
        assert_eq!(point("12.5"), Ok(1250));
        assert_eq!(point(".99"), Ok(99));
        assert_eq!(point("12."), Ok(1200));
        assert_eq!(point("1,234.56"), Ok(123_456));
        assert_eq!(point("1,234,567.89"), Ok(123_456_789));
        assert_eq!(point("  $ 9.90 "), Ok(990));
        assert_eq!(point("US$9"), Ok(900));
    }

    #[test]
    fn reads_comma_decimal_input() {
        assert_eq!(comma("12,50"), Ok(1250));
        assert_eq!(comma("1.234,56"), Ok(123_456));
        assert_eq!(comma("€ 3"), Ok(300));
    }

    #[test]
    fn accepts_the_other_convention_when_unambiguous() {
        assert_eq!(point("12,50"), Ok(1250));
        assert_eq!(point("1,234"), Ok(123_400));
        assert_eq!(comma("12.5"), Ok(1250));
        assert_eq!(comma("1.234"), Ok(123_400));
    }

    #[test]
    fn adds_terms() {
        assert_eq!(point("10 + 5.50"), Ok(1550));
        assert_eq!(point("1+1+1"), Ok(300));
        assert_eq!(point("10 +"), Err(AmountError::Empty));
    }

    #[test]
    fn rejects_bad_input() {
        assert_eq!(point(""), Err(AmountError::Empty));
        assert_eq!(point("abc"), Err(AmountError::Invalid));
        assert_eq!(point("hello"), Err(AmountError::Invalid));
        assert_eq!(point("-5"), Err(AmountError::Invalid));
        assert_eq!(point("1.2.3"), Err(AmountError::Invalid));
        assert_eq!(point("1.234"), Err(AmountError::TooManyDecimals));
        assert_eq!(point("1.2,3"), Err(AmountError::Invalid));
        assert_eq!(point("."), Err(AmountError::Invalid));
        assert_eq!(point("99999999999999999999"), Err(AmountError::Overflow));
    }

    #[test]
    fn respects_currencies_without_decimals() {
        let yen = |s| parse(s, JPY, NumberStyle::POINT).map(Money::minor);
        assert_eq!(yen("1,500"), Ok(1500));
        assert_eq!(yen("12.5"), Err(AmountError::TooManyDecimals));
        assert_eq!(
            format(Money::from_minor(1500), JPY, NumberStyle::POINT),
            "1,500"
        );
    }

    #[test]
    fn formats_with_grouping() {
        let f = |m, s| format(Money::from_minor(m), Currency::USD, s);
        assert_eq!(f(0, NumberStyle::POINT), "0.00");
        assert_eq!(f(5, NumberStyle::POINT), "0.05");
        assert_eq!(f(123_456, NumberStyle::POINT), "1,234.56");
        assert_eq!(f(100_000_000, NumberStyle::POINT), "1,000,000.00");
        assert_eq!(f(-1990, NumberStyle::POINT), "-19.90");
        assert_eq!(f(123_456, NumberStyle::COMMA), "1.234,56");
    }

    proptest! {
        #[test]
        fn formatting_round_trips(minor in 0_i64..1_000_000_000_000, comma_style in any::<bool>()) {
            let style = if comma_style { NumberStyle::COMMA } else { NumberStyle::POINT };
            let money = Money::from_minor(minor);
            prop_assert_eq!(parse(&format(money, Currency::USD, style), Currency::USD, style), Ok(money));
        }

        #[test]
        fn parsing_never_panics(input in "\\PC{0,40}") {
            let _ = parse(&input, Currency::USD, NumberStyle::POINT);
            let _ = parse(&input, JPY, NumberStyle::COMMA);
        }
    }
}
