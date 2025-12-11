use std::collections::{HashMap, VecDeque};

use rust_decimal::Decimal;

use crate::{enums::order_book_errors::OrderBookError, models::{order::Order, order_fill::OrderFill}, traits::order_book::TOrderBook};

pub struct DynamicPriceOrderBook {
    pub bids: HashMap<Decimal, VecDeque<Order>>,
    pub asks: HashMap<Decimal, VecDeque<Order>>
}

impl TOrderBook for DynamicPriceOrderBook {
    fn add_order(&mut self, order: Order) -> Result<(), OrderBookError> {
        Err(OrderBookError::Other("Not implemented yet".into()))
    }

    fn cancel_order(&mut self, order_id: u64) -> Result<(), OrderBookError> {
        Err(OrderBookError::Other("Not implemented yet".into()))
    }

    fn modify_order(&mut self, order_id: u64, order: Order) -> Result<(), OrderBookError> {
        Err(OrderBookError::Other("Not implemented yet".into()))
    }

    fn execute_fill_by_order_type(&mut self, order: Order) -> Result<(), OrderBookError> {
        Err(OrderBookError::Other("Not implemented yet".into()))
    }

    fn fill_limit_order(&mut self, order: &mut Order) -> Result<Vec<OrderFill>, OrderBookError> {
        Err(OrderBookError::Other("Not implemented yet".into()))
    }

    fn fill_market_order(&mut self, order: &mut Order) -> Result<Vec<OrderFill>, OrderBookError> {
        Err(OrderBookError::Other("Not implemented yet".into()))
    }

    fn fill_immediate_or_cancel_order(&mut self, order: &mut Order) -> Result<Vec<OrderFill>, OrderBookError> {
        Err(OrderBookError::Other("Not implemented yet".into()))
    }

    fn fill_fill_or_kill_order(&mut self, order: &mut Order) -> Result<Vec<OrderFill>, OrderBookError> {
        Err(OrderBookError::Other("Not implemented yet".into()))
    }

    fn match_order_against_book(&mut self, aggressive_order: &mut Order, start_index: usize, end_index: usize) -> Result<Vec<OrderFill>, OrderBookError> {
        Err(OrderBookError::Other("Not implemented yet".into()))
    }

    fn rest_remaining_limit_order(&mut self, order_index: Order, partially_filled: bool) -> Result<(), OrderBookError> {
        Err(OrderBookError::Other("Not implemented yet".into()))
    }

    fn recalculate_best_bid(&mut self, order_price: u32) -> Result<(), OrderBookError> {
        Err(OrderBookError::Other("Not implemented yet".into()))
    }

    fn recalculate_best_ask(&mut self, order_price: u32) -> Result<(), OrderBookError> {
        Err(OrderBookError::Other("Not implemented yet".into()))
    }

    fn can_fill_completely(&mut self, order: &Order) -> Result<bool, OrderBookError> {
        Err(OrderBookError::Other("Not implemented yet".into()))
    }
}

