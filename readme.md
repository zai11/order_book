# Ultra-Low Latency Rust Order Book Engine

A high-performance, concurrent order book engine written in Rust, focused on achieving sub-microsecond order processing latencies.

This project is primarily an exploration of data-structure design, cache locality, and hot-path optimisation.

---

## Performance Results

Performance is reported separately for a single order book and for the multi-symbol order book manager, to make the cost of symbol routing and concurrent map access explicit.

### Single Order Book (No Symbol Routing)

Benchmarked using 1,000,000 orders on a single order book, with prices sampled from a Gaussian distribution tightly clustered around a midpoint.

```
p50: 100ns
p90: 300ns
p99: 500ns
avg: 242ns
samples: 1,000,000
```

This measures the core matching and resting logic only:

- Price-level lookup
- Order matching
- Queue operations

### Order Book Manager (10 Symbols)

Benchmarked using 1,000,000 orders, uniformly distributed across 10 symbols.

```
p50: 400ns
p90: 700ns
p99: 2100ns
avg: 820ns
samples: 1,000,000
```

These measurements include:

- Symbol routing
- DashMap shard locking
- Per-symbol order book access
- Order matching and/or resting logic

---

## Core Design

### Fixed Price-Level Book

- Prices are represented as integer ticks
- Direct indexing into price levels
- Optimised for dense spreads typical of liquid markets

### Order Storage

- Orders are stored in a slab, with price-level queues holding slab indices
- Avoids cloning large order structs
- Enables stable references and efficient cancellation

## Multi-Symbol Support

Multiple symbols are handled via a central `OrderBookManager`, with one independent order book per symbol.

### Symbol Representation

```rust
pub enum Symbol {
    AAPL,
    MSFT,
    GOOGL,
    AMZN,
    TSLA,
    ...
}
```

Using a compact `enum` instead of `String` keys:

- Eliminates heap allocation
- Reduces hashing cost
- Improves cache locality

This change contributed to an average latency reduction of \~200ns.

## Benchmark Methodology

- **Orders:** 1,000,000
- **Symbols:** 10 (for multi-symbol benchmark)
- **Price distribution:** Gaussian (σ = 10 ticks)
- **Observed spread:** ~90 distinct price levels
- **Quantity:** Uniform random [1, 1000]
- **Timing:** `std::time::Instant` around the add-order hot path only

This setup reflects the tight price clustering seen in liquid equity markets, rather than uniformly spreading orders across thousands of price levels.

### Machine Specifications

- **CPU:** Intel Core Ultra 5 238V
- **Architecture:** x86\_64
- **RAM:** 32 GB DDR4 (8533 MT/s)
- **OS:** Windows
- Hooked to Rust `release` builds with optimisations enabled

---

## Summary

This project represents a focused exploration of low-latency order book design, approached from the perspective of an engineer optimising a real system rather than showcasing a language or framework.

The emphasis throughout is on:
- identifying and addressing performance bottlenecks through measurement rather than intuition
- making deliberate data-structure and memory-layout trade-offs in latency-sensitive code
- iterating on the hot path with a focus on predictability and simplicity

Sub-microsecond latencies are achieved through incremental refinement of the hot path, careful handling of memory and indirection, and a willingness to remove abstraction when it becomes measurable overhead. The result is a codebase that prioritises clarity and reproducibility alongside performance, reflecting the kinds of trade-offs encountered in production-grade trading infrastructure.

