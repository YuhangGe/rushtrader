use crate::{Broker, Strategy};

pub trait DataSource: Sized {
  /// 读取数据并执行策略，返回是否需要继续读取和处理数据。
  fn read<B: Broker<DS = Self>, S: Strategy<DS = Self, BK = B>>(
    &mut self,
    strat: &mut S,
    broker: &mut B,
  ) -> bool;
  /// 计算仓位的当前现金价值
  fn calc_position_value(&self, position_size: isize) -> f64;
}

pub trait DataLineFeed {
  /// 获取当前 DataLine 的内部数据。元组第一个元素是数据 Slice，第二个元素是数据的有效初始位置。
  /// 我们约定数据 Slice 的长度一定对齐 DataSource 的初始数据长度，即所有 Indicator 的数据存储 Slice 长度一致。
  /// 但不同的 Indicator 的有效数据的开始位置不一定是 0（比如移动平均类型的指标会有 Period），因此通过元组返回。
  fn inner(&self) -> (&[f64], usize);
}
pub trait DataLine {
  fn at(&self, index: usize) -> Option<f64>;
}
