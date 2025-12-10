use std::{collections::HashSet, time::Instant};

use rust_decimal::{Decimal, dec, prelude::FromPrimitive};

use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::{enums::{order_side::OrderSide, order_status::OrderStatus, order_type::OrderType}, fixed_price_order_book::FixedPriceOrderBook, models::{fixed_price_order_book_config::FixedPriceOrderBookConfig, order::Order}, traits::order_book::TOrderBook};

pub mod dynamic_price_order_book;
pub mod enums;
pub mod fixed_price_order_book;
pub mod macros;
pub mod models;
pub mod traits;
pub mod utils;

fn main() {
    check_add_order_latencies();
}

fn check_add_order_latencies() {
    let config = FixedPriceOrderBookConfig {
        min_price: 0,
        max_price: 1_000_00,
        tick_size: 1,
    };

    let mut order_book = FixedPriceOrderBook::new(config);

    let num_orders = 1_000_000;
    let price_levels = 1_000;
    let base_ticks = 5000; // ~ $50.00 midpoint

    let mut rng = StdRng::seed_from_u64(12345);

    // -------------------------------------------------
    // Pre-create all Order structs and track prices
    // -------------------------------------------------
    let mut orders = Vec::with_capacity(num_orders);

    let mut min_tick = i32::MAX;
    let mut max_tick = i32::MIN;
    let mut tick_set = HashSet::<i32>::new();

    for i in 0..num_orders {
        let side = if rng.random::<bool>() {
            OrderSide::Buy
        } else {
            OrderSide::Sell
        };

        let offset: i32 =
            rng.random_range(-price_levels as i32 / 2 .. price_levels as i32 / 2);

        let price_ticks = (base_ticks as i32 + offset).max(1);
        let price = (base_ticks as i32 + offset).max(1) as u32;

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
        Decimal::from_i32(min_tick).unwrap() * dec!(0.01),
        Decimal::from_i32(max_tick).unwrap() * dec!(0.01)
    );

    // -------------------------------------------------
    // Benchmark with latency collection
    // -------------------------------------------------
    let mut latencies = Vec::with_capacity(num_orders);

    let total_start = Instant::now();

    for order in orders {
        let start = Instant::now();
        order_book.add_order(order).unwrap();
        let end = Instant::now();
        latencies.push((end - start).as_nanos() as u64);
    }

    let total_end = Instant::now();

    // -------------------------------------------------
    // Compute percentiles
    // -------------------------------------------------
    latencies.sort_unstable();

    println!("\nLatency Statistics:");
    benchmark_percentiles("fill_order", std::mem::take(&mut order_book.bench_stats.fill_order));
    benchmark_percentiles("add_order", std::mem::take(&mut order_book.bench_stats.add_order));
    benchmark_percentiles("execute_fill_by_order_type", std::mem::take(&mut order_book.bench_stats.execute_fill_by_order_type));

    benchmark_percentiles("fill_limit_order", std::mem::take(&mut order_book.bench_stats.fill_limit_order));
    benchmark_percentiles("fill_market_order", std::mem::take(&mut order_book.bench_stats.fill_market_order));
    benchmark_percentiles("fill_immediate_or_cancel_order", std::mem::take(&mut order_book.bench_stats.fill_immediate_or_cancel_order));
    benchmark_percentiles("fill_fill_or_kill_order", std::mem::take(&mut order_book.bench_stats.fill_fill_or_kill_order));

    benchmark_percentiles("match_order_against_book", std::mem::take(&mut order_book.bench_stats.match_order_against_book));
    benchmark_percentiles("rest_remaining_limit_order", std::mem::take(&mut order_book.bench_stats.rest_remaining_limit_order));
    benchmark_percentiles("can_fill_completely", std::mem::take(&mut order_book.bench_stats.can_fill_completely));

    println!("Total time elapsed: {}ms", (total_end - total_start).as_millis());
}

fn benchmark_percentiles(name: &str, mut data: Vec<u64>) {
    if data.is_empty() {
        println!("{}: no samples", name);
        return;
    }

    data.sort_unstable();

    let n = data.len();
    let p50 = data[n * 50 / 100];
    let p90 = data[n * 90 / 100];
    let p99 = data[n * 99 / 100];
    let avg = data.iter().sum::<u64>() / n as u64;

    println!(
        "{:<32}  p50: {:>6} ns  p90: {:>6} ns  p99: {:>6} ns  avg: {:>6} ns  samples: {}",
        name, p50, p90, p99, avg, n
    );
}