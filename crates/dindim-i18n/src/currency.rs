use dindim_core::Currency;

use crate::Locale;

/// Currency symbols as CLDR writes them for each locale. Codes without a
/// local symbol fall back to the ISO code.
pub fn symbol(locale: Locale, currency: Currency) -> &'static str {
    let table: &[(&str, &str)] = match locale {
        Locale::EnUs => &[
            ("USD", "$"),
            ("EUR", "€"),
            ("GBP", "£"),
            ("BRL", "R$"),
            ("CAD", "CA$"),
            ("AUD", "A$"),
            ("NZD", "NZ$"),
            ("JPY", "¥"),
            ("CNY", "CN¥"),
            ("INR", "₹"),
            ("KRW", "₩"),
            ("MXN", "MX$"),
        ],
        Locale::PtBr => &[
            ("USD", "US$"),
            ("EUR", "€"),
            ("GBP", "£"),
            ("BRL", "R$"),
            ("CAD", "CA$"),
            ("AUD", "AU$"),
            ("NZD", "NZ$"),
            ("JPY", "JP¥"),
            ("CNY", "CN¥"),
            ("INR", "₹"),
            ("KRW", "₩"),
            ("MXN", "MX$"),
        ],
    };
    table
        .iter()
        .find(|(code, _)| *code == currency.code)
        .map_or(currency.code, |(_, symbol)| symbol)
}
