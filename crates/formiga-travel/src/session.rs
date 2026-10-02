use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// One trip to the Hill and back. Desktop draws it when it calls the train; the snapshot, the
/// Hill session and the return receipt all carry it, so a receipt can only ever settle the trip
/// it was written for. 128 random bits, written as 32 lowercase hex digits.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SessionId([u8; 16]);

impl SessionId {
    pub fn random() -> Result<Self, getrandom::Error> {
        let mut bytes = [0; 16];
        getrandom::fill(&mut bytes)?;
        Ok(Self(bytes))
    }

    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// Reads the 32-digit hex form. Either case is accepted; [`fmt::Display`] writes lowercase.
    pub fn parse(text: &str) -> Option<Self> {
        if text.len() != 32 || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return None;
        }
        let mut bytes = [0; 16];
        for (byte, pair) in bytes.iter_mut().zip(text.as_bytes().chunks_exact(2)) {
            *byte = u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok()?;
        }
        Some(Self(bytes))
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|byte| write!(f, "{byte:02x}"))
    }
}

impl fmt::Debug for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SessionId({self})")
    }
}

impl Serialize for SessionId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for SessionId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text)
            .ok_or_else(|| serde::de::Error::custom("a session id is 32 hexadecimal digits"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_form_round_trips() {
        let id = SessionId::from_bytes([
            0x00, 0x01, 0x7f, 0x80, 0xff, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x8a, 0x9b,
            0xac, 0xbd,
        ]);
        let text = id.to_string();
        assert_eq!(text, "00017f80ff102030405060708a9bacbd");
        assert_eq!(SessionId::parse(&text), Some(id));
        assert_eq!(SessionId::parse(&text.to_uppercase()), Some(id));
    }

    #[test]
    fn malformed_ids_are_refused() {
        assert_eq!(SessionId::parse(""), None);
        assert_eq!(SessionId::parse(&"0".repeat(31)), None);
        assert_eq!(SessionId::parse(&"0".repeat(33)), None);
        assert_eq!(SessionId::parse(&format!("+f{}", "0".repeat(30))), None);
        assert_eq!(SessionId::parse(&format!("zz{}", "0".repeat(30))), None);
    }

    #[test]
    fn random_ids_differ() {
        let a = SessionId::random().expect("the OS has randomness");
        let b = SessionId::random().expect("the OS has randomness");
        assert_ne!(a, b);
    }
}
