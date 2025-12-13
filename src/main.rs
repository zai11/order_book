use std::{collections::HashSet, time::Instant};

use rand::{Rng, SeedableRng, rngs::StdRng};
use rand_distr::{Normal, Distribution};

use crate::{enums::{order_side::OrderSide, order_status::OrderStatus, order_type::OrderType}, models::{order::Order, order_book_config::OrderBookConfig}, order_book::OrderBook};

pub mod enums;
pub mod models;
pub mod order_book;
pub mod utils;

fn main() {
    check_add_order_latencies();
}

fn check_add_order_latencies() {
    let config = OrderBookConfig {
        min_price: 0,
        max_price: 10_000_00,
        tick_size: 1,
        queue_size: 1000,
    };

    let mut order_book = OrderBook::new(config);

    let num_orders = 1_000_000;
    let base_ticks = 5000; // ~ $50.00 midpoint

    let mut rng = StdRng::seed_from_u64(12345);

    // Gaussian around base_ticks with std deviation ~10 ticks
    let normal = Normal::new(base_ticks as f64, 10.0).unwrap();

    // Pre-create all orders and track price levels
    let mut orders = Vec::with_capacity(num_orders);
    let mut min_tick = i32::MAX;
    let mut max_tick = i32::MIN;
    let mut tick_set = HashSet::<i32>::new();

    for i in 0..num_orders {
        let side = if rng.random_bool(0.5) {
            OrderSide::Buy
        } else {
            OrderSide::Sell
        };

        // Generate Gaussian price offset
        let mut price_ticks = normal.sample(&mut rng).round() as i32;
        price_ticks = price_ticks.max(1); // Ensure price >= 1
        let price = price_ticks as u32;

        let qty = rng.random_range(1..1000);

        // Track price-level range
        min_tick = min_tick.min(price_ticks);
        max_tick = max_tick.max(price_ticks);
        tick_set.insert(price_ticks);

        orders.push(Order {
            order_id: i as u64,
            order_type: OrderType::Limit,
            order_status: OrderStatus::PendingNew,
            order_side: side,
            user_id: rng.random_range(0..1000),
            price,
            quantity: qty,
        });
    }

    println!("Price tick range: {} → {}", min_tick, max_tick);
    println!("Total distinct price levels: {}", tick_set.len());
    println!(
        "Approx. price range: ${:.2} → ${:.2}",
        min_tick as f32 * 0.01,
        max_tick as f32 * 0.01
    );

    let mut latencies = Vec::with_capacity(num_orders);
    let total_start = Instant::now();

    for order in orders {
        let start = Instant::now();
        order_book.add_order(order).unwrap();
        let end = Instant::now();
        latencies.push((end - start).as_nanos() as u64);
    }

    let total_end = Instant::now();
    latencies.sort_unstable();

    let n = latencies.len();
    let p50 = latencies[n * 50 / 100];
    let p90 = latencies[n * 90 / 100];
    let p99 = latencies[n * 99 / 100];
    let avg = latencies.iter().sum::<u64>() / n as u64;

    println!("\nLatency Statistics:");
    println!("p50: {p50}ns\tp90: {p90}ns\tp99: {p99}ns\tavg: {avg}ns\tsamples: {n}");
    println!("Total time elapsed: {}ms", (total_end - total_start).as_millis());
}