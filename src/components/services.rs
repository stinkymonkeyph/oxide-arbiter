use crate::components::dto::Order;

pub struct OrderBookService {
    pub orders: Vec<Order>,
}

impl OrderBookService {
    pub fn new() -> Self {
        OrderBookService {
            orders: Default::default(),
        }
    }
    pub fn add_order(&mut self, order: Order) {
        self.orders.push(order);
    }
}
