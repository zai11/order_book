use crate::{enums::order_book_errors::OrderBookError, models::{order::Order, order_fill::OrderFill}};

pub trait TOrderBook {
    fn add_order(&mut self, order: Order) -> Result<(), OrderBookError>;
    fn cancel_order(&mut self, order_id: u64) -> Result<(), OrderBookError>;
    fn modify_order(&mut self, order_id: u64, order: Order) -> Result<(), OrderBookError>;
    fn execute_fill_by_order_type(&mut self, order: Order) -> Result<(), OrderBookError>;
    fn fill_limit_order(&mut self, order: &mut Order) -> Result<Vec<OrderFill>, OrderBookError>;
    fn fill_market_order(&mut self, order: &mut Order) -> Result<Vec<OrderFill>, OrderBookError>;
    fn fill_immediate_or_cancel_order(&mut self, order: &mut Order) -> Result<Vec<OrderFill>, OrderBookError>;
    fn fill_fill_or_kill_order(&mut self, order: &mut Order) -> Result<Vec<OrderFill>, OrderBookError>;
    fn match_order_against_book(&mut self, aggressive_order: &mut Order, start_index: usize, end_index: usize) -> Result<Vec<OrderFill>, OrderBookError>;
    fn rest_remaining_limit_order(&mut self, order: Order, partially_filled: bool) -> Result<(), OrderBookError>;
    fn recalculate_best_bid(&mut self, order_price: u32) -> Result<(), OrderBookError>;
    fn recalculate_best_ask(&mut self, order_price: u32) -> Result<(), OrderBookError>;
    
    fn can_fill_completely(&mut self, order: &Order) -> Result<bool, OrderBookError>;
}