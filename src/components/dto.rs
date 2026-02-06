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
    Closed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: Uuid,
    pub order_type: OrderType,
    pub amount: f32,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
