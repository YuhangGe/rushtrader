use chrono::{DateTime, Utc};

#[derive(Debug)]
pub enum OrderType {
  Market,
}
#[derive(Debug)]
pub enum OrderPositionType {
  Buy,
  Sell,
}

#[derive(Debug)]
pub enum OrderStatus {
  Created,
  // Submitted,
  // Accepted,
  // Partial,
  Completed(DateTime<Utc>),
  // Rejected(DateTime<Utc>),
  // Margin,
  // Cancelled,
  // Expired,
  // Error(String),
}

pub struct Order {
  pub ordertype: OrderType,
  pub position_type: OrderPositionType,
  pub size: isize,
  pub status: OrderStatus,
  pub exe_size: isize,
  pub comm: f64,
  pub exe_price: f64,
  pub cost: f64,
  pub created_at: DateTime<Utc>,
  pub completed_at: Option<DateTime<Utc>>,
}

impl Order {
  pub(crate) fn new(
    size: isize,
    position_type: OrderPositionType,
    created_at: DateTime<Utc>,
  ) -> Self {
    Self {
      ordertype: OrderType::Market,
      position_type,
      size,
      status: OrderStatus::Created,
      created_at,
      completed_at: None,
      exe_price: 0.,
      exe_size: 0,
      comm: 0.,
      cost: 0.,
    }
  }
  #[inline]
  pub(crate) fn buy(size: isize, created_at: DateTime<Utc>) -> Self {
    Self::new(size, OrderPositionType::Buy, created_at)
  }
  #[inline]
  pub(crate) fn sell(size: isize, created_at: DateTime<Utc>) -> Self {
    Self::new(size, OrderPositionType::Sell, created_at)
  }
  /// 是否是 buy 类型的订单。false 代表是 sell 类型。
  #[inline]
  pub fn is_buy(&self) -> bool {
    matches!(self.position_type, OrderPositionType::Buy)
  }
}
