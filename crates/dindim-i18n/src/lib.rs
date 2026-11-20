//! Translations (Fluent) and locale-aware formatting of money and dates.

mod currency;

use dindim_core::{AmountError, Currency, Date, Money, NumberStyle, YearMonth, amount};
use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentResource, FluentValue};
use unic_langid::{LanguageIdentifier, langid};

pub use fluent_bundle::FluentArgs;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Locale {
    EnUs,
    PtBr,
}

impl Locale {
    pub const ALL: [Locale; 2] = [Locale::EnUs, Locale::PtBr];

    pub fn tag(self) -> &'static str {
        match self {
            Locale::EnUs => "en-US",
            Locale::PtBr => "pt-BR",
        }
    }

    /// Accepts tags like `pt-BR`, `pt_BR.UTF-8` or plain `pt`.
    pub fn from_tag(tag: &str) -> Option<Locale> {
        let lang = tag.split(['-', '_', '.', '@']).next()?.to_ascii_lowercase();
        match lang.as_str() {
            "en" => Some(Locale::EnUs),
            "pt" => Some(Locale::PtBr),
            _ => None,
        }
    }

    /// The first supported locale among the user's preferred ones.
    pub fn negotiate<I, S>(preferred: I) -> Locale
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        preferred
            .into_iter()
            .find_map(|tag| Locale::from_tag(tag.as_ref()))
            .unwrap_or(Locale::EnUs)
    }

    /// The language name written in itself, for language pickers.
    pub fn native_name(self) -> &'static str {
        match self {
            Locale::EnUs => "English (US)",
            Locale::PtBr => "Português (Brasil)",
        }
    }

    pub fn number_style(self) -> NumberStyle {
        match self {
            Locale::EnUs => NumberStyle::POINT,
            Locale::PtBr => NumberStyle::COMMA,
        }
    }

    pub fn default_currency(self) -> Currency {
        match self {
            Locale::EnUs => Currency::USD,
            Locale::PtBr => Currency::BRL,
        }
    }

    fn langid(self) -> LanguageIdentifier {
        match self {
            Locale::EnUs => langid!("en-US"),
            Locale::PtBr => langid!("pt-BR"),
        }
    }

    fn source(self) -> &'static str {
        match self {
            Locale::EnUs => include_str!("../locales/en-US/main.ftl"),
            Locale::PtBr => include_str!("../locales/pt-BR/main.ftl"),
        }
    }
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.native_name())
    }
}

/// Builds `FluentArgs` from `name => value` pairs.
#[macro_export]
macro_rules! args {
    ($($name:literal => $value:expr),* $(,)?) => {{
        #[allow(unused_mut)]
        let mut args = $crate::FluentArgs::new();
        $(args.set($name, $crate::value($value));)*
        args
    }};
}

/// Converts common Rust values into Fluent values.
pub fn value<'a>(v: impl IntoValue<'a>) -> FluentValue<'a> {
    v.into_value()
}

pub trait IntoValue<'a> {
    fn into_value(self) -> FluentValue<'a>;
}

impl<'a> IntoValue<'a> for &'a str {
    fn into_value(self) -> FluentValue<'a> {
        FluentValue::from(self)
    }
}

impl<'a> IntoValue<'a> for String {
    fn into_value(self) -> FluentValue<'a> {
        FluentValue::from(self)
    }
}

impl<'a> IntoValue<'a> for &'a String {
    fn into_value(self) -> FluentValue<'a> {
        FluentValue::from(self.as_str())
    }
}

macro_rules! numeric_value {
    ($($t:ty),*) => {$(
        impl<'a> IntoValue<'a> for $t {
            fn into_value(self) -> FluentValue<'a> {
                FluentValue::from(self as i64)
            }
        }
    )*};
}
numeric_value!(i8, i16, i32, i64, u8, u16, u32, usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    Plus,
    Minus,
}

pub struct I18n {
    locale: Locale,
    bundle: FluentBundle<FluentResource>,
}

impl std::fmt::Debug for I18n {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("I18n")
            .field("locale", &self.locale)
            .finish_non_exhaustive()
    }
}

impl I18n {
    pub fn new(locale: Locale) -> Self {
        let resource = FluentResource::try_new(locale.source().to_string())
            .unwrap_or_else(|(resource, _)| resource);
        let mut bundle = FluentBundle::new_concurrent(vec![locale.langid()]);
        bundle.set_use_isolating(false);
        let _ = bundle.add_resource(resource);
        I18n { locale, bundle }
    }

    pub fn locale(&self) -> Locale {
        self.locale
    }

    pub fn has(&self, id: &str) -> bool {
        self.bundle.has_message(id)
    }

    /// The translated message, or its id when the message is missing.
    pub fn t(&self, id: &str) -> String {
        self.format(id, None)
    }

    pub fn t_with(&self, id: &str, args: &FluentArgs) -> String {
        self.format(id, Some(args))
    }

    fn format(&self, id: &str, args: Option<&FluentArgs>) -> String {
        let Some(pattern) = self.bundle.get_message(id).and_then(|m| m.value()) else {
            return id.to_string();
        };
        let mut errors = Vec::new();
        self.bundle
            .format_pattern(pattern, args, &mut errors)
            .into_owned()
    }

    // Numbers and money

    pub fn currency_symbol(&self, currency: Currency) -> &'static str {
        currency::symbol(self.locale, currency)
    }

    /// `1,234.56` in English, `1.234,56` in Portuguese.
    pub fn amount(&self, money: Money, currency: Currency) -> String {
        amount::format(money, currency, self.locale.number_style())
    }

    pub fn parse_amount(&self, input: &str, currency: Currency) -> Result<Money, AmountError> {
        amount::parse(input, currency, self.locale.number_style())
    }

    /// `$1,234.56`, `-$5.00`, `R$ 1.234,56`.
    pub fn money(&self, money: Money, currency: Currency) -> String {
        let sign = if money.is_negative() { "-" } else { "" };
        let number = self.amount(money.abs(), currency);
        let symbol = self.currency_symbol(currency);
        let gap = match self.locale {
            Locale::PtBr => "\u{a0}",
            Locale::EnUs if symbol.ends_with(char::is_alphabetic) => "\u{a0}",
            Locale::EnUs => "",
        };
        format!("{sign}{symbol}{gap}{number}")
    }

    /// `+ $12.00` or `− $12.00`, for lists where direction matters.
    pub fn money_signed(&self, money: Money, currency: Currency, sign: Sign) -> String {
        let mark = match sign {
            Sign::Plus => '+',
            Sign::Minus => '\u{2212}',
        };
        format!("{mark}\u{a0}{}", self.money(money.abs(), currency))
    }

    // Dates

    pub fn month_name(&self, month: i8) -> String {
        self.t_with("month-name", &args! { "month" => month })
    }

    pub fn month_short(&self, month: i8) -> String {
        self.t_with("month-short", &args! { "month" => month })
    }

    /// `November 2026`, `novembro de 2026`.
    pub fn month_year(&self, ym: YearMonth) -> String {
        self.t_with(
            "date-month-year",
            &args! { "month" => self.month_name(ym.month()), "year" => ym.year().to_string() },
        )
    }

    /// `November 19, 2026`, `19 de novembro de 2026`.
    pub fn date_long(&self, date: Date) -> String {
        self.t_with(
            "date-long",
            &args! {
                "month" => self.month_name(date.month()),
                "day" => date.day().to_string(),
                "year" => date.year().to_string(),
            },
        )
    }

    /// `11/19/2026`, `19/11/2026`.
    pub fn date_numeric(&self, date: Date) -> String {
        match self.locale {
            Locale::EnUs => format!("{}/{}/{}", date.month(), date.day(), date.year()),
            Locale::PtBr => format!("{:02}/{:02}/{}", date.day(), date.month(), date.year()),
        }
    }

    /// `Today`, `Yesterday`, `Thu, Nov 13`, or with the year when it differs.
    pub fn date_relative(&self, date: Date, today: Date) -> String {
        if date == today {
            return self.t("date-today");
        }
        if today.yesterday().ok() == Some(date) {
            return self.t("date-yesterday");
        }
        let weekday = self.t_with(
            "weekday-short",
            &args! { "day" => date.weekday().to_sunday_zero_offset() },
        );
        let base = args! {
            "weekday" => weekday,
            "month" => self.month_short(date.month()),
            "month-number" => format!("{:02}", date.month()),
            "day" => date.day().to_string(),
            "day-padded" => format!("{:02}", date.day()),
            "year" => date.year().to_string(),
        };
        if date.year() == today.year() {
            self.t_with("date-short", &base)
        } else {
            self.t_with("date-short-year", &base)
        }
    }

    /// One-letter weekday headers for a calendar, starting on Sunday.
    pub fn weekday_initials(&self) -> [String; 7] {
        std::array::from_fn(|day| self.t_with("weekday-initial", &args! { "day" => day }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fluent_syntax::ast;
    use jiff::civil::date;
    use std::collections::BTreeSet;

    fn ids(locale: Locale) -> BTreeSet<String> {
        let resource = FluentResource::try_new(locale.source().to_string())
            .unwrap_or_else(|(_, errors)| panic!("{} does not parse: {errors:?}", locale.tag()));
        resource
            .entries()
            .filter_map(|entry| match entry {
                ast::Entry::Message(m) => Some(m.id.name.to_string()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn every_locale_has_the_same_messages() {
        let reference = ids(Locale::EnUs);
        assert!(!reference.is_empty());
        for locale in Locale::ALL {
            let found = ids(locale);
            let missing: Vec<_> = reference.difference(&found).collect();
            let extra: Vec<_> = found.difference(&reference).collect();
            assert!(
                missing.is_empty() && extra.is_empty(),
                "{}: missing {missing:?}, extra {extra:?}",
                locale.tag()
            );
        }
    }

    #[test]
    fn negotiates_locales() {
        assert_eq!(Locale::negotiate(["pt_BR.UTF-8"]), Locale::PtBr);
        assert_eq!(Locale::negotiate(["fr-FR", "pt"]), Locale::PtBr);
        assert_eq!(Locale::negotiate(["de-DE"]), Locale::EnUs);
        assert_eq!(Locale::negotiate(Vec::<String>::new()), Locale::EnUs);
    }

    #[test]
    fn formats_money_per_locale() {
        let en = I18n::new(Locale::EnUs);
        let pt = I18n::new(Locale::PtBr);
        let m = Money::from_minor(123_456);
        assert_eq!(en.money(m, Currency::USD), "$1,234.56");
        assert_eq!(en.money(-m, Currency::USD), "-$1,234.56");
        assert_eq!(pt.money(m, Currency::BRL), "R$\u{a0}1.234,56");
        assert_eq!(pt.money(m, Currency::USD), "US$\u{a0}1.234,56");
        assert_eq!(
            en.money(m, Currency::from_code("CHF").unwrap()),
            "CHF\u{a0}1,234.56"
        );
        assert_eq!(
            en.money_signed(m, Currency::USD, Sign::Minus),
            "\u{2212}\u{a0}$1,234.56"
        );
        assert_eq!(pt.parse_amount("1.234,56", Currency::BRL), Ok(m));
    }

    #[test]
    fn formats_dates_per_locale() {
        let today = date(2026, 11, 19);
        for locale in Locale::ALL {
            let i18n = I18n::new(locale);
            assert_ne!(i18n.date_relative(today, today), "date-today");
            assert!(i18n.date_long(today).contains("2026"));
            assert!(!i18n.month_year(YearMonth::of(today)).contains("month"));
            assert_eq!(i18n.weekday_initials().len(), 7);
            for month in 1..=12 {
                assert!(!i18n.month_name(month).is_empty());
                assert!(!i18n.month_short(month).is_empty());
            }
        }
        let en = I18n::new(Locale::EnUs);
        assert_eq!(en.date_numeric(today), "11/19/2026");
        assert_eq!(en.date_long(today), "November 19, 2026");
        assert_eq!(en.date_relative(date(2026, 11, 13), today), "Fri, Nov 13");
        assert_eq!(en.date_relative(date(2025, 11, 13), today), "Nov 13, 2025");
        assert_eq!(I18n::new(Locale::PtBr).date_numeric(today), "19/11/2026");
    }

    #[test]
    fn missing_messages_fall_back_to_their_id() {
        assert_eq!(
            I18n::new(Locale::EnUs).t("no-such-message"),
            "no-such-message"
        );
    }
}
