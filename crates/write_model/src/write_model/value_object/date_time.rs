use chrono::SubsecRound;

#[derive(Debug, thiserror::Error)]
#[error("error {0}")]
pub struct Error(String);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DateTime(chrono::DateTime<chrono::Utc>);

impl DateTime {
    pub fn from_unix_timestamp_millis(unix_timestamp_millis: i64) -> Result<Self, Error> {
        chrono::DateTime::from_timestamp_millis(unix_timestamp_millis)
            .ok_or_else(|| Error("invalid timestamp".to_owned()))
            .map(Self)
    }

    pub fn now() -> Self {
        Self(SubsecRound::trunc_subsecs(chrono::Utc::now(), 3))
    }

    pub fn to_unix_timestamp_millis(&self) -> i64 {
        self.0.timestamp_millis()
    }
}

impl std::str::FromStr for DateTime {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        chrono::DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.to_utc())
            .map_err(|e| Error(e.to_string()))
            .and_then(|dt| {
                if dt == SubsecRound::trunc_subsecs(dt, 3) {
                    Ok(dt)
                } else {
                    Err(Error("invalid subsec".to_string()))
                }
            })
            .map(Self)
    }
}

impl std::fmt::Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
            .fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use super::*;

    #[test]
    fn test_string_convesion() -> anyhow::Result<()> {
        let dt = DateTime::now();
        assert_eq!(dt, DateTime::from_str(&dt.to_string())?);
        assert!(DateTime::from_str("2020-01-02T03:04:05.1234Z").is_err());
        for (s, expected) in [
            ("2020-01-02T03:04:05.006Z", "2020-01-02T03:04:05.006Z"),
            ("2020-01-02T03:04:05Z", "2020-01-02T03:04:05.000Z"),
            ("2020-01-02T03:04:05.006+09:00", "2020-01-01T18:04:05.006Z"),
        ] {
            assert_eq!(DateTime::from_str(s)?.to_string(), expected);
        }
        Ok(())
    }

    #[test]
    fn test_unix_timestamp_millis_convesion() -> anyhow::Result<()> {
        let dt = DateTime::from_unix_timestamp_millis(0)?;
        assert_eq!(dt.to_string(), "1970-01-01T00:00:00.000Z");
        assert_eq!(dt.to_unix_timestamp_millis(), 0);
        let dt = DateTime::from_unix_timestamp_millis(1)?;
        assert_eq!(dt.to_string(), "1970-01-01T00:00:00.001Z");
        assert_eq!(dt.to_unix_timestamp_millis(), 1);
        let dt = DateTime::from_unix_timestamp_millis(1000)?;
        assert_eq!(dt.to_string(), "1970-01-01T00:00:01.000Z");
        assert_eq!(dt.to_unix_timestamp_millis(), 1000);
        Ok(())
    }
}
