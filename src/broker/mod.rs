mod order;
mod position;
mod trade;

pub use order::*;
pub use position::*;
pub use trade::*;

use crate::{DataSource, Strategy};

pub trait Broker {
  type DS: DataSource;
  /// 获取当前剩余现金
  fn cash(&self) -> f64;
  /// 获取当前仓位持仓量
  fn position_size(&self) -> isize;
  /// 获取当前仓位建仓价
  fn position(&self) -> &Position;
  /// 当前是否是空仓
  fn is_position_empty(&self) -> bool;
  /// 建买仓（多仓）
  fn buy<S: Strategy<BK = Self, DS = Self::DS>>(&mut self, size: isize, data: &Self::DS, strat: &S);
  /// 建卖仓（空仓）
  fn sell<S: Strategy<BK = Self, DS = Self::DS>>(
    &mut self,
    size: isize,
    data: &Self::DS,
    strat: &S,
  );
}
