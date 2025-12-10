use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderStatus {
    PendingNew,         // Received but not yet in book
    Active,             // Resting in book
    PartiallyFilled,    // Some quantity executed
    Filled,             // Fully executed
    Canceled,           // Canceled by user
    Rejected,           // Rejected by risk/validation
    Expired             // Time limit reached
}

impl Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PendingNew => write!(f, "Pending New"),
            Self::Active => write!(f, "Active"),
            Self::PartiallyFilled => write!(f, "Partially Filled"),
            Self::Filled => write!(f, "Filled"),
            Self::Canceled => write!(f, "Canceled"),
            Self::Rejected => write!(f, "Rejected"),
            Self::Expired => write!(f, "Expired")
        }
    }
}