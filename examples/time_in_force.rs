use oxide_arbiter::{CreateOrderRequest, OrderBookService, OrderSide, OrderType, TimeInForce};

fn main() {
    println!("=== IOC (Immediate Or Cancel) ===");

    let mut book = OrderBookService::new();
    let item = uuid::Uuid::new_v4();

    book.add_order(CreateOrderRequest {
        item_id: item,
        user_id: uuid::Uuid::new_v4(),
        order_side: OrderSide::Sell,
        order_type: OrderType::Limit,
        price: 10.0,
        quantity: 30.0,
        time_in_force: TimeInForce::GTC,
    })
    .unwrap();

    // IOC buy for 100 — only 30 are available
    let ioc = book
        .add_order(CreateOrderRequest {
            item_id: item,
            user_id: uuid::Uuid::new_v4(),
            order_side: OrderSide::Buy,
            order_type: OrderType::Limit,
            price: 10.0,
            quantity: 100.0,
            time_in_force: TimeInForce::IOC,
        })
        .unwrap();

    println!("IOC order status:        {:?}", ioc.status);
    println!("Quantity requested:      100");
    println!("Quantity filled:         {}", ioc.quantity_filled);
    println!("Quantity after IOC trim: {}", ioc.quantity);
    println!("Trades:                  {}", book.trades.len());

    // --- FOK: Fill Or Kill ---
    println!("\n=== FOK (Fill Or Kill) ===");

    let mut book = OrderBookService::new();
    let item = uuid::Uuid::new_v4();

    // Resting sell at a price the FOK buy cannot reach
    book.add_order(CreateOrderRequest {
        item_id: item,
        user_id: uuid::Uuid::new_v4(),
        order_side: OrderSide::Sell,
        order_type: OrderType::Limit,
        price: 20.0,
        quantity: 100.0,
        time_in_force: TimeInForce::GTC,
    })
    .unwrap();

    // FOK buy at 10.0 — no price match, so zero trades → entire order cancelled
    let fok = book
        .add_order(CreateOrderRequest {
            item_id: item,
            user_id: uuid::Uuid::new_v4(),
            order_side: OrderSide::Buy,
            order_type: OrderType::Limit,
            price: 10.0,
            quantity: 50.0,
            time_in_force: TimeInForce::FOK,
        })
        .unwrap();

    println!("FOK order status:  {:?}", fok.status);
    println!("Quantity filled:   {}", fok.quantity_filled);
    println!("Trades:            {}", book.trades.len());

    // --- GTC: Good Till Cancelled ---
    println!("\n=== GTC (Good Till Cancelled) ===");

    let mut book = OrderBookService::new();
    let item = uuid::Uuid::new_v4();

    let gtc = book
        .add_order(CreateOrderRequest {
            item_id: item,
            user_id: uuid::Uuid::new_v4(),
            order_side: OrderSide::Buy,
            order_type: OrderType::Limit,
            price: 25.0,
            quantity: 50.0,
            time_in_force: TimeInForce::GTC,
        })
        .unwrap();

    println!("GTC order status after placement: {:?}", gtc.status);

    book.cancel_order(gtc.id);
    let after_cancel = book.get_order_by_id(gtc.id).unwrap();
    println!(
        "GTC order status after cancel:    {:?}",
        after_cancel.status
    );
}
