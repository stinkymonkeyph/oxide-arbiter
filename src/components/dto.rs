use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub enum OrderType {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy)]
pub enum OrderStatus {
    Open,
    PartiallyFilled,
    Closed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: Uuid,
    pub item_id: Uuid,
    pub user_id: Uuid,
    pub order_type: OrderType,
    pub price: f32,
    pub quantity: f32,
    pub quantity_filled: f32,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Trade {
    pub id: Uuid,
    pub buy_order_id: Uuid,
    pub sell_order_id: Uuid,
    pub item_id: Uuid,
    pub quantity: f32,
    pub price: f32,
    pub timestamp: chrono::DateTime<Utc>,
}

pub struct CreateOrderRequest {
    pub item_id: Uuid,
    pub user_id: Uuid,
    pub order_type: OrderType,
    pub price: f32,
    pub quantity: f32,
}
