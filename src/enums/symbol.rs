use std::fmt::Display;

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Symbol {
    AAPL, 
    MSFT, 
    GOOGL, 
    AMZN, 
    TSLA,
    META, 
    NVDA, 
    AMD, 
    INTC, 
    NFLX,
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AAPL => write!(f, "AAPL"),
            Self::MSFT => write!(f, "MSFT"),
            Self::GOOGL => write!(f, "GOOGL"),
            Self::AMZN => write!(f, "AMZN"),
            Self::TSLA => write!(f, "TSLA"),
            Self::META => write!(f, "META"),
            Self::NVDA => write!(f, "NVDA"),
            Self::AMD => write!(f, "AMD"),
            Self::INTC => write!(f, "INTC"),
            Self::NFLX => write!(f, "NFLX")
        }
    }
}