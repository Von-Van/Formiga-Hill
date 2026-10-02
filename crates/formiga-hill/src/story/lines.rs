//! A package's words, kept apart from its scenes so a story can be translated without touching
//! what happens in it.
//!
//! A localisation file is TOML. A line is either plain text or a table with `text` and a variant
//! for any temperament that says it differently; tables without `text` are just namespaces, so
//! `[spread]` then `hello = "…"` is the line `spread.hello`. `{role}` in a line is replaced by the
//! name of whoever plays that role.
//!
//! ```toml
//! [spread]
//! intro = "Someone has left a picnic blanket on the green."
//! hello = { text = "Look, {friend}! A whole blanket.", grump = "Hmph. A blanket. It will do." }
//! ```

use formiga_core::TemperamentKind;
use std::collections::BTreeMap;
use toml::Value;

pub const MAX_LINES: usize = 2000;
pub const MAX_LINE_CHARS: usize = 280;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub text: String,
    /// Keyed by temperament, as `kind_key` writes it.
    pub variants: BTreeMap<String, String>,
}

impl Line {
    /// The words for a speaker of this temperament.
    pub fn for_kind(&self, kind: Option<TemperamentKind>) -> &str {
        kind.and_then(|kind| self.variants.get(kind_key(kind)))
            .unwrap_or(&self.text)
    }

    /// Every `{role}` any version of the line names.
    pub fn placeholders(&self) -> Vec<String> {
        std::iter::once(&self.text)
            .chain(self.variants.values())
            .flat_map(|text| placeholders(text))
            .collect()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Lines {
    lines: BTreeMap<String, Line>,
}

impl Lines {
    pub fn parse(text: &str) -> Result<Self, String> {
        let table: toml::Table = toml::from_str(text).map_err(|error| error.to_string())?;
        let mut lines = BTreeMap::new();
        flatten("", &table, &mut lines)?;
        if lines.len() > MAX_LINES {
            return Err(format!("more than {MAX_LINES} lines"));
        }
        Ok(Self { lines })
    }

    pub fn get(&self, key: &str) -> Option<&Line> {
        self.lines.get(key)
    }
}

fn flatten(
    prefix: &str,
    table: &toml::Table,
    lines: &mut BTreeMap<String, Line>,
) -> Result<(), String> {
    for (name, value) in table {
        let key = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}.{name}")
        };
        match value {
            Value::String(text) => {
                check(&key, text)?;
                lines.insert(
                    key,
                    Line {
                        text: text.clone(),
                        variants: BTreeMap::new(),
                    },
                );
            }
            Value::Table(inner) if inner.contains_key("text") => {
                let mut line = Line {
                    text: String::new(),
                    variants: BTreeMap::new(),
                };
                for (part, value) in inner {
                    let Value::String(text) = value else {
                        return Err(format!("{key}.{part} is not text"));
                    };
                    check(&key, text)?;
                    if part == "text" {
                        line.text = text.clone();
                    } else if KINDS.contains(&part.as_str()) {
                        line.variants.insert(part.clone(), text.clone());
                    } else {
                        return Err(format!(
                            "{key} has a version for \"{part}\", which is not a temperament ({})",
                            KINDS.join(", ")
                        ));
                    }
                }
                lines.insert(key, line);
            }
            Value::Table(inner) => flatten(&key, inner, lines)?,
            _ => return Err(format!("{key} is not text")),
        }
    }
    Ok(())
}

fn check(key: &str, text: &str) -> Result<(), String> {
    if text.trim().is_empty() || text.chars().count() > MAX_LINE_CHARS {
        return Err(format!(
            "{key} must be between 1 and {MAX_LINE_CHARS} characters"
        ));
    }
    if text.chars().any(char::is_control) {
        return Err(format!(
            "{key} has a control character or a line break in it"
        ));
    }
    let open = text.matches('{').count();
    if open != text.matches('}').count() || open != placeholders(text).len() {
        return Err(format!("{key} has a stray brace; names go in as {{role}}"));
    }
    Ok(())
}

/// The `{role}` names in `text`.
fn placeholders(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find('{') {
        let after = &rest[start + 1..];
        let Some(end) = after.find('}') else {
            break;
        };
        let name = &after[..end];
        if !name.is_empty()
            && name
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            found.push(name.to_owned());
        }
        rest = &after[end + 1..];
    }
    found
}

/// Fills in every `{role}` with a name. `None` if any of them names a role nobody plays, so a
/// line about someone who did not come is never shown half-finished.
pub fn fill(text: &str, name_of: impl Fn(&str) -> Option<String>) -> Option<String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let end = after.find('}')?;
        out.push_str(&name_of(&after[..end])?);
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Some(out)
}

/// The temperaments a line can have a version for, as authors write them.
pub const KINDS: [&str; 10] = [
    "sweetheart",
    "troublemaker",
    "grump",
    "explorer",
    "wallflower",
    "showoff",
    "scholar",
    "oddball",
    "lazybones",
    "guardian",
];

pub fn kind_key(kind: TemperamentKind) -> &'static str {
    match kind {
        TemperamentKind::Sweetheart => "sweetheart",
        TemperamentKind::Troublemaker => "troublemaker",
        TemperamentKind::Grump => "grump",
        TemperamentKind::Explorer => "explorer",
        TemperamentKind::Wallflower => "wallflower",
        TemperamentKind::Showoff => "showoff",
        TemperamentKind::Scholar => "scholar",
        TemperamentKind::Oddball => "oddball",
        TemperamentKind::Lazybones => "lazybones",
        TemperamentKind::Guardian => "guardian",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
        title = "The First Picnic"
        [spread]
        intro = "A blanket on the green."
        hello = { text = "Look, {friend}!", grump = "Hmph. It will do." }
    "#;

    #[test]
    fn namespaces_flatten_and_variants_follow_temperament() {
        let lines = Lines::parse(SAMPLE).unwrap();
        assert_eq!(lines.get("title").unwrap().text, "The First Picnic");
        let hello = lines.get("spread.hello").unwrap();
        assert_eq!(
            hello.for_kind(Some(TemperamentKind::Grump)),
            "Hmph. It will do."
        );
        assert_eq!(
            hello.for_kind(Some(TemperamentKind::Sweetheart)),
            "Look, {friend}!"
        );
        assert_eq!(hello.for_kind(None), "Look, {friend}!");
        assert_eq!(hello.placeholders(), vec!["friend".to_owned()]);
    }

    #[test]
    fn a_misspelt_temperament_is_caught() {
        let error = Lines::parse(r#"hi = { text = "Hi", grumpy = "Hmph" }"#).unwrap_err();
        assert!(error.contains("grumpy"));
    }

    #[test]
    fn text_that_could_break_a_layout_is_refused() {
        assert!(Lines::parse("hi = \"two\\nlines\"").is_err());
        assert!(Lines::parse("hi = \"a {stray brace\"").is_err());
        assert!(Lines::parse(&format!("hi = \"{}\"", "x".repeat(MAX_LINE_CHARS + 1))).is_err());
        assert!(Lines::parse("hi = 3").is_err());
    }

    #[test]
    fn names_fill_in_and_a_missing_one_holds_the_line_back() {
        let name = |role: &str| (role == "host").then(|| "Poppy".to_owned());
        assert_eq!(
            fill("Well done, {host}.", name).as_deref(),
            Some("Well done, Poppy.")
        );
        assert_eq!(fill("{host} and {shy}", name), None);
    }
}
