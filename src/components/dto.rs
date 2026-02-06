use chrono::{DateTime, Utc};

#[derive(Debug)]
pub enum OrderType {
    Buy,
    Sell,
}

#[derive(Debug)]
pub enum OrderStatus {
    Open,
    Closed,
    Cancelled,
}

#[derive(Debug)]
pub struct Order {
    pub order_type: OrderType,
    pub amount: f32,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
