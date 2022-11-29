mod constant;
mod lines;
mod overnight;
mod signal_indicator;
mod sizer;
mod stop_profit_taking_indicator;
mod strat;
mod trade;
mod util;

use std::{fs::read_to_string, time::Instant};

use rushtrader::{CsvBroker, CsvDataSource, CsvTimeType, Engine};

use crate::{lines::VLines, strat::VegasStrategy};

fn main() {
  let st = Instant::now();
  let data = CsvDataSource::builder()
    .time_field("date")
    .time_type(CsvTimeType::Datetime("%m/%d/%Y %H:%M"))
    .load_from_lines(VLines::new(
      read_to_string(
        &std::env::current_dir()
          .unwrap()
          .join("examples/audusd/audusd.csv"),
      )
      .unwrap()
      .lines(),
    ))
    .unwrap_or_else(|e| {
      eprintln!("{}", e);
      panic!("error");
    });

  let strat = VegasStrategy::new();
  let broker = CsvBroker::new(1_000_000_000.0);

  let mut engine = Engine::new(data, strat, broker);
  engine.run();
  let st = Instant::now().duration_since(st);
  println!(
    "Total cost time: {}ms({}us)",
    st.as_millis(),
    st.as_micros()
  );
}
