use crate::enums::{order_side::OrderSide, order_status::OrderStatus, order_type::OrderType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    pub order_id: u64,
    pub order_type: OrderType,
    pub order_status: OrderStatus,
    pub order_side: OrderSide,
    pub user_id: u32,
    pub price: u32,
    pub quantity: i32
}