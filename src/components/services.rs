use crate::components::dto::{Order, OrderStatus, OrderType};
use chrono::Utc;
use uuid::Uuid;

pub struct OrderBookService {
    orders: Vec<Order>,
}

impl OrderBookService {
    pub fn new() -> Self {
        OrderBookService {
            orders: Default::default(),
        }
    }

    pub fn add_order(&mut self, amount: f32, order_type: OrderType) -> Order {
        let order = Order {
            id: Uuid::new_v4(),
            order_type,
            amount,
            status: OrderStatus::Open,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.orders.push(order.clone());
        order
    }

    pub fn get_orders(&self) -> &Vec<Order> {
        &self.orders
    }
}
