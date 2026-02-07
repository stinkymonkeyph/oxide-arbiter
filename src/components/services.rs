use crate::components::dto::{
    CreateOrderRequest, Order, OrderSide, OrderStatus, OrderType, TimeEnforce, Trade,
};
use chrono::Utc;
use uuid::Uuid;

pub struct OrderBookService {
    orders: Vec<Order>,
    pub trades: Vec<Trade>,
}

impl OrderBookService {
    pub fn new() -> Self {
        OrderBookService {
            orders: Default::default(),
            trades: Default::default(),
        }
    }

    pub fn add_order(&mut self, create_order_request: CreateOrderRequest) -> Result<Order, String> {
        if create_order_request.price < 0.0 {
            return Err("Price cannot be negative".to_string());
        }

        let expires_at = match create_order_request.time_enforce {
            TimeEnforce::DAY => Some(Utc::now() + chrono::Duration::days(1)),
            TimeEnforce::IOC => Some(Utc::now()),
            _ => None,
        };

        let mut order = Order {
            id: Uuid::new_v4(),
            item_id: create_order_request.item_id,
            user_id: create_order_request.user_id,
            order_side: create_order_request.order_side,
            order_type: create_order_request.order_type,
            price: create_order_request.price,
            quantity: create_order_request.quantity,
            quantity_filled: 0.0,
            time_enforce: create_order_request.time_enforce,
            status: OrderStatus::Open,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at,
        };

        if matches!(order.order_type, OrderType::Market) {
            match self.get_current_market_price(order.item_id, order.order_side) {
                Some(market_price) if market_price < order.price => {
                    return Err(
                        "Market order price cannot be lower than or equal to zero".to_string()
                    );
                }
                None => return Err(
                    "Market order cannot be placed without any existing orders to determine price"
                        .to_string(),
                ),
                _ => (),
            }
        }

        self.orders.push(order.clone());
        self.execute_order_matching(&mut order);
        let updated_order = self.get_order_by_id(order.id).unwrap().clone();
        Ok(updated_order)
    }

    pub fn get_orders(&self) -> &Vec<Order> {
        &self.orders
    }

    pub fn get_current_market_price(&self, item_id: Uuid, order_side: OrderSide) -> Option<f32> {
        let matched_orders = self.get_orders().iter().filter(|o| {
            o.item_id == item_id
                && matches!(
                    (&o.order_side, &order_side),
                    (OrderSide::Buy, OrderSide::Sell) | (OrderSide::Sell, OrderSide::Buy)
                )
                && matches!(o.status, OrderStatus::Open | OrderStatus::PartiallyFilled)
        });

        let best_price = match order_side {
            OrderSide::Buy => matched_orders
                .filter(|o| matches!(o.order_side, OrderSide::Sell))
                .map(|o| o.price)
                .min_by(|a, b| a.partial_cmp(b).unwrap()),
            OrderSide::Sell => matched_orders
                .filter(|o| matches!(o.order_side, OrderSide::Buy))
                .map(|o| o.price)
                .max_by(|a, b| a.partial_cmp(b).unwrap()),
        };

        best_price
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
            OrderType::Market => true,
            OrderType::Limit => match incoming.order_side {
                OrderSide::Buy => incoming.price >= resting.price,
                OrderSide::Sell => incoming.price <= resting.price,
            },
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
                        (&o.order_side, &incoming_order.order_side),
                        (OrderSide::Buy, OrderSide::Sell) | (OrderSide::Sell, OrderSide::Buy)
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
                buy_order_id: if matches!(incoming_order.order_side, OrderSide::Buy) {
                    incoming_order.id
                } else {
                    matching_order_id
                },
                sell_order_id: if matches!(incoming_order.order_side, OrderSide::Sell) {
                    incoming_order.id
                } else {
                    matching_order_id
                },
                item_id: incoming_order.item_id,
                quantity: trade_quantity,
                price,
                timestamp: Utc::now(),
            });

            if trades.len() == 0 && matches!(incoming_order.time_enforce, TimeEnforce::FOK) {
                self.cancel_order(incoming_order.id);
                break;
            }

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

        if trades.len() > 0 && matches!(incoming_order.time_enforce, TimeEnforce::IOC) {
            self.update_order_quantity(incoming_order.id, incoming_order.quantity_filled);
            self.update_order_status(incoming_order.id, OrderStatus::Closed);
        }

        self.trades = trades;
    }
}
