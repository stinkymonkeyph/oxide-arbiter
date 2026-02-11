use std::{
    cmp::min,
    collections::{BTreeMap, HashMap, VecDeque},
};

use crate::components::dto::{
    CreateOrderRequest, Order, OrderSide, OrderStatus, OrderType, TimeInForce, Trade,
};
use chrono::Utc;
use ordered_float::OrderedFloat;
use uuid::Uuid;

pub struct OrderBookService {
    orders: HashMap<Uuid, Order>,
    buy_orders: HashMap<Uuid, BTreeMap<OrderedFloat<f32>, VecDeque<Order>>>,
    sell_orders: HashMap<Uuid, BTreeMap<OrderedFloat<f32>, VecDeque<Order>>>,
    /// All trades executed since the service was created. Appended to on each `add_order` call.
    pub trades: Vec<Trade>,
}

impl OrderBookService {
    pub fn new() -> Self {
        OrderBookService {
            orders: Default::default(),
            buy_orders: Default::default(),
            sell_orders: Default::default(),
            trades: Default::default(),
        }
    }

    pub fn add_order(&mut self, create_order_request: CreateOrderRequest) -> Result<Order, String> {
        if create_order_request.price < 0.0 {
            return Err("Price cannot be negative".to_string());
        }

        if create_order_request.quantity <= 0.0 {
            return Err("Quantity must be greater than zero".to_string());
        }

        let expires_at = match create_order_request.time_in_force {
            TimeInForce::DAY => Some(Utc::now() + chrono::Duration::days(1)),
            TimeInForce::IOC => Some(Utc::now()),
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
            time_in_force: create_order_request.time_in_force,
            status: OrderStatus::Open,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at,
        };

        if matches!(order.order_type, OrderType::Market) {
            match self.get_current_market_price(order.item_id, order.order_side) {
                Some(market_price) => {
                    let price_difference = match order.order_side {
                        OrderSide::Buy if market_price > order.price => market_price - order.price,
                        OrderSide::Sell if market_price < order.price => order.price - market_price,
                        _ => 0.0,
                    };

                    if price_difference > (order.price * 0.05) {
                        return Err(format!(
                            "Market order price cannot be more than 5% away from the current market price. Current market price: {}, Order price: {}",
                            market_price, order.price
                        ));
                    }
                    order.price = market_price;
                }
                None => return Err(
                    "Market order cannot be placed without any existing orders to determine price"
                        .to_string(),
                ),
            }
        }

        self.orders.insert(order.id, order.clone());
        self.execute_order_matching(&mut order);

        let updated_order = self.get_order_by_id(order.id).unwrap().clone();

        if matches!(
            updated_order.status,
            OrderStatus::Open | OrderStatus::PartiallyFilled
        ) {
            match updated_order.order_side {
                OrderSide::Buy => {
                    self.buy_orders
                        .entry(updated_order.item_id)
                        .or_default()
                        .entry(OrderedFloat(updated_order.price))
                        .or_default()
                        .push_back(updated_order.clone());
                }
                OrderSide::Sell => {
                    self.sell_orders
                        .entry(updated_order.item_id)
                        .or_default()
                        .entry(OrderedFloat(updated_order.price))
                        .or_default()
                        .push_back(updated_order.clone());
                }
            }
        }

        Ok(updated_order)
    }

    pub fn get_orders(&self) -> &HashMap<Uuid, Order> {
        &self.orders
    }

    pub fn get_current_market_price(&self, item_id: Uuid, order_side: OrderSide) -> Option<f32> {
        let price_map = match order_side {
            OrderSide::Buy => self.sell_orders.get(&item_id)?,
            OrderSide::Sell => self.buy_orders.get(&item_id)?,
        };

        match order_side {
            OrderSide::Buy => price_map
                .iter()
                .next()
                .map(|(ordered_price, _)| ordered_price.0),

            OrderSide::Sell => price_map
                .iter()
                .next_back()
                .map(|(ordered_price, _)| ordered_price.0),
        }
    }

    pub fn get_order_by_id(&self, order_id: Uuid) -> Option<&Order> {
        self.orders.get(&order_id)
    }

    pub fn get_mutable_order_by_id(&mut self, order_id: Uuid) -> Option<&mut Order> {
        self.orders.get_mut(&order_id)
    }

    pub fn update_order_status(
        &mut self,
        order_id: Uuid,
        new_status: OrderStatus,
    ) -> Option<&Order> {
        if let Some(order) = self.get_mutable_order_by_id(order_id) {
            order.status = new_status;
            order.updated_at = Utc::now();
            Some(order)
        } else {
            None
        }
    }

    pub fn cancel_order(&mut self, order_id: Uuid) -> bool {
        if let Some(order) = self.get_mutable_order_by_id(order_id) {
            order.status = OrderStatus::Cancelled;
            order.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    pub fn update_order_quantity(&mut self, order_id: Uuid, new_quantity: f32) -> Option<&Order> {
        if let Some(order) = self.orders.get_mut(&order_id) {
            order.quantity = new_quantity;
            order.updated_at = Utc::now();
            Some(order)
        } else {
            None
        }
    }

    pub fn update_order_price(&mut self, order_id: Uuid, new_price: f32) -> Option<&Order> {
        if let Some(order) = self.get_mutable_order_by_id(order_id) {
            order.price = new_price;
            order.updated_at = Utc::now();
            Some(order)
        } else {
            None
        }
    }

    fn remove_from_book(&mut self, order_id: Uuid) {
        let order = match self.get_order_by_id(order_id) {
            Some(order) => order.clone(),
            None => return,
        };

        let item_id = order.item_id;
        let price = OrderedFloat(order.price);
        let side = order.order_side;

        let book = match side {
            OrderSide::Buy => &mut self.buy_orders,
            OrderSide::Sell => &mut self.sell_orders,
        };

        if let Some(price_map) = book.get_mut(&item_id) {
            if let Some(order_queue) = price_map.get_mut(&price) {
                order_queue.retain(|o| o.id != order_id);

                if order_queue.is_empty() {
                    price_map.remove(&price);
                }
            }

            if price_map.is_empty() {
                book.remove(&item_id);
            }
        }
    }

    fn fill_order(&mut self, order_id: Uuid, quantity_filled: f32) -> Option<&mut Order> {
        if let Some(order) = self.get_mutable_order_by_id(order_id) {
            order.quantity_filled += quantity_filled;

            if order.quantity_filled >= order.quantity {
                order.status = OrderStatus::Closed;
            } else {
                order.status = OrderStatus::PartiallyFilled;
            }

            order.updated_at = Utc::now();
            order.quantity_filled >= order.quantity
        } else {
            return None;
        };

        self.get_mutable_order_by_id(order_id)
    }

    fn can_match_price(&self, incoming: &Order, resting: &Order) -> bool {
        match (incoming.order_type, incoming.order_side) {
            (OrderType::Market, _) => true,
            (OrderType::Limit, OrderSide::Buy) => incoming.price >= resting.price,
            (OrderType::Limit, OrderSide::Sell) => incoming.price <= resting.price,
        }
    }

    pub fn execute_order_matching(&mut self, incoming_order: &mut Order) {
        let mut trades: Vec<Trade> = Vec::new();

        let order_book_side = match incoming_order.order_side {
            OrderSide::Buy => self.sell_orders.clone(),
            OrderSide::Sell => self.buy_orders.clone(),
        };

        let price_maps = match order_book_side.get(&incoming_order.item_id) {
            Some(item) => item,
            _ => {
                return;
            }
        };

        let prices: Vec<OrderedFloat<f32>> = match incoming_order.order_side {
            OrderSide::Buy => price_maps.keys().cloned().collect(),
            OrderSide::Sell => price_maps.keys().cloned().rev().collect(),
        };

        let mut matched_trade_list: HashMap<usize, Order> = HashMap::new();

        for price in &prices {
            let order_queue = &price_maps[price];

            for resting_order in order_queue {
                let resting_order = self.get_order_by_id(resting_order.id);

                if !resting_order.is_some() {
                    break;
                }

                let resting_order = resting_order.unwrap();
                let resting_order_snapshot = resting_order.clone();

                let is_match = self.can_match_price(incoming_order, resting_order);

                if !is_match {
                    break;
                }

                let available_quantity = resting_order.quantity - resting_order.quantity_filled;
                if available_quantity <= 0.0 {
                    break;
                }

                let quantity_to_match = incoming_order.quantity - incoming_order.quantity_filled;
                let trade_quantity = min(
                    OrderedFloat(available_quantity),
                    OrderedFloat(quantity_to_match),
                )
                .into_inner();

                if trade_quantity <= 0.0 {
                    break;
                }

                let trade_id = Uuid::new_v4();
                let trade_index = trades.len();

                trades.push(Trade {
                    id: trade_id,
                    buy_order_id: if matches!(incoming_order.order_side, OrderSide::Buy) {
                        incoming_order.id
                    } else {
                        resting_order.id
                    },
                    sell_order_id: if matches!(incoming_order.order_side, OrderSide::Sell) {
                        incoming_order.id
                    } else {
                        resting_order.id
                    },
                    item_id: incoming_order.item_id,
                    quantity: trade_quantity,
                    price: price.into_inner(),
                    timestamp: Utc::now(),
                });

                matched_trade_list.insert(trade_index, resting_order_snapshot);

                self.fill_order(resting_order.id, trade_quantity);
                self.fill_order(incoming_order.id, trade_quantity);
                if let Some(order) = self.get_mutable_order_by_id(incoming_order.id) {
                    incoming_order.quantity = order.quantity;
                    incoming_order.quantity_filled = order.quantity_filled;
                }
            }
        }

        let updated_incoming_order = self.get_order_by_id(incoming_order.id);
        let updated_incoming_order = updated_incoming_order.unwrap().clone();
        let mut performed_reversal = false;

        if trades.len() > 0 && matches!(incoming_order.time_in_force, TimeInForce::IOC) {
            self.update_order_quantity(incoming_order.id, updated_incoming_order.quantity_filled);
            self.update_order_status(incoming_order.id, OrderStatus::Closed);
        }

        if trades.len() == 0 && matches!(incoming_order.time_in_force, TimeInForce::FOK)
            || matches!(incoming_order.time_in_force, TimeInForce::FOK)
                && updated_incoming_order.quantity_filled != updated_incoming_order.quantity
        {
            self.cancel_order(incoming_order.id);
            performed_reversal = true;
            for (trade_index, _) in matched_trade_list.clone().into_iter() {
                self.trades.remove(trade_index);
            }
        }

        if !performed_reversal {
            for (_, order) in matched_trade_list {
                self.remove_from_book(order.id);
            }
        }

        if incoming_order.quantity_filled == incoming_order.quantity {
            self.remove_from_book(incoming_order.id);
        }

        self.trades.append(&mut trades);
    }
}
