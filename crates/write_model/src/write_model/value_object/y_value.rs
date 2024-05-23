/// Y-value (value)
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct YValue(u32);

impl From<u32> for YValue {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<YValue> for u32 {
    fn from(value: YValue) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u32_conversion() {
        assert_eq!(u32::from(YValue::from(u32::MAX)), u32::MAX);
        assert_eq!(u32::from(YValue::from(u32::MIN)), u32::MIN);
    }
}
