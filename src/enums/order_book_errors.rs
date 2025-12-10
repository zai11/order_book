use std::fmt::{Display, Debug};

use rust_decimal::Decimal;

#[derive(PartialEq, Eq)]
pub enum OrderBookError {
    InvalidTick(Decimal),
    PriceOutOfRange,
    OrderNotFound,
    NonLimitOrderRestAttempt,
    CannotFillCompletely,
    InsufficientLiquidity,
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
            Self::Other(msg) => write!(f, "{msg}"),
        }
    }
}