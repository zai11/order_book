use dashmap::DashMap;

use crate::{enums::{order_book_errors::OrderBookError, symbol::Symbol}, models::{order::Order, order_book_config::OrderBookConfig}, order_book::OrderBook};

pub struct OrderBookManager {
    pub books: DashMap<Symbol, OrderBook>,
    pub order_id_symbol_mapping: DashMap<u64, Symbol>
}

impl OrderBookManager {
    pub fn new() -> Self {
        Self {
            books: DashMap::new(),
            order_id_symbol_mapping: DashMap::new()
        }
    }

    pub fn add_symbol(&mut self, symbol: Symbol, config: OrderBookConfig) {
        self.books.insert(symbol, OrderBook::new(config));
    }

    pub fn add_order(&mut self, symbol: Symbol, order: Order) -> Result<(), OrderBookError> {
        let mut book = self.books.get_mut(&symbol)
            .ok_or(OrderBookError::SymbolNotFound(symbol.clone()))?;

        self.order_id_symbol_mapping.insert(order.order_id, symbol);

        book.add_order(order)
    }

    pub fn cancel_order(&mut self, order_id: u64) -> Result<(), OrderBookError> {
        let symbol = self.order_id_symbol_mapping.get(&order_id)
            .ok_or(OrderBookError::OrderNotFound)?;

        let mut book = self.books.get_mut(&*symbol)
            .ok_or(OrderBookError::SymbolNotFound(symbol.to_owned()))?;

        book.cancel_order(order_id)?;
        self.order_id_symbol_mapping.remove(&order_id);

        Ok(())
    }

    pub fn get_bbo(&self, symbol: Symbol) -> Option<(Option<u32>, Option<u32>)> {
        self.books.get(&symbol).map(|book| (
            match book.best_bid_index {
                Some(best_bid) => Some(best_bid as u32),
                None => None
            }, 
            match book.best_ask_index {
                Some(best_ask) => Some(best_ask as u32),
                None => None
            }))
    }
}