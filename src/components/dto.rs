use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum TimeEnforce {
    GTC, // Good Till Cancelled
    IOC, // Immediate Or Cancel
    FOK, // Fill Or Kill
    DAY, // Day Order
}

#[derive(Debug, Clone, Copy)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum OrderStatus {
    Open,
    PartiallyFilled,
    Closed,
    Cancelled,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Order {
    pub id: Uuid,
    pub item_id: Uuid,
    pub user_id: Uuid,
    pub order_side: OrderSide,
    pub order_type: OrderType,
    pub time_enforce: TimeEnforce,
    pub price: f32,
    pub quantity: f32,
    pub quantity_filled: f32,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Trade {
    pub id: Uuid,
    pub buy_order_id: Uuid,
    pub sell_order_id: Uuid,
    pub item_id: Uuid,
    pub quantity: f32,
    pub price: f32,
    pub timestamp: chrono::DateTime<Utc>,
}

#[allow(dead_code)]
pub struct CreateOrderRequest {
    pub item_id: Uuid,
    pub user_id: Uuid,
    pub order_side: OrderSide,
    pub order_type: OrderType,
    pub price: f32,
    pub quantity: f32,
    pub time_enforce: TimeEnforce,
}
