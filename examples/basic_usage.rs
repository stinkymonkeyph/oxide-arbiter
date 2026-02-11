use oxide_arbiter::{CreateOrderRequest, OrderBookService, OrderSide, OrderType, TimeInForce};

fn main() {
    let mut order_book = OrderBookService::new();
    let _ = order_book.add_order(CreateOrderRequest {
        item_id: uuid::Uuid::new_v4(),
        user_id: uuid::Uuid::new_v4(),
        order_side: OrderSide::Buy,
        order_type: OrderType::Limit,
        time_in_force: TimeInForce::DAY,
        price: 10.0,
        quantity: 100.0,
    });

    let _ = order_book.add_order(CreateOrderRequest {
        item_id: uuid::Uuid::new_v4(),
        user_id: uuid::Uuid::new_v4(),
        order_side: OrderSide::Sell,
        order_type: OrderType::Limit,
        time_in_force: TimeInForce::DAY,
        price: 12.0,
        quantity: 50.0,
    });

    for (_, order_book_order) in order_book.get_orders() {
        println!("--- Order Details ---");
        println!("Order ID: {}", order_book_order.id);
        println!("Item ID: {}", order_book_order.item_id);
        println!("User ID: {}", order_book_order.user_id);
        println!("Order Type: {:?}", order_book_order.order_type);
        println!("Order quantity: {}", order_book_order.quantity);
        println!("Order Status: {:?}", order_book_order.status);
        println!("Order Created At: {}", order_book_order.created_at);
        println!("Order Updated At: {}", order_book_order.updated_at);
        println!("---------------------");
    }

    println!("OrderBookService created successfully.");
    println!("Hello, world!");
}
