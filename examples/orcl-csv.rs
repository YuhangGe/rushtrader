use std::{cell::RefCell, time::Instant};

use rushtrader::{
  Broker, CsvBroker, CsvDataSource, CsvTimeType, DataLine, DataSource, Engine, Order, OrderStatus,
  SMAIndicator, Strategy, Trade, TradeStatus,
};

struct DemoStrategy {
  size: isize,
  pnlcomm: RefCell<f64>,
  sma: SMAIndicator,
}
impl DemoStrategy {
  fn new() -> Self {
    DemoStrategy {
      size: 10,
      pnlcomm: RefCell::new(0.),
      sma: SMAIndicator::new(15),
    }
  }
}
impl Strategy for DemoStrategy {
  type DS = CsvDataSource;
  type BK = CsvBroker;

  fn calc_commission(&self, size: isize, price: f64) -> f64 {
    (size.abs() as f64) * price * 0.001
  }
  fn on_start(&self, _data: &CsvDataSource, broker: &CsvBroker) {
    println!("Starting Portfolio Value: {}", broker.cash());
  }
  fn on_finish(&self, data: &CsvDataSource, broker: &CsvBroker) {
    println!("Final cash: {}", broker.cash());
    println!("Final position: {}", broker.position());
    println!("Final pnlmm: {}", self.pnlcomm.borrow());
    println!(
      "Final Portfolio Value: {}",
      broker.cash() + data.calc_position_value(broker.position_size())
    );
  }
  fn feed(&mut self, data: &CsvDataSource) {
    self.sma.feed(&data.close);
  }

  fn on_order(&self, order: &Order, _: &CsvBroker) {
    if let OrderStatus::Completed(ref completed_at) = order.status {
      println!(
        "{}, {} EXECUTED, {}",
        completed_at.date_naive(),
        if order.is_buy() { "BUY" } else { "SELL" },
        order.exe_price
      );
    }
  }

  fn on_trade(&self, trade: &Trade, _: &CsvBroker) {
    if let TradeStatus::Closed(closed_at) = trade.status {
      *self.pnlcomm.borrow_mut() += trade.pnlcomm;
      println!(
        "{}, OPERATION PROFIT, GROSS {}, NET {}",
        closed_at.date_naive(),
        trade.pnl,
        trade.pnlcomm
      );
    }
  }

  fn next(&mut self, index: usize, data: &CsvDataSource, broker: &mut CsvBroker) {
    if let (Some(a), Some(b)) = (data.close.at(index), self.sma.at(index)) {
      let dt = data.timestamp[index].date_naive();
      // println!("{} {} {}", dt, a, b);
      if broker.is_position_empty() {
        if a > b {
          println!("{}, BUY CREATE, {}", dt, a);
          broker.buy(self.size, data, self);
        }
      } else if a < b {
        println!("{}, SELL CREATE, {}", dt, a);
        broker.sell(self.size, data, self);
      }
    }
  }
}

fn main() {
  let st = Instant::now();
  let data = CsvDataSource::builder()
    .time_field("date")
    .time_type(CsvTimeType::Date("%Y-%m-%d"))
    .load_from_file(
      &std::env::current_dir()
        .unwrap()
        .join("examples")
        .join("orcl-1995-2014.csv"),
    )
    .unwrap();

  let strat = DemoStrategy::new();
  let broker = CsvBroker::new(100_000.0);

  let mut engine = Engine::new(data, strat, broker);
  engine.run();
  let st = Instant::now().duration_since(st);
  println!(
    "Total cost time: {}ms({}us)",
    st.as_millis(),
    st.as_micros()
  );
}
