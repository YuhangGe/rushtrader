use rushtrader::{
  util::get_vec_at, CrossOverIndicator, DataLine, DataLineFeed, LinearregSlopeIndicator,
};

use crate::constant::SLOPE_THRESHOLD;

pub struct SignalIndicator {
  pub(crate) start_pos: usize,
  pub(crate) data: Vec<f64>,
}

impl DataLineFeed for SignalIndicator {
  #[inline(always)]
  fn inner(&self) -> (&[f64], usize) {
    (&self.data, self.start_pos)
  }
}
impl DataLine for SignalIndicator {
  #[inline(always)]
  fn at(&self, index: usize) -> Option<f64> {
    get_vec_at(&self.data, self.start_pos, index)
  }
}
impl SignalIndicator {
  pub fn new() -> Self {
    Self {
      data: Vec::new(),
      start_pos: 0,
    }
  }
  pub fn feed(
    &mut self,
    cross_max: &CrossOverIndicator,
    cross_min: &CrossOverIndicator,
    slope: &LinearregSlopeIndicator,
  ) {
    let (buf_a, start_pos_a) = cross_max.inner();
    let (buf_b, start_pos_b) = cross_min.inner();
    let (buf_c, start_pos_c) = slope.inner();
    let buf_a_len = buf_a.len();
    assert_eq!(buf_a_len, buf_b.len());
    assert_eq!(buf_a_len, buf_c.len());
    let data_len = self.data.len();
    if data_len == 0 {
      self.data = Vec::with_capacity(buf_a_len)
    } else {
      assert_eq!(buf_a_len, data_len);
    }
    let start_pos = start_pos_a.max(start_pos_b).max(start_pos_c);
    self.start_pos = start_pos;
    unsafe {
      let mut pa = buf_a.as_ptr().add(start_pos);
      let mut pb = buf_b.as_ptr().add(start_pos);
      let pc = buf_c.as_ptr().add(start_pos);
      let mut pd = self.data.as_mut_ptr().add(start_pos);
      for _ in start_pos..buf_a_len {
        let cross_max = *pa;
        let cross_min = *pb;
        let slope = *pc;
        *pd = if slope.abs() > SLOPE_THRESHOLD {
          if cross_max > 0. {
            1.
          } else if cross_min < 0. {
            -1.
          } else {
            0.
          }
        } else {
          0.
        };
        pa = pa.add(1);
        pb = pb.add(1);
        pd = pd.add(1);
      }
      self.data.set_len(buf_a_len);
    }
  }
}
