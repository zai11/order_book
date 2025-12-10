use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderType {
    Limit,
    Market,
    ImmediateOrCancel,
    FillOrKill
}

impl Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Limit => write!(f, "Limit"),
            Self::Market => write!(f, "Market"),
            Self::ImmediateOrCancel => write!(f, "Immediate or Cancel"),
            Self::FillOrKill => write!(f, "Fill or Kill")
        }
    }
}