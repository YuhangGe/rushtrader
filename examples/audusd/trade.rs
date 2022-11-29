use chrono::{DateTime, Utc};

use crate::{bool_map, strat::VegasStrategy};

pub(super) enum VegasTradeOrderType {
  Open,
  Close,
  MidnightOpen,
  MidnightClose,
  None,
}
pub(super) struct VegasTradeInfo {
  pub order_type: VegasTradeOrderType,
  pub open_time: Option<DateTime<Utc>>,
  pub open_price: Option<f64>,
  pub open_position: isize,
  pub prices: Vec<f64>,
}

impl VegasTradeInfo {
  pub fn new() -> Self {
    Self {
      order_type: VegasTradeOrderType::None,
      open_time: None,
      open_price: None,
      open_position: 0,
      prices: Vec::new(),
    }
  }
}
pub(crate) fn complete_trade(trade: &mut VegasTradeInfo, dt: &DateTime<Utc>) {
  let open_position = trade.open_position;
  let open_time = trade.open_time;
  let prices = &trade.prices;

  let mut trans = Vec::new();
  let mut sd = 0.;
  for (pre, nxt) in prices
    .iter()
    .step_by(2)
    .zip(prices.iter().skip(1).step_by(2))
  {
    let d = bool_map!(open_position > 0, 1., -1.) * (nxt - pre);
    sd += d;
    trans.push(format!(
      "{:.6}, {:.6}, {}{:.6}",
      pre,
      nxt,
      bool_map!(d >= 0., "+", ""),
      d
    ));
  }
  let trade_r = (sd / prices[0] * 10000.).round() / 100.;
  println!(
    "[INFO] [{}] TRADE FINISHED. Summary:
  Type: {}
  Open at: {}
  Transactions:
    {}
  Close at: {}
  {} is: {}%",
    dt,
    bool_map!(open_position > 0, "long", "short"),
    open_time.unwrap(),
    trans.join("\n"),
    dt,
    bool_map!(trade_r > 0., "Profit-take", "Stop-loss"),
    trade_r
  );
  trade.order_type = VegasTradeOrderType::None;
}
impl VegasStrategy {
  pub(crate) fn complete_trade(&mut self, dt: &DateTime<Utc>) {
    complete_trade(unsafe { &mut *self.p_trade }, dt);
  }
}
