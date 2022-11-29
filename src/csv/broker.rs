use chrono::{DateTime, Utc};

use crate::{
  broker::Broker, CsvDataSource, DataLine, Order, OrderStatus, Position, Strategy, Trade,
  TradeStatus,
};

pub struct CsvBroker {
  pub(crate) cash: f64,
  pub(crate) position: Position,
  pub(crate) trade: Trade,
}
impl Broker for CsvBroker {
  type DS = CsvDataSource;

  /// 获取当前剩余现金
  #[inline]
  fn cash(&self) -> f64 {
    self.cash
  }
  /// 获取当前仓位持仓量
  #[inline]
  fn position_size(&self) -> isize {
    self.position.size
  }
  /// 获取当前仓位买入价格
  #[inline]
  fn position(&self) -> &Position {
    &self.position
  }

  /// 是否是空仓位，等价于 position() == 0
  #[inline]
  fn is_position_empty(&self) -> bool {
    self.position_size() == 0
  }
  #[inline]
  fn buy<S: Strategy<BK = Self, DS = Self::DS>>(
    &mut self,
    size: isize,
    data: &CsvDataSource,
    strat: &S,
  ) {
    self.submit_order(Order::buy(size, data.timestamp[data.offset]), data, strat);
  }
  #[inline]
  fn sell<S: Strategy<BK = Self, DS = Self::DS>>(
    &mut self,
    size: isize,
    data: &CsvDataSource,
    strat: &S,
  ) {
    self.submit_order(Order::sell(size, data.timestamp[data.offset]), data, strat);
  }
}

impl CsvBroker {
  pub fn new(cash: f64) -> Self {
    Self {
      cash,
      position: Position::new(),
      // trade: RefCell::new(Trade::new()),
      trade: Trade::new(),
    }
  }
  #[inline]
  pub fn trade(&self) -> &Trade {
    &self.trade
  }
  fn submit_order<S: Strategy<BK = Self, DS = CsvDataSource>>(
    &mut self,
    mut order: Order,
    data: &CsvDataSource,
    strat: &S,
  ) -> Order {
    if order.size <= 0 {
      panic!("order size must be greater than zero");
    }
    // 对于 csv 数据，成交价我们以下一个 bar 的 open 价作为成交价。
    // 对于 http broker 等应该是以实际的交易成交价作为 exe_price。
    // 如果当前已经是最后一个 bar，暂时就还是以 close 价作为成交价。
    let exe_price = data
      .open
      .at(data.offset + 1)
      .unwrap_or_else(|| data.close.at(data.offset).unwrap());
    let exe_time = data
      .timestamp
      .at(data.offset + 1)
      .unwrap_or_else(|| data.timestamp.at(data.offset).unwrap());
    order.exe_price = exe_price;
    // 对于 csv 数据，成交额认为等于下单量。对于 http broker 等应该以实际成交额为准（事实上一个 order 可能会由多个 trade 成交）
    order.exe_size = order.size;

    self.complete_order(&mut order, strat, exe_time);
    order
  }
  #[inline]
  fn complete_order<S: Strategy<BK = Self, DS = CsvDataSource>>(
    &mut self,
    order: &mut Order,
    strat: &S,
    completed_at: DateTime<Utc>,
  ) {
    let exe_price = order.exe_price;
    let exe_size = order.exe_size;
    let deal_size = if order.is_buy() { exe_size } else { -exe_size };
    order.cost = (deal_size as f64) * exe_price;
    self.cash -= order.cost;
    order.comm = strat.calc_commission(exe_size, exe_price);
    self.cash -= order.comm;
    let mut position = &mut self.position;

    let pre_s = position.size;
    if pre_s == 0 {
      position.size = deal_size;
      position.price = exe_price;
    } else {
      position.size += deal_size;
    }
    let post_s = position.size;
    if post_s != 0 {
      if (pre_s > 0 && post_s > 0) || (pre_s < 0 && post_s < 0) {
        // calcuate average price
        position.price = ((pre_s as f64) * position.price + (deal_size as f64) * exe_price)
          / (pre_s + deal_size) as f64;
      } else {
        // close and open position
        position.price = exe_price;
        position.origin_price = exe_price;
      }
    }

    order.status = OrderStatus::Completed(completed_at);
    strat.on_order(order, self);

    let is_uninit = matches!(&self.trade.status, TradeStatus::Uninit);

    if is_uninit && pre_s != 0 {
      panic!("unexpected");
    }

    // println!("{} {} {} {}", order.comm, order.cost, pre_s, post_s);
    // https://zhuanlan.zhihu.com/p/299630905
    let cost = -order.cost;
    let commcost = cost - order.comm;
    if is_uninit {
      self.trade.pnl = cost;
      self.trade.pnlcomm = commcost;
      self.trade.status = TradeStatus::Open(completed_at);
      strat.on_trade(&self.trade, self);
    } else if post_s == 0 {
      self.trade.pnl += cost;
      self.trade.pnlcomm += commcost;
      self.trade.status = TradeStatus::Closed(completed_at);
      strat.on_trade(&self.trade, self);
      self.trade.status = TradeStatus::Uninit;
    } else if pre_s > 0 && post_s < 0 || pre_s < 0 && post_s > 0 {
      let pre_percent = (pre_s / (post_s - pre_s)).abs() as f64;
      self.trade.pnl += cost * pre_percent;
      self.trade.pnlcomm += commcost * pre_percent;
      self.trade.status = TradeStatus::Closed(completed_at);
      strat.on_trade(&self.trade, self);
      let post_percent = (post_s / (post_s - pre_s)).abs() as f64;
      self.trade.pnl = cost * post_percent;
      self.trade.pnlcomm = commcost * post_percent;
      self.trade.status = TradeStatus::Open(completed_at);
      strat.on_trade(&self.trade, self);
    } else {
      self.trade.pnl += cost;
      self.trade.pnlcomm += commcost;
    }
  }
}
