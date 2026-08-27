const MIN_CHARS: usize = 3;
const MAX_CHARS: usize = 20;

const MULTIPLICATION_SIGN: char = '\u{D7}';
const DIVISION_SIGN: char = '\u{F7}';

fn is_allowed(c: char) -> bool {
    if c == '_' || c == '-' {
        return true;
    }
    if c.is_ascii_alphanumeric() {
        return true;
    }
    if c == MULTIPLICATION_SIGN || c == DIVISION_SIGN {
        return false;
    }
    matches!(c as u32, 0x00C0..=0x00FF | 0x0100..=0x017F)
}

pub fn validate_username(raw: &str) -> Result<String, &'static str> {
    let name = raw.trim();
    let count = name.chars().count();

    if count < MIN_CHARS || count > MAX_CHARS {
        return Err("Username must be between 3 and 20 characters");
    }
    if !name.chars().all(is_allowed) {
        return Err("Username may only contain Latin letters, digits, underscore and hyphen");
    }
    if !name.chars().next().is_some_and(|c| c.is_alphanumeric()) {
        return Err("Username must start with a letter or a digit");
    }

    Ok(name.to_string())
}

#[cfg(test)]
mod tests {
    use super::validate_username;

    #[test]
    fn accepts_the_supported_latin_alphabets() {
        for name in [
            "almeekel",
            "Đorđe-Žčć",
            "Jürgen_Weiß",
            "Muñoz",
            "Loïc-Aimé",
            "Niccolò",
        ] {
            assert_eq!(validate_username(name).as_deref(), Ok(name));
        }
    }

    #[test]
    fn rejects_other_scripts() {
        for name in ["\u{430}dmin", "\u{3b1}dmin", "\u{645}\u{631}\u{62d}\u{628}\u{627}", "\u{7528}\u{6237}\u{540d}"] {
            assert!(validate_username(name).is_err());
        }
    }

    #[test]
    fn rejects_symbols_spaces_and_markup() {
        for name in ["joueur\u{1F600}", "a b", "<script>", "a\u{D7}b", "a\u{F7}b"] {
            assert!(validate_username(name).is_err());
        }
    }

    #[test]
    fn counts_characters_not_bytes() {
        assert!(validate_username("Đorđe").is_ok());
        assert!(validate_username("éé").is_err());
        assert!(validate_username("ĐorđeĐorđeĐorđeĐorđeĐ").is_err());
    }

    #[test]
    fn returns_the_trimmed_value() {
        assert_eq!(validate_username("  almeekel  ").as_deref(), Ok("almeekel"));
        assert!(validate_username("  ab  ").is_err());
    }

    #[test]
    fn requires_an_alphanumeric_first_character() {
        assert!(validate_username("_admin").is_err());
        assert!(validate_username("-admin").is_err());
    }
}
