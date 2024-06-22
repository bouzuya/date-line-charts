#[derive(Debug, thiserror::Error)]
#[error("error")]
pub struct Error;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EventStreamId(uuid::Uuid);

impl EventStreamId {
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl std::str::FromStr for EventStreamId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = uuid::Uuid::parse_str(s).map_err(|_| Error)?;
        if uuid.get_version_num() != 4 {
            return Err(Error);
        }
        Ok(Self(uuid))
    }
}

impl std::fmt::Display for EventStreamId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.to_string().fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use super::*;

    #[test]
    fn test_generate() {
        for _ in 0..100 {
            assert_ne!(EventStreamId::generate(), EventStreamId::generate());
        }
    }

    #[test]
    fn test_string_convesion() -> anyhow::Result<()> {
        let s = "67051e1b-fc32-43c8-899f-e2c73a1319f4";
        assert_eq!(EventStreamId::from_str(s)?.to_string(), s);
        let s = "00000000-0000-0000-0000-000000000000";
        assert_eq!(s, uuid::Uuid::nil().to_string());
        assert!(EventStreamId::from_str(s).is_err());
        Ok(())
    }
}
