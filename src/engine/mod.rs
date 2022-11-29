use crate::data::DataSource;
use crate::{Broker, Strategy};

pub struct Engine<D: DataSource, B: Broker<DS = D>, S: Strategy<DS = D, BK = B>> {
  data: D,
  strategy: S,
  broker: B,
}

impl<D, B, S> Engine<D, B, S>
where
  D: DataSource,
  B: Broker<DS = D>,
  S: Strategy<DS = D, BK = B>,
{
  pub fn new(data: D, strategy: S, broker: B) -> Self {
    Self {
      data,
      strategy,
      broker,
    }
  }
  pub fn run(&mut self) {
    self.strategy.on_start(&self.data, &self.broker);
    while self.data.read(&mut self.strategy, &mut self.broker) {}
    self.strategy.on_finish(&self.data, &self.broker);
  }
}
