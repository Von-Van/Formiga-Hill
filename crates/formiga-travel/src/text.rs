//! The two shapes of text that may cross between the apps.

/// Text meant to be shown: not blank, no longer than `max_chars`, and free of control characters,
/// so nothing that crosses can break a line, a layout, or a log.
pub(crate) fn is_display_text(text: &str, max_chars: usize) -> bool {
    !text.trim().is_empty()
        && text.chars().count() <= max_chars
        && !text.chars().any(char::is_control)
}

/// A stable identifier, such as a content package's or an item's: lowercase ASCII letters,
/// digits, `.`, `-` and `_`, starting with a letter or digit, at most `max_len` bytes. Nothing in
/// it can name a path, a URL, or anything outside the namespace it is looked up in.
pub(crate) fn is_identifier(text: &str, max_len: usize) -> bool {
    let mut bytes = text.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    text.len() <= max_len
        && (first.is_ascii_lowercase() || first.is_ascii_digit())
        && bytes.all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_text_is_short_single_line_and_not_blank() {
        assert!(is_display_text("A picnic on the hilltop", 32));
        assert!(!is_display_text("", 32));
        assert!(!is_display_text("   ", 32));
        assert!(!is_display_text("two\nlines", 32));
        assert!(!is_display_text(&"x".repeat(33), 32));
    }

    #[test]
    fn identifiers_cannot_escape_their_namespace() {
        assert!(is_identifier("com.formiga.hill.first-story", 96));
        assert!(is_identifier("leaf_sled", 96));
        assert!(!is_identifier("", 96));
        assert!(!is_identifier("../colony.json", 96));
        assert!(!is_identifier("/etc/passwd", 96));
        assert!(!is_identifier("https://example.com", 96));
        assert!(!is_identifier("Capitalised", 96));
        assert!(!is_identifier(".hidden", 96));
        assert!(!is_identifier(&"a".repeat(97), 96));
    }
}
