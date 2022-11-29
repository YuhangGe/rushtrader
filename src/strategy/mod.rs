use crate::{Broker, DataSource, Order, Trade};

pub trait Strategy {
  type DS: DataSource;
  type BK: Broker<DS = Self::DS>;
  fn feed(&mut self, data: &Self::DS);
  fn next(&mut self, index: usize, data: &Self::DS, broker: &mut Self::BK);
  fn calc_commission(&self, size: isize, price: f64) -> f64;
  fn on_order(&self, _order: &Order, _broker: &Self::BK) {
    // do nothing by default
  }
  fn on_trade(&self, _trade: &Trade, _broker: &Self::BK) {
    // do nothing by default
  }
  fn on_start(&self, _data: &Self::DS, _broker: &Self::BK) {
    // do nothing by default
  }
  fn on_finish(&self, _data: &Self::DS, _broker: &Self::BK) {
    // do nothing by default
  }
}
