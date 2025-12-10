#[derive(Debug)]
pub struct BenchStats {
    pub fill_order: Vec<u64>,
    pub add_order: Vec<u64>,
    pub execute_fill_by_order_type: Vec<u64>,
    pub fill_limit_order: Vec<u64>,
    pub fill_market_order: Vec<u64>,
    pub fill_immediate_or_cancel_order: Vec<u64>,
    pub fill_fill_or_kill_order: Vec<u64>,
    pub match_order_against_book: Vec<u64>,
    pub rest_remaining_limit_order: Vec<u64>,
    pub can_fill_completely: Vec<u64>,
}

impl Default for BenchStats {
    fn default() -> Self {
        BenchStats { 
            fill_order: vec![],
            add_order: vec![], 
            execute_fill_by_order_type: vec![], 
            fill_limit_order: vec![], 
            fill_market_order: vec![], 
            fill_immediate_or_cancel_order: vec![],
            fill_fill_or_kill_order: vec![], 
            match_order_against_book: vec![], 
            rest_remaining_limit_order: vec![], 
            can_fill_completely: vec![]
        }
    }
}