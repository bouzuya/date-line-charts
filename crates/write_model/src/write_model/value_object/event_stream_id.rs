#[derive(Debug, thiserror::Error)]
#[error("error")]
pub struct Error;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EventStreamId(String);

impl AsRef<str> for EventStreamId {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl std::str::FromStr for EventStreamId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() <= 100 && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            Ok(Self(s.to_owned()))
        } else {
            Err(Error)
        }
    }
}

impl std::fmt::Display for EventStreamId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use super::*;

    #[test]
    fn test_string_convesion() -> anyhow::Result<()> {
        let s = "67051e1b-fc32-43c8-899f-e2c73a1319f4";
        assert_eq!(EventStreamId::from_str(s)?.to_string(), s);
        Ok(())
    }
}
