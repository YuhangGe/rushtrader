extern crate ta_lib_wrapper;

use ta_lib_wrapper::{TA_Integer, TA_MAType, TA_RetCode, TA_MA};

use crate::{
  impl_indicator_trait, impl_indicator_with_period, indicator::util::get_vec_at, DataLine,
  DataLineFeed,
};

pub struct EMAIndicator {
  period: usize,
  start_pos: usize,
  data: Vec<f64>,
}
impl_indicator_with_period!(EMAIndicator);
impl_indicator_trait!(EMAIndicator);

impl EMAIndicator {
  pub fn feed<D: DataLineFeed>(&mut self, data_line: &D) {
    let (src_data, src_start_pos) = data_line.inner();
    self.start_pos = src_start_pos + self.period - 1;

    let src_len = src_data.len();
    let data_len = self.data.len();
    if data_len == 0 {
      self.data = Vec::with_capacity(src_len)
    } else {
      assert_eq!(src_len, data_len);
    }

    let mut out_begin: TA_Integer = 0;
    let mut out_size: TA_Integer = 0;

    unsafe {
      let ret_code = TA_MA(
        0,
        (src_len - src_start_pos - 1) as i32,
        src_data.as_ptr().add(src_start_pos),
        self.period as i32,
        TA_MAType::TA_MAType_EMA,
        &mut out_begin,
        &mut out_size,
        self.data.as_mut_ptr().add(self.start_pos),
      );
      match ret_code {
        // Indicator was computed correctly, since the vector was filled by TA-lib C library,
        // Rust doesn't know what is the new length of the vector, so we set it manually
        // to the number of values returned by the TA_MA call
        TA_RetCode::TA_SUCCESS => {
          assert_eq!(src_start_pos + out_begin as usize, self.start_pos);
          assert_eq!(src_start_pos + (out_begin + out_size) as usize, src_len);
          // 注意 prepare_opacity 函数只分配空间并不进行数据初始化（没有初始化的空间也可以被 c 函数使用），因此需要强行 set_len 指定长度。
          self.data.set_len(src_len);
        }
        // An error occured
        _ => panic!("Could not compute indicator, err: {:?}", ret_code),
      }
      // println!(
      //   "ema: {} {} {} {}",
      //   self.period,
      //   out_begin,
      //   out_size,
      //   self.data.len()
      // );
      // println!("{:?}", self.data);
    }
  }
}
// impl Indicator for SMAIndicator {}
