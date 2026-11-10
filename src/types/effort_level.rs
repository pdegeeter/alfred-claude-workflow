use anyhow::bail;

/// Claude effort level: thinking depth and overall token spend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffortLevel {
    Low,
    Medium,
    High,
    Xhigh,
    Max,
}

impl EffortLevel {
    const ALL: [EffortLevel; 5] = [Self::Low, Self::Medium, Self::High, Self::Xhigh, Self::Max];

    /// # Errors
    /// When the value is not a known effort level.
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        let Some(level) = Self::ALL.into_iter().find(|level| level.as_str() == value) else {
            let names: Vec<&str> = Self::ALL.iter().map(|level| level.as_str()).collect();
            bail!("Unknown level \"{value}\": use one of {}", names.join(", "));
        };

        Ok(level)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Xhigh => "xhigh",
            Self::Max => "max",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EffortLevel;

    #[test]
    fn accepts_every_effort_level() {
        for name in ["low", "medium", "high", "xhigh", "max"] {
            assert_eq!(EffortLevel::parse(name).unwrap().as_str(), name);
        }
    }

    #[test]
    fn rejects_an_unknown_level() {
        let error = EffortLevel::parse("extreme").unwrap_err();

        assert_eq!(
            error.to_string(),
            "Unknown level \"extreme\": use one of low, medium, high, xhigh, max"
        );
    }
}
