use std::{collections::{HashMap, VecDeque}, vec};

use rust_decimal::{prelude::ToPrimitive};
use slab::Slab;

use crate::{enums::{order_book_errors::OrderBookError, order_side::OrderSide, order_status::OrderStatus, order_type::OrderType}, models::{bench_stats::BenchStats, fixed_price_order_book_config::FixedPriceOrderBookConfig, order::Order, order_fill::OrderFill}, traits::order_book::TOrderBook, utils::get_timestamp};

pub struct FixedPriceOrderBook {
    pub config: FixedPriceOrderBookConfig,
    pub bids: Vec<VecDeque<usize>>,         // Stores an index of order_ledger
    pub asks: Vec<VecDeque<usize>>,         // ""
    pub order_ledger: Slab<Order>,
    pub index_mappings: HashMap<u64, usize>,       // <order_id, ledger_index>
    pub trade_history: Vec<OrderFill>,
    pub best_bid_index: Option<usize>,
    pub best_ask_index: Option<usize>,
    pub bench_stats: BenchStats
}

impl FixedPriceOrderBook {
    pub fn new(config: FixedPriceOrderBookConfig) -> Self {
        let vec_capacity = ((config.max_price - config.min_price) / config.tick_size)
            .to_u64()
            .ok_or(OrderBookError::Other("Unable to convert index to u64.".into())).unwrap() as usize;

        let mut bids = vec![];
        for _ in 0..(vec_capacity + 1) {
            let mut queue = VecDeque::new();
            queue.reserve(config.queue_size);
            bids.push(queue);
        }

        let mut asks = vec![];
        for _ in 0..(vec_capacity + 1) {
            let mut queue = VecDeque::new();
            queue.reserve(config.queue_size);
            asks.push(queue);
        }

        FixedPriceOrderBook {
            config,
            bids,
            asks,
            order_ledger: Slab::new(),
            index_mappings: HashMap::new(),
            trade_history: vec![],
            best_bid_index: None,
            best_ask_index: None,
            bench_stats: Default::default()
        }
    }
    
    #[inline(never)]
    pub fn fill_order(&mut self, queue: &mut VecDeque<usize>, aggressive_order: &mut Order, resting_order_index: usize, fills: &mut Vec<OrderFill>) -> Result<bool, OrderBookError> {
        crate::time_func!(self.bench_stats.fill_order, {

            let mut remove_resting_order = false;
            let mut filled_order = false;

            {
                let resting_order = self.order_ledger.get_mut(resting_order_index)
                    .ok_or(OrderBookError::OrderNotFound)?;

                if resting_order.quantity == aggressive_order.quantity {
                    let fill = OrderFill {
                        aggressive_order_id: aggressive_order.order_id,
                        resting_order_id: resting_order.order_id,
                        price: resting_order.price,
                        quantity: resting_order.quantity as u32,
                        timestamp: get_timestamp()
                    };
                    fills.push(fill);
                    remove_resting_order = true;
                    aggressive_order.quantity -= resting_order.quantity;
                    filled_order = true;
                }
                else if resting_order.quantity > aggressive_order.quantity {
                    let fill = OrderFill {
                        aggressive_order_id: aggressive_order.order_id,
                        resting_order_id: resting_order.order_id,
                        price: resting_order.price,
                        quantity: aggressive_order.quantity as u32,
                        timestamp: get_timestamp()
                    };
                    fills.push(fill);
                    resting_order.quantity -= aggressive_order.quantity;
                    queue.push_front(resting_order_index);
                    aggressive_order.quantity = 0;
                    filled_order = true;
                }
                else {
                    let fill = OrderFill {
                        aggressive_order_id: aggressive_order.order_id,
                        resting_order_id: resting_order.order_id,
                        price: resting_order.price,
                        quantity: resting_order.quantity as u32,
                        timestamp: get_timestamp()
                    };
                    fills.push(fill);
                    aggressive_order.quantity -= resting_order.quantity; 
                    remove_resting_order = true;
                }
            }

            if remove_resting_order {
                self.order_ledger.remove(resting_order_index);  
            }

            Ok(filled_order)
        })
    }
}

impl TOrderBook for FixedPriceOrderBook {
    #[inline(never)]
    fn add_order(&mut self, order: Order) -> Result<(), OrderBookError> {
        crate::time_func!(self.bench_stats.add_order, {
            if order.price as usize >= self.bids.len() {
                return Err(OrderBookError::PriceOutOfRange);
            }

            self.execute_fill_by_order_type(order)?;

            Ok(())
        })
    }

    fn cancel_order(&mut self, order_id: u64) -> Result<(), OrderBookError> {
        if !self.order_ledger.iter().any(|(_, order)| order.order_id == order_id) {
            return Err(OrderBookError::OrderNotFound);
        }

        let ledger_index = self.index_mappings[&order_id];

        let order = &self.order_ledger[ledger_index];
        if order.price as usize >= self.bids.len() {
            return Err(OrderBookError::PriceOutOfRange);
        }

        match order.order_side {
            OrderSide::Buy => {
                if let Some(queue) = self.bids.get_mut(order.price as usize) {
                    queue.retain(|&idx| idx != ledger_index);
                    self.order_ledger.remove(ledger_index);
                }
                else {
                    return Err(OrderBookError::OrderNotFound);
                }
            },
            OrderSide::Sell => {
                if let Some(queue) = self.asks.get_mut(order.price as usize) {
                    queue.retain(|&idx| idx != ledger_index);
                    self.order_ledger.remove(ledger_index);
                }
                else {
                    return Err(OrderBookError::OrderNotFound);
                }
            }
        }

        Ok(())
    }

    fn modify_order(&mut self, order_id: u64, order: Order) -> Result<(), OrderBookError> {
        self.cancel_order(order_id)?;
        self.add_order(order)
    }

    #[inline(never)]
    fn execute_fill_by_order_type(&mut self, mut order: Order) -> Result<(), OrderBookError> {
        crate::time_func!(self.bench_stats.execute_fill_by_order_type, {
            match order.order_type {
                OrderType::Limit => {
                    let fills = self.fill_limit_order(&mut order)?;

                    let partially_filled = fills.len() > 0;

                    if order.quantity > 0 {
                        self.rest_remaining_limit_order(order, partially_filled)?;
                    }
                },
                OrderType::Market => {
                    self.fill_market_order(&mut order)?;

                    if order.quantity > 0 {
                        return Err(OrderBookError::InsufficientLiquidity);
                    }
                },
                OrderType::ImmediateOrCancel => {
                    self.fill_immediate_or_cancel_order(&mut order)?;
                },
                OrderType::FillOrKill => {
                    self.fill_fill_or_kill_order(&mut order)?;
                }
            }
        
            Ok(())
        })
    }

    #[inline(never)]
    fn fill_limit_order(&mut self, order: &mut Order) -> Result<Vec<OrderFill>, OrderBookError> {
        crate::time_func!(self.bench_stats.fill_limit_order, {
            let fills = match order.order_side {
                OrderSide::Buy => {
                    self.match_order_against_book(order, 0, order.price as usize)?
                }
                OrderSide::Sell => {
                    self.match_order_against_book(order, order.price as usize, self.bids.len() - 1)?
                }
            };

            self.trade_history.append(&mut fills.clone());

            Ok(fills)
        })
    }

    #[inline(never)]
    fn fill_market_order(&mut self, order: &mut Order) -> Result<Vec<OrderFill>, OrderBookError> {
        crate::time_func!(self.bench_stats.fill_market_order, {
            let mut fills = match order.order_side {
                OrderSide::Buy => {
                    self.match_order_against_book(order, 0, self.asks.len() - 1)?
                },
                OrderSide::Sell => {
                    self.match_order_against_book(order, 0, self.bids.len() - 1)?
                }
            };

            self.trade_history.append(&mut fills);

            Ok(fills)
        })
    }

    #[inline(never)]
    fn fill_immediate_or_cancel_order(&mut self, order: &mut Order) -> Result<Vec<OrderFill>, OrderBookError> {
        crate::time_func!(self.bench_stats.fill_immediate_or_cancel_order, {
            let fills = self.fill_limit_order(order)?;
            
            Ok(fills)
        })
    }

    #[inline(never)]
    fn fill_fill_or_kill_order(&mut self, order: &mut Order) -> Result<Vec<OrderFill>, OrderBookError> {
        crate::time_func!(self.bench_stats.fill_fill_or_kill_order, {
            if !self.can_fill_completely(&order)? {
                return Err(OrderBookError::CannotFillCompletely);
            }

            let fills = self.fill_limit_order(order)?;

            Ok(fills)
        })
    }

    #[inline(never)]
    fn match_order_against_book(&mut self, aggressive_order: &mut Order, start_index: usize, end_index: usize) -> Result<Vec<OrderFill>, OrderBookError> {
        crate::time_func!(self.bench_stats.match_order_against_book, {
            let mut fills = Vec::new();

            let match_side = if aggressive_order.order_side == OrderSide::Buy {
                OrderSide::Sell
            }
            else {
                OrderSide::Buy
            };

            match match_side {
                OrderSide::Buy => {
                    let end_index = self.best_bid_index.unwrap_or(end_index);
                    //println!("{} price levels could be checked at worst case", end_index - start_index);
                    let mut empty_queues = 0;
                    for i in (start_index..=end_index).rev() {
                        if aggressive_order.quantity == 0 {
                            break;
                        }

                        let queue_option = self.bids.get_mut(i);
                        if queue_option.is_none() {
                            empty_queues += 1;
                            continue;
                        }
                        let mut queue = std::mem::take(queue_option.unwrap());

                        while aggressive_order.quantity > 0 && !queue.is_empty() {
                            let resting_order_index = queue.pop_front().unwrap();
                            let _filled = self.fill_order(&mut queue, aggressive_order, resting_order_index, &mut fills)?;
                        }

                        self.bids[i] = queue;
                    }
                    if empty_queues > 0 {
                        println!("{empty_queues} empty queues were encountered.");
                    }
                },
                OrderSide::Sell => {
                    let start_index = self.best_ask_index.unwrap_or(start_index);
                    //println!("{} price levels could be checked at worst case", end_index - start_index);
                    let mut empty_queues = 0;
                    for i in start_index..=end_index {
                        if aggressive_order.quantity == 0 {
                            break;
                        }

                        let queue_option = self.asks.get_mut(i);
                        if queue_option.is_none() {
                            empty_queues += 1;
                            continue;
                        }

                        let mut queue = std::mem::take(queue_option.unwrap());

                        while aggressive_order.quantity > 0 && !queue.is_empty() {
                            let resting_order = queue.pop_front().unwrap();
                            let _filled = self.fill_order(&mut queue, aggressive_order, resting_order, &mut fills)?;
                        }

                        self.asks[i] = queue;
                    }
                    if empty_queues > 0 {
                        println!("{empty_queues} empty queues were encountered.");
                    }
                }
            }

            Ok(fills)
        })
    }

    #[inline(never)]
    fn rest_remaining_limit_order(&mut self, mut order: Order, partially_filled: bool) -> Result<(), OrderBookError> {
        crate::time_func!(self.bench_stats.rest_remaining_limit_order, {
            if order.order_type != OrderType::Limit {
                return Err(OrderBookError::NonLimitOrderRestAttempt);
            }

            order.order_status = if partially_filled {
                OrderStatus::PartiallyFilled
            }
            else {
                OrderStatus::Active
            };

            match order.order_side {
                OrderSide::Buy => {
                    self.recalculate_best_bid(order.price)?;
                    if let Some(queue) = self.bids.get_mut(order.price as usize) {
                        let order_id = order.order_id;
                        let order_index = self.order_ledger.insert(order);
                        queue.push_back(order_index);
                        self.index_mappings.insert(order_id, order_index);
                    }
                    else {
                        let order_id = order.order_id;
                        let order_price = order.price;
                        let order_index = self.order_ledger.insert(order);
                        let mut queue = VecDeque::new();
                        queue.push_back(order_index);
                        self.bids.insert(order_price as usize, queue);
                        self.index_mappings.insert(order_id, order_index);
                    }
                },
                OrderSide::Sell => {
                    self.recalculate_best_ask(order.price)?;
                    if let Some(queue) = self.asks.get_mut(order.price as usize) {
                        let order_id = order.order_id;
                        let order_index = self.order_ledger.insert(order);
                        queue.push_back(order_index);
                        self.index_mappings.insert(order_id, order_index);
                    }
                    else {
                        let order_id = order.order_id;
                        let order_price = order.price;
                        let order_index = self.order_ledger.insert(order);
                        let mut queue = VecDeque::new();
                        queue.push_back(order_index);
                        self.asks.insert(order_price as usize, queue);
                        self.index_mappings.insert(order_id, order_index);
                    }
                }
            }

            Ok(())
        })
        
    }

    fn recalculate_best_bid(&mut self, order_price: u32) -> Result<(), OrderBookError> {
        if let Some(current_best) = self.best_bid_index {
            if order_price as usize > current_best {
                self.best_bid_index = Some(order_price as usize);
                /*self.best_bid_index = (0..self.bids.len())
                    .rev()
                    .find(|&i| !self.bids[i].is_empty());*/
            }
        }
        else {
            self.best_bid_index = Some(order_price as usize);
            /*self.best_bid_index = (0..self.bids.len())
                .rev()
                .find(|&i| !self.bids[i].is_empty());*/
        }

        Ok(())
    }

    fn recalculate_best_ask(&mut self, order_price: u32) -> Result<(), OrderBookError> {
        if let Some(current_best) = self.best_ask_index {
            if (order_price as usize) < current_best {
                self.best_ask_index = Some(order_price as usize);
                /*self.best_ask_index = (0..self.asks.len())
                    .find(|&i| !self.asks[i].is_empty());*/
            }
        }
        else {
            self.best_ask_index = Some(order_price as usize);
            /*self.best_ask_index = (0..self.asks.len())
                .find(|&i| !self.asks[i].is_empty());*/
        }

        Ok(())
    }

    #[inline(never)]
    fn can_fill_completely(&mut self, order: &Order) -> Result<bool, OrderBookError> {
        crate::time_func!(self.bench_stats.can_fill_completely, {
            let mut available_quantity = 0u32;

            match order.order_side {
                OrderSide::Buy => {
                    for i in 0..=order.price as usize {
                        let queue = &self.asks[i];
                        available_quantity += queue.iter().map(|&idx| self.order_ledger[idx].quantity as u32).sum::<u32>();
                        if available_quantity >= order.quantity as u32 {
                            return Ok(true);
                        }
                    }
                },
                OrderSide::Sell => {
                    for i in (order.price as usize..self.bids.len()).rev() {
                        let queue = &self.bids[i];
                        available_quantity += queue.iter().map(|&idx| self.order_ledger[idx].quantity as u32).sum::<u32>();
                        if available_quantity >= order.quantity as u32 {
                            return Ok(true);
                        }
                    }
                }
            }

            Ok(false)
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_fill_order_correctly_fills_aggressive_order_resting_and_aggressive_order_quantities_equal() {
        let config = FixedPriceOrderBookConfig {
            min_price: 0,
            max_price: 10000,
            tick_size: 1,
            queue_size: 100
        };
        let mut order_book = FixedPriceOrderBook::new(config);

        let sell_order = Order {
            order_id: 0,
            order_type: OrderType::Limit,
            order_status: OrderStatus::Active,
            order_side: OrderSide::Sell,
            user_id: 0,
            price: 10000,
            quantity: 800
        };

        let mut buy_order = Order {
            order_id: 1,
            order_type: OrderType::Market,
            order_status: OrderStatus::PendingNew,
            order_side: OrderSide::Buy,
            user_id: 1,
            price: 10000,
            quantity: 800
        };

        let price_index = sell_order.price as usize;


        let sell_order_index = order_book.order_ledger.insert(sell_order.clone());
        order_book.asks[price_index].push_back(sell_order_index);

        let mut queue = order_book.asks[price_index].clone();
        let mut fills = Vec::new();

        queue.pop_front();

        let fill_order_result = order_book.fill_order(&mut queue, &mut buy_order, sell_order_index, &mut fills);

        assert!(fill_order_result.is_ok());
        assert!(fill_order_result.unwrap());
        assert!(queue.is_empty());
        assert!(fills.len() == 1);
        assert!(fills[0].aggressive_order_id == buy_order.order_id);
        assert!(fills[0].resting_order_id == sell_order.order_id);
    }

    #[test]
    fn test_fill_order_correctly_fills_aggressive_order_resting_order_quantity_greater_than_aggressive_order_quantity() {
        let config = FixedPriceOrderBookConfig {
            min_price: 0,
            max_price: 10000,
            tick_size: 1,
            queue_size: 100
        };
        let mut order_book = FixedPriceOrderBook::new(config);

        let sell_order = Order {
            order_id: 0,
            order_type: OrderType::Limit,
            order_status: OrderStatus::Active,
            order_side: OrderSide::Sell,
            user_id: 0,
            price: 10000,
            quantity: 800
        };

        let mut buy_order = Order {
            order_id: 1,
            order_type: OrderType::Market,
            order_status: OrderStatus::PendingNew,
            order_side: OrderSide::Buy,
            user_id: 1,
            price: 10000,
            quantity: 300
        };

        let price_index = sell_order.price as usize;

        let sell_order_index = order_book.order_ledger.insert(sell_order.clone());
        order_book.asks[price_index].push_back(sell_order_index);

        let mut queue = order_book.asks[price_index].clone();
        let mut fills = Vec::new();

        queue.pop_front();

        let fill_order_result = order_book.fill_order(&mut queue, &mut buy_order, sell_order_index, &mut fills);

        assert!(fill_order_result.is_ok());
        assert!(fill_order_result.unwrap());
        assert_eq!(queue.len(), 1);
        assert_eq!(queue[0], sell_order_index);
        assert_eq!(order_book.order_ledger[sell_order_index].quantity, 500);
        assert_eq!(fills.len(), 1);
        assert_eq!(fills[0].aggressive_order_id, buy_order.order_id);
        assert_eq!(fills[0].resting_order_id, sell_order.order_id);
    }

    #[test]
    fn test_fill_order_correctly_fills_aggressive_order_aggressive_order_quantity_greater_than_resting_order_quantity() {
        let config = FixedPriceOrderBookConfig {
            min_price: 0,
            max_price: 10000,
            tick_size: 1,
            queue_size: 100
        };
        let mut order_book = FixedPriceOrderBook::new(config);

        let sell_order = Order {
            order_id: 0,
            order_type: OrderType::Limit,
            order_status: OrderStatus::Active,
            order_side: OrderSide::Sell,
            user_id: 0,
            price: 10000,
            quantity: 300
        };

        let mut buy_order = Order {
            order_id: 1,
            order_type: OrderType::Market,
            order_status: OrderStatus::PendingNew,
            order_side: OrderSide::Buy,
            user_id: 1,
            price: 10000,
            quantity: 800
        };

        let price_index = sell_order.price as usize;

        let sell_order_index = order_book.order_ledger.insert(sell_order.clone());
        order_book.asks[price_index].push_back(sell_order_index);

        let mut queue = order_book.asks[price_index].clone();
        let mut fills = Vec::new();

        queue.pop_front();

        let fill_order_result = order_book.fill_order(&mut queue, &mut buy_order, sell_order_index, &mut fills);

        assert!(fill_order_result.is_ok());
        assert!(!fill_order_result.unwrap());
        assert!(queue.is_empty());
        assert_eq!(buy_order.quantity, 500);
        assert_eq!(fills.len(), 1);
        assert_eq!(fills[0].aggressive_order_id, buy_order.order_id);
        assert_eq!(fills[0].resting_order_id, sell_order.order_id);
    }

    #[test]
    fn test_add_order_correctly_adds_limit_order_to_empty_order_book() {
        let config = FixedPriceOrderBookConfig {
            min_price: 0,
            max_price: 10000,
            tick_size: 1,
            queue_size: 100
        };
        let mut order_book = FixedPriceOrderBook::new(config);

        let mut order = Order {
            order_id: 0,
            order_type: OrderType::Limit,
            order_status: OrderStatus::PendingNew,
            order_side: OrderSide::Sell,
            user_id: 0,
            price: 10000,
            quantity: 300
        };

        let price_index = order.price as usize;

        let add_order_result = order_book.add_order(order.clone());

        let order_index = order_book.index_mappings[&order.order_id];

        order.order_status = OrderStatus::Active;

        assert!(add_order_result.is_ok());
        assert_eq!(order_book.asks[price_index].len(), 1);
        assert_eq!(order_book.asks[price_index][0], order_index);
    }

    #[test]
    fn test_add_order_correctly_executes_order_fill() {
        let config = FixedPriceOrderBookConfig {
            min_price: 0,
            max_price: 10000,
            tick_size: 1,
            queue_size: 100
        };
        let mut order_book = FixedPriceOrderBook::new(config);

        let mut sell_order = Order {
            order_id: 0,
            order_type: OrderType::Limit,
            order_status: OrderStatus::PendingNew,
            order_side: OrderSide::Sell,
            user_id: 0,
            price: 10000,
            quantity: 300
        };

        let price_index = sell_order.price as usize;

        let add_sell_order_result = order_book.add_order(sell_order.clone());

        sell_order.order_status = OrderStatus::Active;

        let sell_order_index = order_book.index_mappings[&sell_order.order_id];

        assert!(add_sell_order_result.is_ok());
        assert_eq!(order_book.asks[price_index].len(), 1);
        assert_eq!(order_book.asks[price_index][0], sell_order_index);

        let buy_order = Order {
            order_id: 1,
            order_type: OrderType::Market,
            order_status: OrderStatus::PendingNew,
            order_side: OrderSide::Buy,
            user_id: 1,
            price: 10000,
            quantity: 300
        };

        let add_buy_order_result = order_book.add_order(buy_order.clone());

        assert!(add_buy_order_result.is_ok());
        assert!(order_book.asks[price_index].is_empty());
    }

    #[test]
    fn test_add_order_correctly_executes_order_fill_on_limit_order_and_adds_remaining_to_order_book() {
        let config = FixedPriceOrderBookConfig {
            min_price: 0,
            max_price: 10000,
            tick_size: 1,
            queue_size: 100
        };
        let mut order_book = FixedPriceOrderBook::new(config);

        let mut sell_order = Order {
            order_id: 0,
            order_type: OrderType::Limit,
            order_status: OrderStatus::PendingNew,
            order_side: OrderSide::Sell,
            user_id: 0,
            price: 10000,
            quantity: 300
        };

        let price_index = sell_order.price as usize;

        let add_sell_order_result = order_book.add_order(sell_order.clone());

        sell_order.order_status = OrderStatus::Active;

        let sell_order_index = order_book.index_mappings[&sell_order.order_id];

        assert!(add_sell_order_result.is_ok());
        assert_eq!(order_book.asks[price_index].len(), 1);
        assert_eq!(order_book.asks[price_index][0], sell_order_index);

        let mut buy_order = Order {
            order_id: 1,
            order_type: OrderType::Limit,
            order_status: OrderStatus::PendingNew,
            order_side: OrderSide::Buy,
            user_id: 1,
            price: 10000,
            quantity: 500
        };

        let add_buy_order_result = order_book.add_order(buy_order.clone());

        buy_order.order_status = OrderStatus::PartiallyFilled;
        buy_order.quantity = 200;

        let buy_order_index = order_book.index_mappings[&buy_order.order_id];

        assert!(add_buy_order_result.is_ok());
        assert!(order_book.asks[price_index].is_empty());
        assert_eq!(order_book.bids[price_index].len(), 1);
        assert_eq!(order_book.bids[price_index][0], buy_order_index);
    }

    #[test]
    fn test_add_order_errors_price_out_of_range() {
        let config = FixedPriceOrderBookConfig {
            min_price: 0,
            max_price: 10000,
            tick_size: 1,
            queue_size: 100
        };
        let mut order_book = FixedPriceOrderBook::new(config);

        let order = Order {
            order_id: 0,
            order_type: OrderType::Limit,
            order_status: OrderStatus::PendingNew,
            order_side: OrderSide::Sell,
            user_id: 0,
            price: 100000,
            quantity: 300
        };

        let add_order_result = order_book.add_order(order.clone());

        assert!(add_order_result.is_err());
        assert_eq!(add_order_result.err().unwrap(), OrderBookError::PriceOutOfRange);
    }

    #[test]
    fn test_cancel_order_correctly_cancels_resting_limit_order() {
        let config = FixedPriceOrderBookConfig {
            min_price: 0,
            max_price: 10000,
            tick_size: 1,
            queue_size: 100
        };
        let mut order_book = FixedPriceOrderBook::new(config);

        let mut order = Order {
            order_id: 0,
            order_type: OrderType::Limit,
            order_status: OrderStatus::PendingNew,
            order_side: OrderSide::Sell,
            user_id: 0,
            price: 10000,
            quantity: 300
        };

        let price_index = order.price as usize;

        let add_order_result = order_book.add_order(order.clone());

        order.order_status = OrderStatus::Active;

        let order_index = order_book.index_mappings[&order.order_id];

        assert!(add_order_result.is_ok());
        assert_eq!(order_book.asks[price_index].len(), 1);
        assert_eq!(order_book.asks[price_index][0], order_index);

        let cancel_order_result = order_book.cancel_order(order.order_id);

        assert!(cancel_order_result.is_ok());
        assert!(order_book.asks[price_index].is_empty());
    }

    #[test]
    fn test_cancel_order_errors_order_not_found() {
        let config = FixedPriceOrderBookConfig {
            min_price: 0,
            max_price: 10000,
            tick_size: 1,
            queue_size: 100
        };
        let mut order_book = FixedPriceOrderBook::new(config);

        let mut order = Order {
            order_id: 0,
            order_type: OrderType::Limit,
            order_status: OrderStatus::PendingNew,
            order_side: OrderSide::Sell,
            user_id: 0,
            price: 10000,
            quantity: 300
        };

        let price_index = order.price as usize;

        let add_order_result = order_book.add_order(order.clone());

        order.order_status = OrderStatus::Active;

        let order_index = order_book.index_mappings[&order.order_id];

        assert!(add_order_result.is_ok());
        assert_eq!(order_book.asks[price_index].len(), 1);
        assert_eq!(order_book.asks[price_index][0], order_index);

        let cancel_order_result = order_book.cancel_order(99);

        assert!(cancel_order_result.is_err());
        assert_eq!(cancel_order_result.err().unwrap(), OrderBookError::OrderNotFound);
        assert_eq!(order_book.asks[price_index].len(), 1);
        assert_eq!(order_book.asks[price_index][0], order_index);
    }

    #[test]
    fn test_cancel_order_errors_price_out_of_range() {
        let config = FixedPriceOrderBookConfig {
            min_price: 0,
            max_price: 10000,
            tick_size: 1,
            queue_size: 100
        };
        let mut order_book = FixedPriceOrderBook::new(config);

        let order = Order {
            order_id: 0,
            order_type: OrderType::Limit,
            order_status: OrderStatus::Active,
            order_side: OrderSide::Sell,
            user_id: 0,
            price: 10100,
            quantity: 300
        };

        let price_index = order.price as usize;

        
        let order_index = order_book.order_ledger.insert(order.clone());
        order_book.asks.extend([const { VecDeque::new() }; 10000]);
        order_book.asks[price_index].push_back(order_index);

        let cancel_order_result = order_book.cancel_order(99);

        assert!(cancel_order_result.is_err());
        assert_eq!(cancel_order_result.err().unwrap(), OrderBookError::OrderNotFound);
        assert_eq!(order_book.asks[price_index].len(), 1);
        assert_eq!(order_book.asks[price_index][0], order_index);
    }

    #[test]
    fn test_modify_order_correctly_modifies_resting_limit_order() {
        let config = FixedPriceOrderBookConfig {
            min_price: 0,
            max_price: 10000,
            tick_size: 1,
            queue_size: 100
        };
        let mut order_book = FixedPriceOrderBook::new(config);

        let mut order = Order {
            order_id: 0,
            order_type: OrderType::Limit,
            order_status: OrderStatus::Active,
            order_side: OrderSide::Sell,
            user_id: 0,
            price: 10000,
            quantity: 300
        };

        let price_index = order.price as usize;

        let add_order_result = order_book.add_order(order.clone());

        order.order_status = OrderStatus::Active;

        let order_index = order_book.index_mappings[&order.order_id];

        assert!(add_order_result.is_ok());
        assert_eq!(order_book.asks[price_index].len(), 1);
        assert_eq!(order_book.asks[price_index][0], order_index);

        let mut modified_order = order.clone();
        modified_order.quantity = 500;

        let modify_order_result = order_book.modify_order(order.order_id, modified_order.clone());

        let buy_order_index = order_book.index_mappings[&order.order_id];

        assert!(modify_order_result.is_ok());
        assert_eq!(order_book.asks[price_index].len(), 1);
        assert_eq!(order_book.order_ledger[buy_order_index], modified_order);
    }

    #[test]
    fn test_execute_fill_by_order_type_correctly_fills_limit_order_no_remaining_quantity() {
        let config = FixedPriceOrderBookConfig {
            min_price: 0,
            max_price: 10000,
            tick_size: 1,
            queue_size: 100
        };
        let mut order_book = FixedPriceOrderBook::new(config);

        let mut sell_order = Order {
            order_id: 0,
            order_type: OrderType::Limit,
            order_status: OrderStatus::PendingNew,
            order_side: OrderSide::Sell,
            user_id: 0,
            price: 10000,
            quantity: 300
        };

        let buy_order = Order {
            order_id: 1,
            order_type: OrderType::Limit,
            order_status: OrderStatus::PendingNew,
            order_side: OrderSide::Buy,
            user_id: 1,
            price: 10000,
            quantity: 300
        };

        let price_index = sell_order.price as usize;

        let add_order_result = order_book.add_order(sell_order.clone());

        sell_order.order_status = OrderStatus::Active;

        let sell_order_index = order_book.index_mappings[&sell_order.order_id];

        assert!(add_order_result.is_ok());
        assert_eq!(order_book.asks[price_index].len(), 1);
        assert_eq!(order_book.asks[price_index][0], sell_order_index);

        let execute_fill_by_order_type_result = order_book.execute_fill_by_order_type(buy_order.clone());

        assert!(execute_fill_by_order_type_result.is_ok());
        assert!(order_book.asks[price_index].is_empty());
        assert!(order_book.bids[price_index].is_empty());
        assert_eq!(order_book.trade_history.len(), 1);
        assert_eq!(order_book.trade_history[0].aggressive_order_id, buy_order.order_id);
        assert_eq!(order_book.trade_history[0].resting_order_id, sell_order.order_id);
        assert_eq!(order_book.trade_history[0].quantity, 300);
    }

    #[test]
    fn test_execute_fill_by_order_type_correctly_fills_limit_order_with_remaining_quantity() {
        let config = FixedPriceOrderBookConfig {
            min_price: 0,
            max_price: 10000,
            tick_size: 1,
            queue_size: 100
        };
        let mut order_book = FixedPriceOrderBook::new(config);

        let mut sell_order = Order {
            order_id: 0,
            order_type: OrderType::Limit,
            order_status: OrderStatus::PendingNew,
            order_side: OrderSide::Sell,
            user_id: 0,
            price: 10000,
            quantity: 300
        };

        let buy_order = Order {
            order_id: 1,
            order_type: OrderType::Limit,
            order_status: OrderStatus::PendingNew,
            order_side: OrderSide::Buy,
            user_id: 1,
            price: 10000,
            quantity: 600
        };

        let price_index = sell_order.price as usize;

        let add_order_result = order_book.add_order(sell_order.clone());

        sell_order.order_status = OrderStatus::Active;

        let sell_order_index = order_book.index_mappings[&sell_order.order_id];

        assert!(add_order_result.is_ok());
        assert_eq!(order_book.asks[price_index].len(), 1);
        assert_eq!(order_book.asks[price_index][0], sell_order_index);

        let execute_fill_by_order_type_result = order_book.execute_fill_by_order_type(buy_order.clone());

        let buy_order_index = order_book.index_mappings[&buy_order.order_id];

        assert!(execute_fill_by_order_type_result.is_ok());
        assert_eq!(order_book.bids[price_index].len(), 1);
        assert_eq!(order_book.order_ledger[buy_order_index].quantity, 300);
        assert!(order_book.asks[price_index].is_empty());
        assert_eq!(order_book.trade_history.len(), 1);
        assert_eq!(order_book.trade_history[0].aggressive_order_id, buy_order.order_id);
        assert_eq!(order_book.trade_history[0].resting_order_id, sell_order.order_id);
        assert_eq!(order_book.trade_history[0].quantity, 300);
    }

    #[test]
    fn test_execute_fill_by_order_type_correctly_fills_market_order() {
        let config = FixedPriceOrderBookConfig {
            min_price: 0,
            max_price: 10000,
            tick_size: 1,
            queue_size: 100
        };
        let mut order_book = FixedPriceOrderBook::new(config);

        let mut sell_order = Order {
            order_id: 0,
            order_type: OrderType::Limit,
            order_status: OrderStatus::PendingNew,
            order_side: OrderSide::Sell,
            user_id: 0,
            price: 10000,
            quantity: 600
        };

        let buy_order = Order {
            order_id: 1,
            order_type: OrderType::Market,
            order_status: OrderStatus::PendingNew,
            order_side: OrderSide::Buy,
            user_id: 1,
            price: 10000,
            quantity: 300
        };

        let price_index = sell_order.price as usize;

        let add_order_result = order_book.add_order(sell_order.clone());

        sell_order.order_status = OrderStatus::Active;

        let sell_order_index = order_book.index_mappings[&sell_order.order_id];

        assert!(add_order_result.is_ok());
        assert_eq!(order_book.asks[price_index].len(), 1);
        assert_eq!(order_book.asks[price_index][0], sell_order_index);

        let execute_fill_by_order_type_result = order_book.execute_fill_by_order_type(buy_order.clone());

        assert!(execute_fill_by_order_type_result.is_ok());
        assert_eq!(order_book.asks[price_index].len(), 1);
        assert_eq!(order_book.order_ledger[sell_order_index].quantity, 300);
        assert!(order_book.bids[price_index].is_empty());
        assert_eq!(order_book.trade_history.len(), 1);
        assert_eq!(order_book.trade_history[0].aggressive_order_id, buy_order.order_id);
        assert_eq!(order_book.trade_history[0].resting_order_id, sell_order.order_id);
        assert_eq!(order_book.trade_history[0].quantity, 300);
    }

    #[test]
    fn test_execute_fill_by_order_type_fills_part_of_market_order_and_errors_insufficient_liquidity() {
        let config = FixedPriceOrderBookConfig {
            min_price: 0,
            max_price: 10000,
            tick_size: 1,
            queue_size: 100
        };
        let mut order_book = FixedPriceOrderBook::new(config);

        let mut sell_order = Order {
            order_id: 0,
            order_type: OrderType::Limit,
            order_status: OrderStatus::PendingNew,
            order_side: OrderSide::Sell,
            user_id: 0,
            price: 10000,
            quantity: 300
        };

        let buy_order = Order {
            order_id: 1,
            order_type: OrderType::Market,
            order_status: OrderStatus::PendingNew,
            order_side: OrderSide::Buy,
            user_id: 1,
            price: 10000,
            quantity: 600
        };

        let price_index = sell_order.price as usize;

        let add_order_result = order_book.add_order(sell_order.clone());

        sell_order.order_status = OrderStatus::Active;

        let sell_order_index = order_book.index_mappings[&sell_order.order_id];

        assert!(add_order_result.is_ok());
        assert_eq!(order_book.asks[price_index].len(), 1);
        assert_eq!(order_book.asks[price_index][0], sell_order_index);

        let execute_fill_by_order_type_result = order_book.execute_fill_by_order_type(buy_order.clone());

        assert!(execute_fill_by_order_type_result.is_err());
        assert_eq!(execute_fill_by_order_type_result.err().unwrap(), OrderBookError::InsufficientLiquidity);
        assert!(order_book.asks[price_index].is_empty());
        assert!(order_book.bids[price_index].is_empty());
        assert_eq!(order_book.trade_history.len(), 1);
        assert_eq!(order_book.trade_history[0].aggressive_order_id, buy_order.order_id);
        assert_eq!(order_book.trade_history[0].resting_order_id, sell_order.order_id);
        assert_eq!(order_book.trade_history[0].quantity, 300);
    }

    #[test]
    fn test_execute_fill_by_order_type_correctly_fills_immediate_or_cancel_order() {
        let config = FixedPriceOrderBookConfig {
            min_price: 0,
            max_price: 10000,
            tick_size: 1,
            queue_size: 100
        };
        let mut order_book = FixedPriceOrderBook::new(config);

        let mut sell_order = Order {
            order_id: 0,
            order_type: OrderType::Limit,
            order_status: OrderStatus::PendingNew,
            order_side: OrderSide::Sell,
            user_id: 0,
            price: 10000,
            quantity: 600
        };

        let buy_order = Order {
            order_id: 1,
            order_type: OrderType::ImmediateOrCancel,
            order_status: OrderStatus::PendingNew,
            order_side: OrderSide::Buy,
            user_id: 1,
            price: 10000,
            quantity: 300
        };

        let price_index = sell_order.price as usize;

        let add_order_result = order_book.add_order(sell_order.clone());

        sell_order.order_status = OrderStatus::Active;

        let sell_order_index = order_book.index_mappings[&sell_order.order_id];

        assert!(add_order_result.is_ok());
        assert_eq!(order_book.asks[price_index].len(), 1);
        assert_eq!(order_book.asks[price_index][0], sell_order_index);

        let execute_fill_by_order_type_result = order_book.execute_fill_by_order_type(buy_order.clone());

        assert!(execute_fill_by_order_type_result.is_ok());
        assert_eq!(order_book.asks[price_index].len(), 1);
        assert_eq!(order_book.order_ledger[sell_order_index].quantity, 300);
        assert!(order_book.bids[price_index].is_empty());
        assert_eq!(order_book.trade_history.len(), 1);
        assert_eq!(order_book.trade_history[0].aggressive_order_id, buy_order.order_id);
        assert_eq!(order_book.trade_history[0].resting_order_id, sell_order.order_id);
        assert_eq!(order_book.trade_history[0].quantity, 300);
    }

    #[test]
    fn test_execute_fill_by_order_type_correctly_cancels_immediate_or_cancel_order_if_no_resting_order_exists() {
        let config = FixedPriceOrderBookConfig {
            min_price: 0,
            max_price: 10000,
            tick_size: 1,
            queue_size: 100
        };
        let mut order_book = FixedPriceOrderBook::new(config);

        let buy_order = Order {
            order_id: 1,
            order_type: OrderType::ImmediateOrCancel,
            order_status: OrderStatus::PendingNew,
            order_side: OrderSide::Buy,
            user_id: 1,
            price: 10000,
            quantity: 300
        };

        let price_index = buy_order.price as usize;

        let execute_fill_by_order_type_result = order_book.execute_fill_by_order_type(buy_order.clone());

        assert!(execute_fill_by_order_type_result.is_ok());
        assert!(order_book.asks[price_index].is_empty());
        assert!(order_book.bids[price_index].is_empty());
        assert!(order_book.trade_history.is_empty());
    }

    #[test]
    fn test_execute_fill_by_order_type_correctly_fills_fill_or_kill_order() {
        let config = FixedPriceOrderBookConfig {
            min_price: 0,
            max_price: 10000,
            tick_size: 1,
            queue_size: 100
        };
        let mut order_book = FixedPriceOrderBook::new(config);

        let mut sell_order = Order {
            order_id: 0,
            order_type: OrderType::Limit,
            order_status: OrderStatus::PendingNew,
            order_side: OrderSide::Sell,
            user_id: 0,
            price: 10000,
            quantity: 600
        };

        let buy_order = Order {
            order_id: 1,
            order_type: OrderType::FillOrKill,
            order_status: OrderStatus::PendingNew,
            order_side: OrderSide::Buy,
            user_id: 1,
            price: 10000,
            quantity: 300
        };

        let price_index = sell_order.price as usize;

        let add_order_result = order_book.add_order(sell_order.clone());

        sell_order.order_status = OrderStatus::Active;

        let sell_order_index = order_book.index_mappings[&sell_order.order_id];

        assert!(add_order_result.is_ok());
        assert_eq!(order_book.asks[price_index].len(), 1);
        assert_eq!(order_book.asks[price_index][0], sell_order_index);

        let execute_fill_by_order_type_result = order_book.execute_fill_by_order_type(buy_order.clone());

        assert!(execute_fill_by_order_type_result.is_ok());
        assert_eq!(order_book.asks[price_index].len(), 1);
        assert_eq!(order_book.order_ledger[sell_order_index].quantity, 300);
        assert!(order_book.bids[price_index].is_empty());
        assert_eq!(order_book.trade_history.len(), 1);
        assert_eq!(order_book.trade_history[0].aggressive_order_id, buy_order.order_id);
        assert_eq!(order_book.trade_history[0].resting_order_id, sell_order.order_id);
        assert_eq!(order_book.trade_history[0].quantity, 300);
    }

    #[test]
    fn test_execute_fill_by_order_type_errors_cannot_fill_completely() {
        let config = FixedPriceOrderBookConfig {
            min_price: 0,
            max_price: 10000,
            tick_size: 1,
            queue_size: 100
        };
        let mut order_book = FixedPriceOrderBook::new(config);

        let mut sell_order = Order {
            order_id: 0,
            order_type: OrderType::Limit,
            order_status: OrderStatus::PendingNew,
            order_side: OrderSide::Sell,
            user_id: 0,
            price: 10000,
            quantity: 300
        };

        let buy_order = Order {
            order_id: 1,
            order_type: OrderType::FillOrKill,
            order_status: OrderStatus::PendingNew,
            order_side: OrderSide::Buy,
            user_id: 1,
            price: 10000,
            quantity: 600
        };

        let price_index = sell_order.price as usize;

        let add_order_result = order_book.add_order(sell_order.clone());

        sell_order.order_status = OrderStatus::Active;

        let sell_order_index = order_book.index_mappings[&sell_order.order_id];

        assert!(add_order_result.is_ok());
        assert_eq!(order_book.asks[price_index].len(), 1);
        assert_eq!(order_book.asks[price_index][0], sell_order_index);

        let execute_fill_by_order_type_result = order_book.execute_fill_by_order_type(buy_order.clone());

        assert!(execute_fill_by_order_type_result.is_err());
        assert_eq!(execute_fill_by_order_type_result.err().unwrap(), OrderBookError::CannotFillCompletely);
        assert_eq!(order_book.asks[price_index].len(), 1);
        assert_eq!(order_book.order_ledger[sell_order_index].quantity, 300);
        assert!(order_book.bids[price_index].is_empty());
        assert!(order_book.trade_history.is_empty());
    }

    #[test]
    fn test_fill_limit_order_correctly_fills_buy_limit_order() {

    }

    #[test]
    fn test_fill_limit_order_correctly_fills_sell_limit_order() {

    }

    #[test]
    fn test_fill_market_order_correctly_fills_buy_market_order() {

    }

    #[test]
    fn test_fill_market_order_correctly_fills_sell_market_order() {

    }

    #[test]
    fn test_fill_immediate_or_cancel_order_correctly_fills_immediate_or_cancel_order() {

    }

    #[test]
    fn test_fill_fill_or_kill_order_correctly_fills_fill_or_kill_order() {

    }

    #[test]
    fn test_fill_fill_or_kill_order_errors_cannot_fill_completely() {

    }

    #[test]
    fn test_match_order_against_book_correctly_matches_and_fills_buy_order() {

    }

    #[test]
    fn test_match_order_against_book_correctly_matches_and_fills_buy_order_excess_quantity() {

    }

    #[test]
    fn test_match_order_against_book_correctly_matches_and_fills_sell_order() {

    }

    #[test]
    fn test_match_order_against_book_correctly_matches_and_fills_sell_order_excess_quantity() {

    }

    #[test]
    fn test_rest_remaining_limit_order_correctly_rests_buy_limit_order() {

    }

    #[test]
    fn test_rest_remaining_limit_order_correctly_rests_sell_limit_order() {

    }

    #[test]
    fn test_rest_remaining_limit_order_errors_non_limit_order_rest_attempt() {

    }

    #[test]
    fn test_can_fill_completely_correctly_returns_true_for_buy_order_that_can_be_filled_completely() {

    }

    #[test]
    fn test_can_fill_completely_correctly_returns_false_for_buy_order_with_remaining_quantity() {

    }

    #[test]
    fn test_can_fill_completely_correctly_returns_true_for_sell_order_that_can_be_filled_completely() {

    }

    #[test]
    fn test_can_fill_completely_correctly_returns_false_for_sell_order_with_remaining_quantity() {

    }

    #[test]
    fn benchmark() {
        

    }
}