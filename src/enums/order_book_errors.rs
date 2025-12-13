use std::fmt::{Display, Debug};

#[derive(PartialEq, Eq)]
pub enum OrderBookError {
    InvalidTick(u32),
    PriceOutOfRange,
    OrderNotFound,
    NonLimitOrderRestAttempt,
    CannotFillCompletely,
    InsufficientLiquidity,
    BitsetIndexOutOfRange(usize),
    FullRingBuffer,
    EmptyRingBuffer,
    InvalidConfigData,
    Other(String)
}

impl Display for OrderBookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidTick(tick_size) => write!(f, "An invalid tick size was specified. Must be {tick_size}"),
            Self::PriceOutOfRange => write!(f, "The specified price was outside of the valid range."),
            Self::OrderNotFound => write!(f, "The specified order was not found."),
            Self::NonLimitOrderRestAttempt => write!(f, "An attempt was made to rest a non-limit order. Limit orders are the only supported order that can be resting."),
            Self::CannotFillCompletely => write!(f, "A Fill or Kill order could not be completely filled. The order has been cancelled."),
            Self::InsufficientLiquidity => write!(f, "There is insufficient liquidity in the specified security to entirely fill this order."),
            Self::BitsetIndexOutOfRange(n) => write!(f, "The specified bitset index must be between 0 and {n} inclusive."),
            Self::FullRingBuffer => write!(f, "An attempt was made to append a value to a full ring buffer."),
            Self::EmptyRingBuffer => write!(f, "An attempt was made to remove a value from an empty ring buffer."),
            Self::InvalidConfigData => write!(f, "Order book config data was invalid."),
            Self::Other(msg) => write!(f, "{msg}")
        }
    }
}

impl Debug for OrderBookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidTick(tick_size) => write!(f, "An invalid tick size was specified. Must be {tick_size}"),
            Self::PriceOutOfRange => write!(f, "The specified price was outside of the valid range."),
            Self::OrderNotFound => write!(f, "The specified order was not found."),
            Self::NonLimitOrderRestAttempt => write!(f, "An attempt was made to rest a non-limit order. Limit orders are the only supported order that can be resting."),
            Self::CannotFillCompletely => write!(f, "A Fill or Kill order could not be completely filled. The order has been cancelled."),
            Self::InsufficientLiquidity => write!(f, "There is insufficient liquidity in the specified security to entirely fill this order."),
            Self::BitsetIndexOutOfRange(n) => write!(f, "The specified bitset index must be between 0 and {n} inclusive."),
            Self::FullRingBuffer => write!(f, "An attempt was made to append a value to a full ring buffer."),
            Self::EmptyRingBuffer => write!(f, "An attempt was made to remove a value from an empty ring buffer."),
            Self::InvalidConfigData => write!(f, "Order book config data was invalid."),
            Self::Other(msg) => write!(f, "{msg}"),
        }
    }
}