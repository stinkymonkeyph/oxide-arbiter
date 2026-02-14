# oxide-arbiter

A memory-safe order matching engine written in Rust.

oxide-arbiter implements a Centralized Limit Order Book (CLOB) with price-time priority matching. It supports limit and market orders, four time-in-force policies, partial fills, multi-asset matching, and a full trade history — built as a foundation for exchange platforms or trading system integrations.

<img width="1024" height="1024" alt="Gemini_Generated_Image_dwxbg7dwxbg7dwxb" src="https://github.com/user-attachments/assets/99cae915-fe0a-41fe-bca5-093d04dbb277" />

---

## Features

- **Price-time priority matching** — orders at the same price level execute FIFO
- **Limit and market orders** — limit orders execute at a specified price or better; market orders execute at the current best available price
- **Market order slippage protection** — market orders rejected if execution price deviates more than 5% from current market price
- **Four time-in-force policies** — GTC, IOC, FOK, DAY
- **Partial fills** — tracks `quantity_filled` independently; status transitions Open → PartiallyFilled → Closed
- **Multi-asset support** — a single `OrderBookService` manages independent order books per `item_id`
- **O(1) order lookups** — orders stored directly in a `HashMap<Uuid, Order>`
- **Trade history** — every execution recorded with buy/sell order IDs, quantity, price, and timestamp

---

## Architecture

`OrderBookService` uses a layered data structure:

```
orders: HashMap<Uuid, Order>               // source of truth; O(1) lookup by ID

buy_orders:  HashMap<item_id, BTreeMap<Decimal, VecDeque<Uuid>>>
sell_orders: HashMap<item_id, BTreeMap<Decimal, VecDeque<Uuid>>>

trades: Vec<Trade>                          // append-only execution history
```

- The outer `HashMap` partitions the book by asset (`item_id`).
- `BTreeMap` keeps price levels sorted automatically — buy side descending, sell side ascending — so the best price is always at the front.
- `VecDeque` at each price level provides O(1) FIFO insertion and removal, enforcing time priority within a price level.

**Matching flow:**

1. Validate price (non-negative) and quantity (> 0).
2. For market orders: resolve execution price from best opposing price; reject if slippage > 5%.
3. Insert the incoming order into `orders`.
4. Iterate compatible resting orders; for each match:
   - Calculate `min(incoming_remaining, resting_remaining)`.
   - Stage the fill quantity (accumulated per order ID).
   - Create a `Trade` record.
5. Validate time-in-force rules before committing:
   - IOC: if any fills staged, trim order quantity to filled amount and mark Closed.
   - FOK: if staged fills don't cover the full quantity, cancel the order and discard all staged fills and trades.
6. Commit: apply staged fills, remove matched resting orders from the book, and append trades.
7. If the incoming order is fully filled, remove it from the book.

---

## Data Types

### Enums

```rust
enum OrderSide   { Buy, Sell }
enum OrderType   { Limit, Market }
enum OrderStatus { Open, PartiallyFilled, Closed, Cancelled }
enum TimeInForce { GTC, IOC, FOK, DAY }
```

| TimeInForce | Behaviour |
|-------------|-----------|
| `GTC` | Active until cancelled or fully filled |
| `IOC` | Executes immediately; unfilled remainder cancelled |
| `FOK` | Must fill completely or the entire order is cancelled |
| `DAY` | Expires 24 hours after submission |

### Order

| Field | Type | Description |
|-------|------|-------------|
| `id` | `Uuid` | Unique order identifier |
| `item_id` | `Uuid` | Asset identifier |
| `user_id` | `Uuid` | Owner identifier |
| `order_side` | `OrderSide` | Buy or Sell |
| `order_type` | `OrderType` | Limit or Market |
| `time_in_force` | `TimeInForce` | Execution policy |
| `price` | `Decimal` | Limit price (market orders normalized to resting price) |
| `quantity` | `Decimal` | Requested quantity |
| `quantity_filled` | `Decimal` | Executed quantity |
| `status` | `OrderStatus` | Current lifecycle state |
| `created_at` | `DateTime<Utc>` | Creation timestamp |
| `updated_at` | `DateTime<Utc>` | Last modification timestamp |
| `expires_at` | `Option<DateTime<Utc>>` | Expiration (set for DAY/IOC orders) |

### Trade

| Field | Type | Description |
|-------|------|-------------|
| `id` | `Uuid` | Trade identifier |
| `buy_order_id` | `Uuid` | Matched buy order |
| `sell_order_id` | `Uuid` | Matched sell order |
| `item_id` | `Uuid` | Asset matched |
| `quantity` | `Decimal` | Execution size |
| `price` | `Decimal` | Execution price (resting order's price) |
| `timestamp` | `DateTime<Utc>` | Execution timestamp |

### CreateOrderRequest

| Field | Type |
|-------|------|
| `item_id` | `Uuid` |
| `user_id` | `Uuid` |
| `order_side` | `OrderSide` |
| `order_type` | `OrderType` |
| `price` | `Decimal` |
| `quantity` | `Decimal` |
| `time_in_force` | `TimeInForce` |

---

## API Reference

```rust
// Construction
OrderBookService::new() -> Self

// Order submission
add_order(&mut self, req: CreateOrderRequest) -> Result<Order, String>

// Queries
get_orders(&self) -> &HashMap<Uuid, Order>
get_order_by_id(&self, order_id: Uuid) -> Option<&Order>
get_current_market_price(&self, item_id: Uuid, side: OrderSide) -> Option<Decimal>

// Mutations
cancel_order(&mut self, order_id: Uuid) -> bool
update_order_status(&mut self, order_id: Uuid, status: OrderStatus) -> Option<&Order>
update_order_quantity(&mut self, order_id: Uuid, quantity: Decimal) -> Option<&Order>
update_order_price(&mut self, order_id: Uuid, price: Decimal) -> Option<&Order>

// Trade history (public field)
trades: Vec<Trade>
```

**`add_order` validation errors:**

| Error | Condition |
|-------|-----------|
| `"Price cannot be negative"` | `price < 0.0` |
| `"Quantity must be greater than zero"` | `quantity <= 0.0` |
| `"Market order cannot be placed without any existing orders to determine price"` | No opposing liquidity |
| `"Market order price cannot be more than 5% away from the current market price..."` | Slippage exceeded |

---

## Installation

```sh
cargo add oxide-arbiter
```

Or add manually to `Cargo.toml`:

```toml
[dependencies]
oxide-arbiter = "0.1.0-beta.1"
```

---

## Usage

```rust
use oxide_arbiter::{CreateOrderRequest, OrderBookService, OrderSide, OrderType, TimeInForce};
use rust_decimal::Decimal;
use std::str::FromStr;

let mut book = OrderBookService::new();
let asset_id = uuid::Uuid::new_v4();
let user_id = uuid::Uuid::new_v4();

// Resting buy limit order
let buy = book.add_order(CreateOrderRequest {
    item_id: asset_id,
    user_id,
    order_side: OrderSide::Buy,
    order_type: OrderType::Limit,
    time_in_force: TimeInForce::GTC,
    price: Decimal::from_str("100.0").unwrap(),
    quantity: Decimal::from_str("50.0").unwrap(),
}).unwrap();

// Incoming sell limit order — matches immediately
let sell = book.add_order(CreateOrderRequest {
    item_id: asset_id,
    user_id,
    order_side: OrderSide::Sell,
    order_type: OrderType::Limit,
    time_in_force: TimeInForce::GTC,
    price: Decimal::from_str("100.0").unwrap(),
    quantity: Decimal::from_str("50.0").unwrap(),
}).unwrap();

// Inspect executed trades
for trade in &book.trades {
    println!("Trade {} — qty: {} @ {}", trade.id, trade.quantity, trade.price);
}

// Check final order status
let filled = book.get_order_by_id(buy.id).unwrap();
println!("Buy order status: {:?}", filled.status); // Closed
```

---

## Tests

```bash
cargo test
```

15 test cases covering:

- Order CRUD — creation, lookup, status/quantity/price updates, cancellation
- Matching — partial fills, full fills, incompatible price rejection
- Market orders — price discovery, slippage protection, no-liquidity error
- Time-in-force — IOC partial fill behaviour
- Trade recording — trade history integrity

---

## Todos

### Indexing & Queries

Every query other than lookup-by-ID currently requires an O(n) scan of the full order map.

| Item | Detail |
|------|--------|
| Secondary index by `user_id` | Enables `get_orders_by_user(user_id)` — required for per-user position views |
| Secondary index by `item_id` + status | Enables `get_open_orders_for_item(item_id)` — required for efficient book management |
| Order book depth snapshot | `get_depth(item_id, levels)` returning top N bid/ask price levels with aggregated volume at each level |

### Features

| Item | Detail |
|------|--------|
| DAY order expiration enforcement | `expires_at` is set on DAY orders but never checked. Requires an explicit `expire_orders()` sweep to remove stale orders from the book. |
| Stop orders | `StopLoss` and `StopLimit` variants with a trigger price field; activates the order when the market reaches the trigger. |
| Serde support | `#[derive(Serialize, Deserialize)]` on all public types, behind an optional `serde` feature flag. |

### Infrastructure

| Item | Detail |
|------|--------|
| Thread safety | `OrderBookService` is not `Sync`. An `Arc<Mutex<OrderBookService>>` wrapper or a channel-based design is needed for concurrent order acceptance. |
| Benchmarks | No performance benchmarks exist. A `criterion`-based suite would establish baseline throughput and catch regressions. |
