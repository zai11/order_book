#[derive(Debug, Clone)]
pub struct OrderFill {
    pub aggressive_order_id: u64,
    pub resting_order_id: u64,
    pub price: u32,
    pub quantity: u32,
    pub timestamp: u128
}