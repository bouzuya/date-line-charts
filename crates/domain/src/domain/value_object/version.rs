#[derive(Debug, thiserror::Error)]
#[error("error")]
pub struct Error;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Version(u32);

impl Version {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(1)
    }
}

impl TryFrom<i64> for Version {
    type Error = Error;
    fn try_from(n: i64) -> Result<Self, Self::Error> {
        u32::try_from(n).map(Self).map_err(|_| Error)
    }
}

impl From<Version> for i64 {
    fn from(version: Version) -> Self {
        i64::from(version.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i64_conversion() -> anyhow::Result<()> {
        assert!(Version::try_from(-1_i64).is_err());
        assert!(Version::try_from(0_i64).is_ok());
        assert!(Version::try_from(1_i64).is_ok());
        assert!(Version::try_from(i64::from(u32::MAX)).is_ok());
        assert!(Version::try_from(i64::from(u32::MAX) + 1).is_err());
        Ok(())
    }

    #[test]
    fn test_new() -> anyhow::Result<()> {
        let version = Version::new();
        assert_eq!(version, Version::try_from(1_i64)?);
        Ok(())
    }
}
