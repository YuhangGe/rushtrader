extern crate ta_lib_wrapper;

use ta_lib_wrapper::{TA_Integer, TA_RetCode, TA_MOM};

use crate::{
  impl_indicator_trait, impl_indicator_with_period, indicator::util::get_vec_at, DataLine,
  DataLineFeed,
};

pub struct MOMIndicator {
  period: usize,
  start_pos: usize,
  data: Vec<f64>,
}
impl_indicator_with_period!(MOMIndicator);
impl_indicator_trait!(MOMIndicator);

impl MOMIndicator {
  pub fn feed<D: DataLineFeed>(&mut self, data_line: &D) {
    let (src_data, src_start_pos) = data_line.inner();
    self.start_pos = src_start_pos + self.period;
    // println!("xx: {} {} {}", src_start_pos, self.period, self.start_pos);

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
      let ret_code = TA_MOM(
        0,
        (src_len - src_start_pos - 1) as i32,
        src_data.as_ptr().add(src_start_pos),
        self.period as i32,
        &mut out_begin,
        &mut out_size,
        self.data.as_mut_ptr().add(self.start_pos),
      );
      // println!("yy: {} {} {}", src_len - src_start_pos - 1, out_begin, out_size);
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
      //   "sma: {} {} {} {}",
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

#[test]
fn test_mom_indicator() {
  struct D(Vec<f64>, usize);
  impl DataLineFeed for D {
    fn inner(&self) -> (&[f64], usize) {
      (&self.0, self.1)
    }
  }
  let d1 = D(vec![1., 1., 2., 3., 5., 8., 13.], 0);
  let mut ind = MOMIndicator::new(2);
  ind.feed(&d1);
  assert_eq!(ind.data, vec![0., 0., 1., 2., 3., 5., 8.]);
}
