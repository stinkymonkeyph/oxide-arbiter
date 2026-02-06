mod components;

use chrono::Utc;
use components::dto::{Order, OrderStatus, OrderType};

use crate::components::services::OrderBookService;

fn main() {
    let mut order_book = OrderBookService::new();
    let order_time = Utc::now();
    order_book.add_order(Order {
        order_type: OrderType::Buy,
        amount: 100.0,
        status: OrderStatus::Open,
        created_at: order_time,
        updated_at: order_time,
    });

    order_book.add_order(Order {
        order_type: OrderType::Sell,
        amount: 50.0,
        status: OrderStatus::Open,
        created_at: order_time,
        updated_at: order_time,
    });

    for order_book_order in &order_book.orders {
        println!("--- Order Details ---");
        println!("Order Type: {:?}", order_book_order.order_type);
        println!("Order Amount: {}", order_book_order.amount);
        println!("Order Status: {:?}", order_book_order.status);
        println!("Order Created At: {}", order_book_order.created_at);
        println!("Order Updated At: {}", order_book_order.updated_at);
        println!("---------------------");
    }

    println!("{:?}", order_book.orders);
    println!("OrderBookService created successfully.");
    println!("Hello, world!");
}
