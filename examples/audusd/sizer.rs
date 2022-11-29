use rushtrader::{Broker, CsvBroker};

use crate::{overnight::OvernightType, strat::VegasStrategy};

const FIXED_SIZE: isize = 5000000;

impl VegasStrategy {
  pub fn get_sizer(&mut self, broker: &CsvBroker) -> isize {
    if !matches!(self.overnight.overtype, OvernightType::None) {
      return self.overnight.position;
    }
    if broker.position_size() != 0 {
      self.overnight.position = broker.position_size().abs();
    } else {
      self.overnight.position = FIXED_SIZE;
    }
    self.overnight.position
  }
}
