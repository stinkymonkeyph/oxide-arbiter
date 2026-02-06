mod components;

use crate::components::services::OrderBookService;
use components::dto::OrderType;

fn main() {
    let mut order_book = OrderBookService::new();
    order_book.add_order(100.0, OrderType::Buy);
    order_book.add_order(50.0, OrderType::Sell);

    for order_book_order in order_book.get_orders() {
        println!("--- Order Details ---");
        println!("Order ID: {}", order_book_order.id);
        println!("Order Type: {:?}", order_book_order.order_type);
        println!("Order Amount: {}", order_book_order.amount);
        println!("Order Status: {:?}", order_book_order.status);
        println!("Order Created At: {}", order_book_order.created_at);
        println!("Order Updated At: {}", order_book_order.updated_at);
        println!("---------------------");
    }

    println!("OrderBookService created successfully.");
    println!("Hello, world!");
}
