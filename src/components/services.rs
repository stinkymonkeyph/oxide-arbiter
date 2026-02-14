use std::{
    cmp::min,
    collections::{BTreeMap, HashMap, VecDeque},
    str::FromStr,
};

use crate::components::dto::{
    CreateOrderRequest, Order, OrderSide, OrderStatus, OrderType, TimeInForce, Trade,
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

pub struct OrderBookService {
    orders: HashMap<Uuid, Order>,
    buy_orders: HashMap<Uuid, BTreeMap<Decimal, VecDeque<Uuid>>>,
    sell_orders: HashMap<Uuid, BTreeMap<Decimal, VecDeque<Uuid>>>,
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
        if create_order_request.price < Decimal::ZERO {
            return Err("Price cannot be negative".to_string());
        }

        if create_order_request.quantity <= Decimal::ZERO {
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
            quantity_filled: Decimal::ZERO,
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
                        _ => Decimal::ZERO,
                    };

                    if price_difference > (order.price * Decimal::from_str("0.05").unwrap()) {
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
                        .entry(updated_order.price)
                        .or_default()
                        .push_back(updated_order.id);
                }
                OrderSide::Sell => {
                    self.sell_orders
                        .entry(updated_order.item_id)
                        .or_default()
                        .entry(updated_order.price)
                        .or_default()
                        .push_back(updated_order.id);
                }
            }
        }

        Ok(updated_order)
    }

    pub fn get_orders(&self) -> &HashMap<Uuid, Order> {
        &self.orders
    }

    fn is_expired(&self, expires_at: Option<DateTime<Utc>>) -> bool {
        match expires_at {
            Some(expiry) => {
                let now = Utc::now();
                expiry < now
            }
            None => false,
        }
    }

    pub fn get_current_market_price(
        &self,
        item_id: Uuid,
        order_side: OrderSide,
    ) -> Option<Decimal> {
        let price_map = match order_side {
            OrderSide::Buy => self.sell_orders.get(&item_id)?,
            OrderSide::Sell => self.buy_orders.get(&item_id)?,
        };

        match order_side {
            OrderSide::Buy => price_map.iter().next().map(|(price, _)| *price),

            OrderSide::Sell => price_map.iter().next_back().map(|(price, _)| *price),
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
            self.remove_from_book(order_id);
            true
        } else {
            false
        }
    }

    pub fn update_order_quantity(
        &mut self,
        order_id: Uuid,
        new_quantity: Decimal,
    ) -> Option<&Order> {
        if let Some(order) = self.orders.get_mut(&order_id) {
            order.quantity = new_quantity;
            order.updated_at = Utc::now();
            Some(order)
        } else {
            None
        }
    }

    pub fn update_order_price(&mut self, order_id: Uuid, new_price: Decimal) -> Option<&Order> {
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
        let price = order.price;
        let side = order.order_side;

        let book = match side {
            OrderSide::Buy => &mut self.buy_orders,
            OrderSide::Sell => &mut self.sell_orders,
        };

        if let Some(price_map) = book.get_mut(&item_id) {
            if let Some(order_queue) = price_map.get_mut(&price) {
                order_queue.retain(|order_id_from_queue| *order_id_from_queue != order_id);

                if order_queue.is_empty() {
                    price_map.remove(&price);
                }
            }

            if price_map.is_empty() {
                book.remove(&item_id);
            }
        }
    }

    fn fill_order(&mut self, order_id: Uuid, quantity_filled: Decimal) -> Option<&mut Order> {
        let is_fully_filled = if let Some(order) = self.get_mutable_order_by_id(order_id) {
            order.quantity_filled += quantity_filled;

            if order.quantity_filled >= order.quantity {
                order.status = OrderStatus::Closed;
                true
            } else {
                order.status = OrderStatus::PartiallyFilled;
                false
            }
        } else {
            return None;
        };

        if is_fully_filled {
            self.remove_from_book(order_id);
        }

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
        let order_book_side = match incoming_order.order_side {
            OrderSide::Buy => &self.sell_orders,
            OrderSide::Sell => &self.buy_orders,
        };

        let price_maps = match order_book_side.get(&incoming_order.item_id) {
            Some(item) => item,
            _ => {
                return;
            }
        };

        let prices: Vec<Decimal> = match incoming_order.order_side {
            OrderSide::Buy => price_maps.keys().cloned().collect(),
            OrderSide::Sell => price_maps.keys().cloned().rev().collect(),
        };

        let mut queue_orders: Vec<(Decimal, Uuid)> = Vec::new();

        for price in prices {
            let order_queue = price_maps.get(&price);
            for order_id in order_queue.unwrap() {
                queue_orders.push((price, *order_id));
            }
        }

        let mut trades: Vec<Trade> = Vec::new();
        let mut staged_order_to_fill: HashMap<Uuid, Decimal> = HashMap::new();

        for (price, order_id) in queue_orders {
            let resting_order = self.get_order_by_id(order_id);

            if !resting_order.is_some() {
                continue;
            }

            let resting_order = resting_order.unwrap();

            if self.is_expired(resting_order.expires_at)
                && matches!(resting_order.time_in_force, TimeInForce::DAY)
            {
                self.remove_from_book(resting_order.id);
                continue;
            }

            if !self.can_match_price(incoming_order, resting_order) {
                break;
            }

            let available_quantity = resting_order.quantity - resting_order.quantity_filled;
            if available_quantity <= Decimal::ZERO {
                continue;
            }

            let quantity_to_match = incoming_order.quantity - incoming_order.quantity_filled;
            let trade_quantity = min(available_quantity, quantity_to_match);

            let trade_id: Uuid = Uuid::new_v4();

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
                price,
                timestamp: Utc::now(),
            });

            *staged_order_to_fill
                .entry(resting_order.id)
                .or_insert(Decimal::ZERO) += trade_quantity;
            *staged_order_to_fill
                .entry(incoming_order.id)
                .or_insert(Decimal::ZERO) += trade_quantity;

            incoming_order.quantity_filled += trade_quantity;

            if incoming_order.quantity_filled == incoming_order.quantity {
                break;
            }
        }

        let mut unstaged_matched_orders = false;

        if trades.len() > 0 && matches!(incoming_order.time_in_force, TimeInForce::IOC) {
            self.update_order_quantity(incoming_order.id, incoming_order.quantity_filled);
            self.update_order_status(incoming_order.id, OrderStatus::Closed);
        }

        if matches!(incoming_order.time_in_force, TimeInForce::FOK)
            && incoming_order.quantity_filled != incoming_order.quantity
        {
            self.cancel_order(incoming_order.id);
            unstaged_matched_orders = true;
        }

        if !unstaged_matched_orders {
            for (order_id, trade_quantity) in staged_order_to_fill {
                self.fill_order(order_id, trade_quantity);
            }
            self.trades.append(&mut trades);
        }

        if incoming_order.quantity_filled == incoming_order.quantity {
            self.remove_from_book(incoming_order.id);
        }
    }
}

