use chrono::{DateTime, Timelike, Utc};
use rushtrader::{
  Broker, CrossOverIndicator, CsvBroker, CsvDataSource, DataLine, DataSource, EMAIndicator,
  LinearregSlopeIndicator, MOMIndicator, MaxIndicator, MinIndicator, Order, OrderStatus, Strategy,
};

use crate::{
  bool_map,
  constant::{
    FAST_TUNNEL_PERIOD, FILTER_PERIOD, PROFIT_TAKING, SECOND_ORDER_SLOPE_PERIOD, SLOPE_PERIOD,
    SLOW_TUNNEL_PERIOD, STOP_LOSS,
  },
  overnight::{Overnight, OvernightSignal, OvernightType},
  signal_indicator::SignalIndicator,
  stop_profit_taking_indicator::StopProfitTakingIndicator,
  trade::{complete_trade, VegasTradeInfo, VegasTradeOrderType},
  util::get_market_close_utc_hours,
};

pub(super) struct VegasStrategy {
  // indicators begin
  pub fast_tunnel: EMAIndicator,
  pub slow_tunnel: EMAIndicator,
  pub filter_tunnel: EMAIndicator,
  pub max_tunnel: MaxIndicator,
  pub min_tunnel: MinIndicator,
  pub slope: LinearregSlopeIndicator,
  pub tunnel4h: EMAIndicator,
  pub slope4h: LinearregSlopeIndicator,
  pub second_order_slope: MOMIndicator,
  pub cross_max: CrossOverIndicator,
  pub cross_min: CrossOverIndicator,
  pub signal: SignalIndicator,
  pub stop_profit_taking: StopProfitTakingIndicator,
  // indicator end
  pub overnight: Overnight,
  // pub trade: VegasTradeInfo,
  pub p_trade: *mut VegasTradeInfo,
}
impl Drop for VegasStrategy {
  fn drop(&mut self) {
    unsafe {
      let _ = Box::from_raw(self.p_trade); // drop
      self.p_trade = std::ptr::null_mut();
    }
  }
}
impl VegasStrategy {
  pub(super) fn new() -> Self {
    VegasStrategy {
      fast_tunnel: EMAIndicator::new(FAST_TUNNEL_PERIOD),
      slow_tunnel: EMAIndicator::new(SLOW_TUNNEL_PERIOD),
      filter_tunnel: EMAIndicator::new(FILTER_PERIOD),
      max_tunnel: MaxIndicator::new(),
      min_tunnel: MinIndicator::new(),
      slope: LinearregSlopeIndicator::new(SLOPE_PERIOD),
      tunnel4h: EMAIndicator::new(FAST_TUNNEL_PERIOD * 4),
      slope4h: LinearregSlopeIndicator::new(SLOPE_PERIOD * 4),
      second_order_slope: MOMIndicator::new(SECOND_ORDER_SLOPE_PERIOD),
      cross_max: CrossOverIndicator::new(),
      cross_min: CrossOverIndicator::new(),
      signal: SignalIndicator::new(),
      stop_profit_taking: StopProfitTakingIndicator::new(),

      overnight: Overnight::new(),
      p_trade: Box::into_raw(Box::new(VegasTradeInfo::new())),
    }
  }

  #[inline]
  fn is_pre_midnight(&self, dt: &DateTime<Utc>) -> bool {
    let uh = dt.hour();
    let ch = get_market_close_utc_hours(dt);
    uh >= ch - 1 && uh < ch
  }

  #[inline]
  fn is_midnight(&self, dt: &DateTime<Utc>) -> bool {
    let uh = dt.hour();
    let ch = get_market_close_utc_hours(dt);
    uh >= ch && uh < ch + 1
  }
}

impl Strategy for VegasStrategy {
  type DS = CsvDataSource;
  type BK = CsvBroker;

  fn calc_commission(&self, size: isize, price: f64) -> f64 {
    ((size.abs() as f64) * price * 0.00002).max(2.)
  }
  // fn on_start(&self, _data: &CsvDataSource, broker: &CsvBroker) {
  //   println!("Starting Portfolio Value: {}", broker.cash());
  // }
  fn on_finish(&self, data: &CsvDataSource, broker: &CsvBroker) {
    // println!("Final cash: {}", broker.cash());
    println!(
      "[INFO] Final position: Position {{ size: {}, price: {}, originPrice: {} }}",
      broker.position().size,
      broker.position().price,
      broker.position().origin_price,
    );
    // println!("Final pnlmm: {}", self.pnlcomm);
    println!(
      "[INFO] Final value: {}",
      broker.cash() + data.calc_position_value(broker.position_size())
    );
  }
  fn feed(&mut self, data: &CsvDataSource) {
    self.fast_tunnel.feed(&data.close);
    self.slow_tunnel.feed(&data.close);
    self.filter_tunnel.feed(&data.close);
    self.max_tunnel.feed(&self.fast_tunnel, &self.slow_tunnel);
    self.min_tunnel.feed(&self.fast_tunnel, &self.slow_tunnel);
    self.slope.feed(&self.slow_tunnel);
    self.tunnel4h.feed(&data.close);
    self.slope4h.feed(&self.tunnel4h);
    self.second_order_slope.feed(&self.slope);
    self.cross_max.feed(&self.filter_tunnel, &self.max_tunnel);
    self.cross_min.feed(&self.filter_tunnel, &self.min_tunnel);
    self
      .signal
      .feed(&self.cross_max, &self.cross_min, &self.slope);
    self
      .stop_profit_taking
      .feed(&self.second_order_slope, &self.slope4h);

    // println!("{}", self.stop_profit_taking.inner().1);
    // let (a, b) = self.signal.inner();
    // println!("{}", b);
    // a.iter().enumerate().skip(b).for_each(|(i, v)| {
    //   println!("{}: {}", i, v)
    // });
    // panic!("exit")
  }

  fn on_order(&self, order: &Order, broker: &CsvBroker) {
    if let OrderStatus::Completed(completed_at) = order.status {
      println!(
        "[INFO] [{}] {} EXECUTED, Price: {:.6}, Current position: {}",
        completed_at,
        bool_map!(order.is_buy(), "BUY", "SELL"),
        order.exe_price,
        broker.position_size()
      );
      let trade = unsafe { &mut *self.p_trade };
      match &trade.order_type {
        VegasTradeOrderType::Open => {
          trade.open_time = Some(order.created_at);
          trade.prices = vec![order.exe_price];
          trade.open_position = broker.position_size();
          if trade.open_price.is_none() {
            trade.open_price.replace(order.exe_price);
          }
        }
        VegasTradeOrderType::Close => {
          trade.prices.push(order.exe_price);
          complete_trade(trade, &completed_at);
        }
        VegasTradeOrderType::MidnightClose | VegasTradeOrderType::MidnightOpen => {
          trade.prices.push(order.exe_price);
        }
        _ => panic!("impossible"),
      }

      trade.order_type = VegasTradeOrderType::None;
      // this.onTradeUpdated(order.product.symbol, order.exePrice);
    }
  }

  fn next(&mut self, index: usize, data: &CsvDataSource, broker: &mut CsvBroker) {
    macro_rules! get_data {
      ($line: expr) => {
        match $line.at(index) {
          Some(v) => v,
          None => return,
        }
      };
    }
    let dt = get_data!(data.timestamp);
    let v_close_price = get_data!(data.close);
    let v_signal = get_data!(self.signal);
    let v_stop_profit_taking = get_data!(self.stop_profit_taking);

    let has_overnight = !matches!(self.overnight.overtype, OvernightType::None);
    let has_overnight_signal = !matches!(self.overnight.signal, OvernightSignal::None);
    let position_size = broker.position_size();
    let trade_info = unsafe { &mut *self.p_trade };

    // if index == 804 {
    //   println!("{} {} {}", self.is_pre_midnight(&dt), dt, dt.with_timezone(&chrono_tz::America::New_York));
    // }
    if position_size == 0 {
      if self.is_pre_midnight(&dt) {
        if has_overnight {
          eprintln!("[ERROR] [{}] UNEXPECTED overnight type", dt);
          self.overnight.overtype = OvernightType::None;
        }
        if v_signal != 0. {
          let sig_ty = bool_map!(
            v_signal > 0.,
            OvernightSignal::Long(v_close_price),
            OvernightSignal::Short(v_close_price)
          );
          println!("[INFO] [{}] Overnight signal: {:?}", dt, sig_ty);
          self.overnight.signal = sig_ty;
          // self.on_trade_updated
        }
        return;
      }
      if self.is_midnight(&dt) {
        if v_signal != 0. {
          let sig_ty = bool_map!(
            v_signal > 0.,
            OvernightSignal::Long(v_close_price),
            OvernightSignal::Short(v_close_price)
          );
          if has_overnight {
            println!(
              "[INFO] [{}] Skip overnight position as {:?} signal",
              dt, sig_ty
            );
            self.complete_trade(&dt);
            self.overnight.overtype = OvernightType::None;
          }
          println!("[INFO] [{}] Overnight signal: {:?}", dt, sig_ty);
          self.overnight.signal = sig_ty;
          // self.on_update_trade
          return;
        }
        if has_overnight {
          if has_overnight_signal {
            // 21:00 理论上不可能出现信号。
            eprintln!("[ERROR] [{}] UNEXPECTED 21:00 signal", dt);
            self.overnight.signal = OvernightSignal::None;
          }
          let is_positive = matches!(self.overnight.overtype, OvernightType::Long);

          let trade_r = bool_map!(is_positive, 1., -1.)
            * (v_close_price / trade_info.open_price.unwrap() - 1.0);
          if (trade_r >= PROFIT_TAKING
            && bool_map!(
              is_positive,
              v_stop_profit_taking > 0.,
              v_stop_profit_taking < 0.
            ))
            || (trade_r <= STOP_LOSS)
          {
            println!(
              "[INFO] [{}] Skip overnight position as {}%, {}, {}",
              dt,
              trade_r * 100.,
              v_close_price,
              v_stop_profit_taking
            );
            self.complete_trade(&dt);
            self.overnight.overtype = OvernightType::None;
          }
        }
        return;
      }
      if v_signal != 0. {
        let sig = bool_map!(v_signal > 0., "Long", "Short");
        if has_overnight {
          println!("[INFO] [{}] Skip overnight position as {} signal", dt, sig);
          self.complete_trade(&dt);
        }
        if has_overnight_signal {
          println!("[INFO] [{}] Skip overnight signal as {} signal", dt, sig);
        }
        println!("[INFO] [{}] {} signal.", dt, sig);
        trade_info.order_type = VegasTradeOrderType::Open;
        trade_info.open_price = None;
        self.overnight.overtype = OvernightType::None;
        self.overnight.signal = OvernightSignal::None;
        if v_signal > 0. {
          broker.buy(self.get_sizer(broker), data, self);
        } else {
          broker.sell(self.get_sizer(broker), data, self);
        }
      } else if has_overnight_signal {
        if has_overnight {
          eprintln!("[ERROR] [{}] UNEXPECTED overnight 2", dt);
          self.overnight.overtype = OvernightType::None;
        }
        let is_positive = matches!(self.overnight.overtype, OvernightType::Long);
        let trade_r =
          bool_map!(is_positive, 1., -1.) * (v_close_price / trade_info.open_price.unwrap() - 1.0);
        if (trade_r >= PROFIT_TAKING
          && bool_map!(
            is_positive,
            v_stop_profit_taking > 0.,
            v_stop_profit_taking < 0.
          ))
          || (trade_r <= STOP_LOSS)
        {
          println!(
            "[INFO] [{}] Skip overnight position as {}%, {}, {}",
            dt,
            trade_r * 100.,
            v_close_price,
            v_stop_profit_taking
          );
          self.overnight.overtype = OvernightType::None;
          // this.onTradeUpdated(this.data.symbol, 0);
        } else {
          let sig_price = match self.overnight.signal {
            OvernightSignal::Long(price) => price,
            OvernightSignal::Short(price) => price,
            _ => panic!("impossible"),
          };
          println!(
            "[INFO] [{}] Recover overnight signal {}",
            dt, self.overnight.signal
          );
          trade_info.order_type = VegasTradeOrderType::Open;
          trade_info.open_price = Some(sig_price);
          self.overnight.signal = OvernightSignal::None;
          if is_positive {
            broker.buy(self.get_sizer(broker), data, self);
          } else {
            broker.sell(self.get_sizer(broker), data, self);
          }
        }
      } else if has_overnight {
        let is_positive = matches!(self.overnight.overtype, OvernightType::Long);
        let trade_r =
          bool_map!(is_positive, 1., -1.) * (v_close_price / trade_info.open_price.unwrap() - 1.0);
        if (trade_r >= PROFIT_TAKING
          && bool_map!(
            is_positive,
            v_stop_profit_taking > 0.,
            v_stop_profit_taking < 0.
          ))
          || (trade_r <= STOP_LOSS)
        {
          println!(
            "[INFO] [{}] Skip overnight position as {}%, {}, {}",
            dt,
            trade_r * 100.,
            v_close_price,
            v_stop_profit_taking
          );
          self.complete_trade(&dt);
          self.overnight.overtype = OvernightType::None;
          // self.onTradeUpdated(this.data.symbol, 0);
        } else {
          println!(
            "[INFO] [{}] Recover overnight {} position.",
            dt, self.overnight.overtype
          );
          trade_info.order_type = VegasTradeOrderType::MidnightOpen;
          self.overnight.overtype = OvernightType::None;
          if is_positive {
            broker.buy(self.get_sizer(broker), data, self);
          } else {
            broker.sell(self.get_sizer(broker), data, self);
          }
        }
      }
    } else {
      if has_overnight || has_overnight_signal {
        eprintln!(
          "[ERROR] [{}] UNEXPECTED position with overnight type or signal",
          dt
        );
        self.overnight.overtype = OvernightType::None;
        self.overnight.signal = OvernightSignal::None;
      }
      let is_positive = broker.position_size() > 0;
      let sig = bool_map!(is_positive, OvernightType::Long, OvernightType::Short);
      let trade_r =
        bool_map!(is_positive, 1., -1.) * (v_close_price / trade_info.open_price.unwrap() - 1.0);

      #[inline(always)]
      fn deal(
        is_positive: bool,
        order_type: VegasTradeOrderType,
        trade_info: &mut VegasTradeInfo,
        strat: &mut VegasStrategy,
        data: &CsvDataSource,
        broker: &mut CsvBroker,
      ) {
        trade_info.order_type = order_type;
        if is_positive {
          broker.sell(strat.get_sizer(broker), data, strat);
        } else {
          broker.buy(strat.get_sizer(broker), data, strat);
        }
      }

      if trade_r >= PROFIT_TAKING {
        if bool_map!(
          is_positive,
          v_stop_profit_taking > 0.,
          v_stop_profit_taking < 0.
        ) {
          println!(
            "[INFO] [{}] Profit-taking triggered. Close {:?} position.",
            dt, sig
          );
          deal(
            is_positive,
            VegasTradeOrderType::Close,
            trade_info,
            self,
            data,
            broker,
          );
        } else if self.is_pre_midnight(&dt) {
          println!(
            "[INFO] [{}] Midnight upcoming. Temporarily close {} position.",
            dt,
            bool_map!(is_positive, "long", "short")
          );
          deal(
            is_positive,
            VegasTradeOrderType::MidnightClose,
            trade_info,
            self,
            data,
            broker,
          );
          self.overnight.overtype = sig;
        }
      } else if trade_r <= STOP_LOSS {
        println!(
          "[INFO] [{}] Stop-loss triggered. Close {} position.",
          dt,
          bool_map!(is_positive, "long", "short")
        );
        // println!("KKKK {}", index);
        deal(
          is_positive,
          VegasTradeOrderType::Close,
          trade_info,
          self,
          data,
          broker,
        );
      } else if self.is_pre_midnight(&dt) {
        println!(
          "[INFO] {} Midnight upcoming. Temporarily close {} position.",
          dt,
          bool_map!(is_positive, "long", "short")
        );
        deal(
          is_positive,
          VegasTradeOrderType::MidnightClose,
          trade_info,
          self,
          data,
          broker,
        );
        self.overnight.overtype = sig;
      }
    }
  }
}
