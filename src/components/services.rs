use crate::components::dto::{CreateOrderRequest, Order, OrderSide, OrderStatus, OrderType, Trade};
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
        let mut order = Order {
            id: Uuid::new_v4(),
            item_id: create_order_request.item_id,
            user_id: create_order_request.user_id,
            order_side: create_order_request.order_side,
            order_type: create_order_request.order_type,
            price: create_order_request.price,
            quantity: create_order_request.quantity,
            quantity_filled: 0.0,
            status: OrderStatus::Open,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        if matches!(order.order_type, OrderType::Market) {
            if let Some(market_price) =
                self.get_current_market_price(order.item_id, order.order_side)
            {
                order.price = market_price;
            }

            if order.price == 0.0 {
                return Err(
                    "Market order cannot be placed without any existing orders to determine price"
                        .to_string(),
                );
            }
        }

        self.orders.push(order.clone());
        self.execute_order_matching(&mut order);
        Ok(order)
    }

    pub fn get_orders(&self) -> &Vec<Order> {
        &self.orders
    }

    pub fn get_current_market_price(&self, item_id: Uuid, order_side: OrderSide) -> Option<f32> {
        let relevant_orders: Vec<&Order> = self
            .orders
            .iter()
            .filter(|o| {
                o.item_id == item_id
                    && matches!(
                        (&o.order_side, &order_side),
                        (OrderSide::Buy, OrderSide::Sell) | (OrderSide::Sell, OrderSide::Buy)
                    )
                    && matches!(o.status, OrderStatus::Open | OrderStatus::PartiallyFilled)
            })
            .collect();

        if relevant_orders.is_empty() {
            None
        } else {
            let best_order = match order_side {
                OrderSide::Buy => relevant_orders
                    .iter()
                    .max_by(|a, b| a.price.partial_cmp(&b.price).unwrap()),
                OrderSide::Sell => relevant_orders
                    .iter()
                    .min_by(|a, b| a.price.partial_cmp(&b.price).unwrap()),
            };
            best_order.map(|o| o.price)
        }
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

        self.trades = trades;
    }
}
