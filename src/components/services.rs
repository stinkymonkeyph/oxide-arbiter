use crate::components::dto::{CreateOrderRequest, Order, OrderStatus, OrderType, Trade};
use chrono::Utc;
use uuid::Uuid;

pub struct OrderBookService {
    orders: Vec<Order>,
    trades: Vec<Trade>,
}

impl OrderBookService {
    pub fn new() -> Self {
        OrderBookService {
            orders: Default::default(),
            trades: Default::default(),
        }
    }

    pub fn add_order(&mut self, create_order_request: CreateOrderRequest) -> Order {
        let mut order = Order {
            id: Uuid::new_v4(),
            item_id: create_order_request.item_id,
            user_id: create_order_request.user_id,
            order_type: create_order_request.order_type,
            price: create_order_request.price,
            quantity: create_order_request.quantity,
            quantity_filled: 0.0,
            status: OrderStatus::Open,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.orders.push(order.clone());
        self.execute_order_matching(&mut order);
        order
    }

    pub fn get_orders(&self) -> &Vec<Order> {
        &self.orders
    }

    pub fn get_order_by_id(&self, order_id: Uuid) -> Option<&Order> {
        self.orders.iter().find(|order| order.id == order_id)
    }

    pub fn update_order_status(
        &mut self,
        order_id: Uuid,
        new_status: OrderStatus,
    ) -> Option<&Order> {
        if let Some(order) = self.orders.iter_mut().find(|order| order.id == order_id) {
            order.status = new_status;
            order.updated_at = Utc::now();
            Some(order)
        } else {
            None
        }
    }

    pub fn cancel_order(&mut self, order_id: Uuid) -> bool {
        if let Some(order) = self.orders.iter_mut().find(|order| order.id == order_id) {
            order.status = OrderStatus::Cancelled;
            order.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    pub fn update_order_quantity(&mut self, order_id: Uuid, new_quantity: f32) -> Option<&Order> {
        if let Some(order) = self.orders.iter_mut().find(|order| order.id == order_id) {
            order.quantity = new_quantity;
            order.updated_at = Utc::now();
            Some(order)
        } else {
            None
        }
    }

    pub fn update_order_price(&mut self, order_id: Uuid, new_price: f32) -> Option<&Order> {
        if let Some(order) = self.orders.iter_mut().find(|order| order.id == order_id) {
            order.price = new_price;
            order.updated_at = Utc::now();
            Some(order)
        } else {
            None
        }
    }

    fn fill_order(&mut self, order_id: Uuid, quantity_filled: f32) -> Option<&Order> {
        if let Some(order) = self.orders.iter_mut().find(|order| order.id == order_id) {
            order.quantity_filled += quantity_filled;
            if order.quantity_filled >= order.quantity {
                order.status = OrderStatus::Closed;
            } else {
                order.status = OrderStatus::PartiallyFilled;
            }
            order.updated_at = Utc::now();
            Some(order)
        } else {
            None
        }
    }

    fn can_match_price(&self, incoming: &Order, resting: &Order) -> bool {
        match incoming.order_type {
            OrderType::Buy => incoming.price >= resting.price,
            OrderType::Sell => incoming.price <= resting.price,
        }
    }

    pub fn execute_order_matching(&mut self, incoming_order: &mut Order) {
        let mut trades: Vec<Trade> = Vec::new();

        let mut matching_order_ids: Vec<(Uuid, f32, f32, chrono::DateTime<Utc>)> = self
            .orders
            .iter()
            .filter(|o| {
                o.item_id == incoming_order.item_id
                    && matches!(
                        (&o.order_type, &incoming_order.order_type),
                        (OrderType::Buy, OrderType::Sell) | (OrderType::Sell, OrderType::Buy)
                    )
                    && matches!(o.status, OrderStatus::Open | OrderStatus::PartiallyFilled)
                    && self.can_match_price(&incoming_order, o)
            })
            .map(|o| {
                (
                    o.id,
                    o.quantity,
                    o.quantity - o.quantity_filled,
                    o.created_at,
                )
            })
            .collect();

        matching_order_ids.sort_by_key(|(_, _, _, created_at)| *created_at);

        for (matching_order_id, available_quantity, price, _) in matching_order_ids {
            if incoming_order.quantity_filled >= incoming_order.quantity {
                break;
            }

            let trade_quantity =
                available_quantity.min(incoming_order.quantity - incoming_order.quantity_filled);
            if trade_quantity <= 0.0 {
                continue;
            }

            trades.push(Trade {
                id: Uuid::new_v4(),
                buy_order_id: if matches!(incoming_order.order_type, OrderType::Buy) {
                    incoming_order.id
                } else {
                    matching_order_id
                },
                sell_order_id: if matches!(incoming_order.order_type, OrderType::Sell) {
                    incoming_order.id
                } else {
                    matching_order_id
                },
                item_id: incoming_order.item_id,
                quantity: trade_quantity,
                price,
                timestamp: Utc::now(),
            });

            if let Some(matching_order) = self
                .orders
                .clone()
                .iter_mut()
                .find(|o| o.id == matching_order_id.clone())
            {
                matching_order.quantity_filled += trade_quantity;
                self.fill_order(matching_order.id.clone(), matching_order.quantity_filled);
            }

            incoming_order.quantity_filled += trade_quantity;
            self.fill_order(incoming_order.id, incoming_order.quantity_filled);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_add_order() {
        let mut order_book = OrderBookService::new();
        let create_order_request = CreateOrderRequest {
            item_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            order_type: OrderType::Buy,
            price: 10.0,
            quantity: 100.0,
        };
        let order = order_book.add_order(create_order_request);
        assert_eq!(order.quantity, 100.0);
        assert_eq!(matches!(order.order_type, OrderType::Buy), true);
        assert_eq!(matches!(order.status, OrderStatus::Open), true);
    }

    #[test]
    fn should_get_order_by_id() {
        let mut order_book = OrderBookService::new();
        let create_order_request = CreateOrderRequest {
            item_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            order_type: OrderType::Sell,
            price: 20.0,
            quantity: 50.0,
        };
        let order = order_book.add_order(create_order_request);
        let fetched_order = order_book.get_order_by_id(order.id);
        assert!(fetched_order.is_some());
        assert_eq!(fetched_order.unwrap().id, order.id);
    }

    #[test]
    fn should_update_order_status() {
        let mut order_book = OrderBookService::new();
        let create_order_request = CreateOrderRequest {
            item_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            order_type: OrderType::Buy,
            price: 15.0,
            quantity: 100.0,
        };
        let order = order_book.add_order(create_order_request);
        let updated_order = order_book.update_order_status(order.id, OrderStatus::Closed);
        assert!(updated_order.is_some());
        assert_eq!(
            matches!(updated_order.unwrap().status, OrderStatus::Closed),
            true
        );
    }

    #[test]
    fn should_update_order_quantity() {
        let mut order_book = OrderBookService::new();
        let create_order_request = CreateOrderRequest {
            item_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            order_type: OrderType::Sell,
            price: 25.0,
            quantity: 50.0,
        };
        let order = order_book.add_order(create_order_request);
        let updated_order = order_book.update_order_quantity(order.id, 75.0);
        assert!(updated_order.is_some());
        assert_eq!(updated_order.unwrap().quantity, 75.0);
    }

    #[test]
    fn should_cancel_order() {
        let mut order_book = OrderBookService::new();
        let create_order_request = CreateOrderRequest {
            item_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            order_type: OrderType::Buy,
            price: 30.0,
            quantity: 100.0,
        };
        let order = order_book.add_order(create_order_request);
        order_book.cancel_order(order.id);
        let fetched_order = order_book.get_order_by_id(order.id);
        assert!(matches!(
            fetched_order.unwrap().status,
            OrderStatus::Cancelled
        ));
    }

    #[test]
    fn should_be_partially_filled() {
        let mut order_book = OrderBookService::new();
        let item_id = Uuid::new_v4();

        let buy_order_request = CreateOrderRequest {
            item_id,
            user_id: Uuid::new_v4(),
            order_type: OrderType::Buy,
            price: 10.0,
            quantity: 100.0,
        };
        let buy_order = order_book.add_order(buy_order_request);

        let sell_order_request = CreateOrderRequest {
            item_id,
            user_id: Uuid::new_v4(),
            order_type: OrderType::Sell,
            price: 10.0,
            quantity: 50.0,
        };
        let sell_order = order_book.add_order(sell_order_request);

        let fetched_buy_order = order_book.get_order_by_id(buy_order.id).unwrap();
        let fetched_sell_order = order_book.get_order_by_id(sell_order.id).unwrap();

        assert_eq!(fetched_buy_order.quantity_filled, 50.0);
        assert_eq!(fetched_sell_order.quantity_filled, 50.0);
        assert_eq!(
            matches!(fetched_buy_order.status, OrderStatus::PartiallyFilled),
            true
        );
        assert_eq!(
            matches!(fetched_sell_order.status, OrderStatus::Closed),
            true
        );
    }

    #[test]
    fn should_fully_filled() {
        let mut order_book = OrderBookService::new();
        let item_id = Uuid::new_v4();

        let buy_order_request = CreateOrderRequest {
            item_id,
            user_id: Uuid::new_v4(),
            order_type: OrderType::Buy,
            price: 10.0,
            quantity: 100.0,
        };
        let buy_order = order_book.add_order(buy_order_request);

        let sell_order_request = CreateOrderRequest {
            item_id,
            user_id: Uuid::new_v4(),
            order_type: OrderType::Sell,
            price: 10.0,
            quantity: 100.0,
        };
        let sell_order = order_book.add_order(sell_order_request);

        let fetched_buy_order = order_book.get_order_by_id(buy_order.id).unwrap();
        let fetched_sell_order = order_book.get_order_by_id(sell_order.id).unwrap();

        assert_eq!(fetched_buy_order.quantity_filled, 100.0);
        assert_eq!(fetched_sell_order.quantity_filled, 100.0);
        assert_eq!(
            matches!(fetched_buy_order.status, OrderStatus::Closed),
            true
        );
        assert_eq!(
            matches!(fetched_sell_order.status, OrderStatus::Closed),
            true
        );
    }
}
