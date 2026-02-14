use oxide_arbiter::{
    CreateOrderRequest, OrderBookService, OrderSide, OrderStatus, OrderType, TimeInForce,
};
use rust_decimal::Decimal;
use std::str::FromStr;

fn print_orders(book: &OrderBookService) {
    let mut orders: Vec<_> = book.get_orders().values().collect();
    orders.sort_by_key(|o| o.created_at);
    for order in orders {
        println!(
            "  [{:?}] {:?} {:?} — {}/{} units @ {} — created {}",
            order.status,
            order.order_side,
            order.order_type,
            order.quantity_filled,
            order.quantity,
            order.price,
            order.created_at.format("%H:%M:%S%.3f"),
        );
    }
}

fn main() {
    let mut book = OrderBookService::new();
    println!("=== Full Fill ===");
    let item_a = uuid::Uuid::new_v4();
    book.add_order(CreateOrderRequest {
        item_id: item_a,
        user_id: uuid::Uuid::new_v4(),
        order_side: OrderSide::Sell,
        order_type: OrderType::Limit,
        price: Decimal::from_str("50.0").unwrap(),
        quantity: Decimal::from_str("100.0").unwrap(),
        time_in_force: TimeInForce::GTC,
    })
    .unwrap();
    book.add_order(CreateOrderRequest {
        item_id: item_a,
        user_id: uuid::Uuid::new_v4(),
        order_side: OrderSide::Buy,
        order_type: OrderType::Limit,
        price: Decimal::from_str("50.0").unwrap(),
        quantity: Decimal::from_str("100.0").unwrap(),
        time_in_force: TimeInForce::GTC,
    })
    .unwrap();
    println!("Trades produced:");
    for trade in &book.trades {
        println!(
            "  trade {} — {} units @ {}",
            trade.id, trade.quantity, trade.price
        );
    }
    println!("\nOrders:");
    print_orders(&book);
    // --- Partial fill ---
    println!("\n=== Partial Fill ===");
    let item_b = uuid::Uuid::new_v4();
    book.add_order(CreateOrderRequest {
        item_id: item_b,
        user_id: uuid::Uuid::new_v4(),
        order_side: OrderSide::Buy,
        order_type: OrderType::Limit,
        price: Decimal::from_str("30.0").unwrap(),
        quantity: Decimal::from_str("200.0").unwrap(),
        time_in_force: TimeInForce::GTC,
    })
    .unwrap();
    // Sell fills only part of the resting buy — buy stays PartiallyFilled
    book.add_order(CreateOrderRequest {
        item_id: item_b,
        user_id: uuid::Uuid::new_v4(),
        order_side: OrderSide::Sell,
        order_type: OrderType::Limit,
        price: Decimal::from_str("30.0").unwrap(),
        quantity: Decimal::from_str("80.0").unwrap(),
        time_in_force: TimeInForce::GTC,
    })
    .unwrap();
    println!("Trades produced:");
    for trade in &book.trades {
        println!(
            "  trade {} — {} units @ {}",
            trade.id, trade.quantity, trade.price
        );
    }
    println!("\nOrders:");
    print_orders(&book);
    // --- Summary: filter by fill status ---
    println!("\n=== Filled Orders ===");
    let mut closed: Vec<_> = book
        .get_orders()
        .values()
        .filter(|o| matches!(o.status, OrderStatus::Closed))
        .collect();
    closed.sort_by_key(|o| o.created_at);
    println!("Fully filled ({}):", closed.len());
    for order in &closed {
        println!(
            "  {:?} {} units @ {}",
            order.order_side, order.quantity_filled, order.price
        );
    }
    let mut partial: Vec<_> = book
        .get_orders()
        .values()
        .filter(|o| matches!(o.status, OrderStatus::PartiallyFilled))
        .collect();
    partial.sort_by_key(|o| o.created_at);
    println!("Partially filled ({}):", partial.len());
    for order in &partial {
        println!(
            "  {:?} {}/{} units @ {} (remaining: {})",
            order.order_side,
            order.quantity_filled,
            order.quantity,
            order.price,
            order.quantity - order.quantity_filled,
        );
    }
    let open_count = book
        .get_orders()
        .values()
        .filter(|o| matches!(o.status, OrderStatus::Open))
        .count();
    println!("Open: {open_count}");
}

