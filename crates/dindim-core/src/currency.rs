/// An ISO 4217 currency and how many decimal places its amounts use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Currency {
    pub code: &'static str,
    pub minor_units: u8,
}

impl Currency {
    pub const USD: Currency = Currency::new("USD", 2);
    pub const BRL: Currency = Currency::new("BRL", 2);
    pub const EUR: Currency = Currency::new("EUR", 2);

    pub const ALL: &'static [Currency] = &[
        Currency::USD,
        Currency::EUR,
        Currency::GBP,
        Currency::BRL,
        Currency::CAD,
        Currency::AUD,
        Currency::NZD,
        Currency::JPY,
        Currency::CHF,
        Currency::CNY,
        Currency::INR,
        Currency::KRW,
        Currency::MXN,
        Currency::ARS,
        Currency::CLP,
        Currency::COP,
        Currency::PEN,
        Currency::UYU,
        Currency::ZAR,
        Currency::SEK,
        Currency::NOK,
        Currency::DKK,
        Currency::PLN,
    ];

    const GBP: Currency = Currency::new("GBP", 2);
    const CAD: Currency = Currency::new("CAD", 2);
    const AUD: Currency = Currency::new("AUD", 2);
    const NZD: Currency = Currency::new("NZD", 2);
    const JPY: Currency = Currency::new("JPY", 0);
    const CHF: Currency = Currency::new("CHF", 2);
    const CNY: Currency = Currency::new("CNY", 2);
    const INR: Currency = Currency::new("INR", 2);
    const KRW: Currency = Currency::new("KRW", 0);
    const MXN: Currency = Currency::new("MXN", 2);
    const ARS: Currency = Currency::new("ARS", 2);
    const CLP: Currency = Currency::new("CLP", 0);
    const COP: Currency = Currency::new("COP", 2);
    const PEN: Currency = Currency::new("PEN", 2);
    const UYU: Currency = Currency::new("UYU", 2);
    const ZAR: Currency = Currency::new("ZAR", 2);
    const SEK: Currency = Currency::new("SEK", 2);
    const NOK: Currency = Currency::new("NOK", 2);
    const DKK: Currency = Currency::new("DKK", 2);
    const PLN: Currency = Currency::new("PLN", 2);

    const fn new(code: &'static str, minor_units: u8) -> Self {
        Currency { code, minor_units }
    }

    pub fn from_code(code: &str) -> Option<Currency> {
        Currency::ALL
            .iter()
            .copied()
            .find(|c| c.code.eq_ignore_ascii_case(code))
    }

    /// How many minor units make one major unit, e.g. 100 cents in a dollar.
    pub fn scale(self) -> i64 {
        10_i64.pow(u32::from(self.minor_units))
    }
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn looks_up_codes_case_insensitively() {
        assert_eq!(Currency::from_code("usd"), Some(Currency::USD));
        assert_eq!(Currency::from_code("JPY").map(|c| c.minor_units), Some(0));
        assert_eq!(Currency::from_code("XYZ"), None);
    }

    #[test]
    fn codes_are_unique() {
        for (i, a) in Currency::ALL.iter().enumerate() {
            assert!(Currency::ALL[i + 1..].iter().all(|b| a.code != b.code));
        }
    }

    #[test]
    fn scale_follows_minor_units() {
        assert_eq!(Currency::USD.scale(), 100);
        assert_eq!(Currency::from_code("JPY").unwrap().scale(), 1);
    }
}
