use chrono::{DateTime, Utc};

#[derive(Debug)]
pub enum TradeStatus {
  Open(DateTime<Utc>),
  Closed(DateTime<Utc>),
  Uninit,
}

pub struct Trade {
  pub pnl: f64,
  pub pnlcomm: f64,
  pub status: TradeStatus,
}

impl Trade {
  pub(crate) fn new() -> Self {
    Self {
      status: TradeStatus::Uninit,
      pnl: 0.,
      pnlcomm: 0.,
      // status_watchers: Vec::new()
    }
  }
}
