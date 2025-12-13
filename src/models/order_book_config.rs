
#[derive(Clone)]
pub struct OrderBookConfig {
    pub min_price: u32,
    pub max_price: u32,
    pub tick_size: u32,
    pub queue_size: usize
}