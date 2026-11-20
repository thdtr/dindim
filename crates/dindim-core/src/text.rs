use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

/// Lowercases and strips accents so `Café` and `cafe` compare equal.
pub fn fold(text: &str) -> String {
    text.trim()
        .nfd()
        .filter(|c| !is_combining_mark(*c))
        .flat_map(char::to_lowercase)
        .collect()
}

/// Collapses runs of whitespace into single spaces and trims the ends.
pub fn squish(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// `"Mary's Café"` becomes `"marys-cafe"`. ASCII letters, digits and hyphens only.
pub fn slugify(text: &str) -> String {
    let mut out = String::new();
    for c in fold(text).chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if (c.is_whitespace() || c == '-' || c == '_' || c == '.' || c == '/')
            && !out.is_empty()
            && !out.ends_with('-')
        {
            out.push('-');
        }
    }
    out.trim_end_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_case_and_accents() {
        assert_eq!(fold("  Café Crème "), "cafe creme");
        assert_eq!(fold("NAÏVE"), "naive");
        assert_eq!(fold("Zoë"), "zoe");
    }

    #[test]
    fn squishes_whitespace() {
        assert_eq!(squish("  milk \n and   bread "), "milk and bread");
    }

    #[test]
    fn slugifies() {
        assert_eq!(slugify("Mary's Café"), "marys-cafe");
        assert_eq!(slugify("  Home & Family!! "), "home-family");
        assert_eq!(slugify("../../etc"), "etc");
        assert_eq!(slugify("???"), "");
    }
}
