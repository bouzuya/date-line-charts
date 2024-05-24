use super::{ChartId, XValue};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("chart id")]
    ChartId(#[source] crate::value_object::chart_id::Error),
    #[error("x value")]
    XValue(#[source] crate::value_object::x_value::Error),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DataPointId(ChartId, XValue);

impl DataPointId {
    pub fn new(chart_id: ChartId, x_value: XValue) -> Self {
        Self(chart_id, x_value)
    }

    pub fn chart_id(&self) -> ChartId {
        self.0
    }

    pub fn x_value(&self) -> XValue {
        self.1
    }
}

impl std::str::FromStr for DataPointId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chart_id = ChartId::from_str(&s[0..36]).map_err(Error::ChartId)?;
        let x_value = XValue::from_str(&s[37..]).map_err(Error::XValue)?;
        Ok(Self(chart_id, x_value))
    }
}

impl std::fmt::Display for DataPointId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use super::*;

    #[test]
    fn test_string_convesion() -> anyhow::Result<()> {
        let s = "67051e1b-fc32-43c8-899f-e2c73a1319f4:2020-01-02";
        let id = DataPointId::from_str(s)?;
        assert_eq!(id.to_string(), s);
        assert_eq!(
            id.chart_id(),
            ChartId::from_str("67051e1b-fc32-43c8-899f-e2c73a1319f4")?
        );
        assert_eq!(id.x_value(), XValue::from_str("2020-01-02")?);
        Ok(())
    }
}
