#[derive(Debug, thiserror::Error)]
#[error("error")]
pub struct Error;

/// X-value (date)
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct XValue(u32);

impl XValue {
    pub fn day_of_month(&self) -> u8 {
        (self.0 % 100) as u8
    }

    pub fn month(&self) -> u8 {
        (self.0 / 100 % 100) as u8
    }

    pub fn year(&self) -> u16 {
        (self.0 / 10000) as u16
    }
}

impl std::str::FromStr for XValue {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 10 {
            return Err(Error);
        }
        let parts = s.split('-').collect::<Vec<&str>>();
        if parts.len() != 3 || parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
            return Err(Error);
        }
        let yyyy = parts[0].parse::<u16>().map_err(|_| Error)?;
        if !(0..=9999).contains(&yyyy) {
            return Err(Error);
        }
        let mm = parts[1].parse::<u8>().map_err(|_| Error)?;
        if !(1..=12).contains(&mm) {
            return Err(Error);
        }
        let dd = parts[2].parse::<u8>().map_err(|_| Error)?;
        if !(1..=31).contains(&dd) {
            return Err(Error);
        }
        let is_leap = yyyy % 4 == 0 && (yyyy % 100 != 0 || yyyy % 400 == 0);
        let max_dd = [
            31,
            28 + if is_leap { 1 } else { 0 },
            31,
            30,
            31,
            30,
            31,
            31,
            30,
            31,
            30,
            31,
        ][mm as usize - 1];
        if dd > max_dd {
            return Err(Error);
        }

        Ok(Self(
            u32::from(yyyy) * 10000 + u32::from(mm) * 100 + u32::from(dd),
        ))
    }
}

impl std::fmt::Display for XValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format!(
            "{:04}-{:02}-{:02}",
            self.year(),
            self.month(),
            self.day_of_month(),
        )
        .fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use super::*;

    #[test]
    fn test_day_of_month() -> anyhow::Result<()> {
        assert_eq!(XValue::from_str("0000-01-01")?.day_of_month(), 1_u8);
        assert_eq!(XValue::from_str("0000-01-31")?.day_of_month(), 31_u8);
        Ok(())
    }

    #[test]
    fn test_month() -> anyhow::Result<()> {
        assert_eq!(XValue::from_str("0000-01-01")?.month(), 1_u8);
        assert_eq!(XValue::from_str("0000-12-01")?.month(), 12_u8);
        Ok(())
    }

    #[test]
    fn test_string_convesion() -> anyhow::Result<()> {
        assert!(XValue::from_str("0000-00-00").is_err());
        assert!(XValue::from_str("0000-00-01").is_err());
        assert_eq!(XValue::from_str("0000-01-01")?.to_string(), "0000-01-01");
        assert_eq!(XValue::from_str("0000-01-31")?.to_string(), "0000-01-31");
        assert!(XValue::from_str("0000-01-32").is_err());
        assert_eq!(XValue::from_str("0000-02-28")?.to_string(), "0000-02-28");
        assert_eq!(XValue::from_str("0000-02-29")?.to_string(), "0000-02-29");
        assert!(XValue::from_str("0001-02-29").is_err());
        assert_eq!(XValue::from_str("0004-02-29")?.to_string(), "0004-02-29");
        assert!(XValue::from_str("0100-02-29").is_err());
        assert_eq!(XValue::from_str("0000-03-31")?.to_string(), "0000-03-31");
        assert_eq!(XValue::from_str("0000-04-30")?.to_string(), "0000-04-30");
        assert!(XValue::from_str("0000-04-31").is_err());
        assert_eq!(XValue::from_str("0000-05-31")?.to_string(), "0000-05-31");
        assert_eq!(XValue::from_str("0000-06-30")?.to_string(), "0000-06-30");
        assert!(XValue::from_str("0000-06-31").is_err());
        assert_eq!(XValue::from_str("0000-07-31")?.to_string(), "0000-07-31");
        assert_eq!(XValue::from_str("0000-08-31")?.to_string(), "0000-08-31");
        assert_eq!(XValue::from_str("0000-09-30")?.to_string(), "0000-09-30");
        assert!(XValue::from_str("0000-09-31").is_err());
        assert_eq!(XValue::from_str("0000-10-31")?.to_string(), "0000-10-31");
        assert_eq!(XValue::from_str("0000-11-30")?.to_string(), "0000-11-30");
        assert!(XValue::from_str("0000-11-31").is_err());
        assert_eq!(XValue::from_str("0000-12-31")?.to_string(), "0000-12-31");
        Ok(())
    }

    #[test]
    fn test_year() -> anyhow::Result<()> {
        assert_eq!(XValue::from_str("0000-01-01")?.year(), 0_u16);
        assert_eq!(XValue::from_str("9999-01-01")?.year(), 9999_u16);
        Ok(())
    }
}
