/// Normalize a phone number to E.164 format.
/// Handles common African phone number formats:
/// - 07012345678 → +2347012345678 (Nigeria)
/// - 2347012345678 → +2347012345678
/// - +2347012345678 → +2347012345678
/// - 07XX... (Kenya) → +254...
pub fn normalize(input: &str) -> String {
    let digits: String = input.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.is_empty() {
        return String::new();
    }

    // Already has country code
    if digits.starts_with("250") && digits.len() == 12 {
        return format!("+{}", digits); // Rwanda: +250XXXXXXXXX
    }
    if digits.starts_with("234") && digits.len() == 13 {
        return format!("+{}", digits); // Nigeria: +234XXXXXXXXXX
    }
    if digits.starts_with("254") && digits.len() == 12 {
        return format!("+{}", digits); // Kenya: +254XXXXXXXXX
    }
    if digits.starts_with("233") && digits.len() == 12 {
        return format!("+{}", digits); // Ghana: +233XXXXXXXXX
    }

    // Rwandan local format: 07XXXXXXXX (10 digits starting with 07)
    if digits.starts_with("07") && digits.len() == 10 {
        return format!("+250{}", &digits[1..]);
    }

    // Nigerian local format: 0XXXXXXXXXX (11 digits)
    if digits.starts_with('0') && digits.len() == 11 {
        return format!("+234{}", &digits[1..]);
    }

    // Kenyan local format: 0XXXXXXXXX (10 digits, non-7 prefix)
    if digits.starts_with('0') && digits.len() == 10 {
        return format!("+254{}", &digits[1..]);
    }

    // If it already starts with a +, preserve it
    if input.starts_with('+') {
        return format!("+{}", digits);
    }

    // Fallback: assume Rwandan without leading 0 (9 digits)
    if digits.len() == 9 && digits.starts_with('7') {
        return format!("+250{}", digits);
    }

    // Can't normalize — return as-is with +
    if digits.len() >= 10 {
        format!("+{}", digits)
    } else {
        String::new()
    }
}

/// Extract the country code from an E.164 phone number
pub fn country_code(phone: &str) -> &str {
    if phone.starts_with("+250") {
        "RW"
    } else if phone.starts_with("+234") {
        "NG"
    } else if phone.starts_with("+254") {
        "KE"
    } else if phone.starts_with("+233") {
        "GH"
    } else if phone.starts_with("+256") {
        "UG"
    } else if phone.starts_with("+255") {
        "TZ"
    } else if phone.starts_with("+27") {
        "ZA"
    } else {
        "UNKNOWN"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_rwandan_local() {
        assert_eq!(normalize("0781234567"), "+250781234567");
        assert_eq!(normalize("0731234567"), "+250731234567");
    }

    #[test]
    fn test_normalize_rwandan_with_country_code() {
        assert_eq!(normalize("250781234567"), "+250781234567");
        assert_eq!(normalize("+250781234567"), "+250781234567");
    }

    #[test]
    fn test_normalize_rwandan_no_zero() {
        assert_eq!(normalize("781234567"), "+250781234567");
    }

    #[test]
    fn test_normalize_nigerian_local() {
        assert_eq!(normalize("07012345678"), "+2347012345678");
        assert_eq!(normalize("08101234567"), "+2348101234567");
    }

    #[test]
    fn test_normalize_with_country_code() {
        assert_eq!(normalize("2347012345678"), "+2347012345678");
        assert_eq!(normalize("+2347012345678"), "+2347012345678");
    }

    #[test]
    fn test_normalize_empty() {
        assert_eq!(normalize(""), "");
        assert_eq!(normalize("abc"), "");
    }

    #[test]
    fn test_country_code() {
        assert_eq!(country_code("+250781234567"), "RW");
        assert_eq!(country_code("+2347012345678"), "NG");
        assert_eq!(country_code("+254712345678"), "KE");
        assert_eq!(country_code("+233201234567"), "GH");
    }
}
