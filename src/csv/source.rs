use std::ops::Index;

use chrono::{DateTime, Utc};

use crate::data::DataSource;
use crate::{Broker, CsvDataSourceBuilder, DataLine, DataLineFeed, Strategy};
//
// macro_rules! gen_mem_data_source {
//   ($struct_name: ident, $($name: ident: $idx: literal), +) => {
//     pub struct $struct_name {
//       offset: usize,
//       timestamp: Vec<i64>,
//       $ (
//         $name: Vec<f64>,
//       )*
//     }
//     impl $struct_name {
//       pub(super) fn new(timestamp_vec: Vec<i64>, data_vecs: [Vec<f64>; 7]) -> Self {
//         let mut data_vecs = data_vecs.map(|v| Some(v));
//         Self {
//           offset: 0,
//           timestamp: timestamp_vec,
//           $(
//             $name: data_vecs[$idx].take().unwrap(),
//           )*
//         }
//       }
//     }
//   };
// }
//
// gen_mem_data_source!(
//   CsvMemoryDataSource,
//   open: 0, close: 1, high: 2, low: 3,
//   volume: 4, openintrest: 5, adjustclose: 6
// );

pub struct CsvTimeLine {
  data: Vec<DateTime<Utc>>,
}

impl CsvTimeLine {
  #[inline(always)]
  pub fn at(&self, index: usize) -> Option<DateTime<Utc>> {
    self.data.get(index).copied()
  }
}
impl Index<usize> for CsvTimeLine {
  type Output = DateTime<Utc>;
  #[inline(always)]
  fn index(&self, index: usize) -> &Self::Output {
    &self.data[index]
  }
}

pub struct CsvDataLine {
  data: Vec<f64>,
}

impl DataLine for CsvDataLine {
  #[inline(always)]
  fn at(&self, index: usize) -> Option<f64> {
    self.data.get(index).copied()
  }
}

impl DataLineFeed for CsvDataLine {
  #[inline(always)]
  fn inner(&self) -> (&[f64], usize) {
    (&self.data, 0)
  }
}

pub struct CsvDataSource {
  pub(crate) offset: usize,
  pub timestamp: CsvTimeLine,
  pub close: CsvDataLine,
  pub open: CsvDataLine,
}

impl CsvDataSource {
  pub fn builder() -> CsvDataSourceBuilder {
    CsvDataSourceBuilder::new()
  }
  pub(super) fn inner_new(timestamp_vec: Vec<DateTime<Utc>>, data_vecs: [Vec<f64>; 7]) -> Self {
    // let mut data_vecs = data_vecs.map(|v| Some(v));
    let mut data_vecs = data_vecs.into_iter();
    Self {
      offset: 0,
      timestamp: CsvTimeLine {
        data: timestamp_vec,
      },
      open: CsvDataLine {
        data: data_vecs.next().unwrap(),
      },
      close: CsvDataLine {
        data: data_vecs.next().unwrap(),
      },
    }
  }
}

impl DataSource for CsvDataSource {
  fn read<B: Broker<DS = Self>, S: Strategy<DS = Self, BK = B>>(
    &mut self,
    strat: &mut S,
    broker: &mut B,
  ) -> bool {
    strat.feed(self);
    let len = self.timestamp.data.len();
    while self.offset < len {
      strat.next(self.offset, self, broker);
      self.offset += 1;
    }
    false
  }
  fn calc_position_value(&self, position_size: isize) -> f64 {
    position_size as f64 * self.close.data[self.offset - 1]
  }
}
