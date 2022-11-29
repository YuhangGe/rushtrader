use rushtrader::{util::get_vec_at, DataLine, DataLineFeed, LinearregSlopeIndicator, MOMIndicator};

pub struct StopProfitTakingIndicator {
  start_pos: usize,
  data: Vec<f64>,
}

impl DataLineFeed for StopProfitTakingIndicator {
  #[inline(always)]
  fn inner(&self) -> (&[f64], usize) {
    (&self.data, self.start_pos)
  }
}
impl DataLine for StopProfitTakingIndicator {
  #[inline(always)]
  fn at(&self, index: usize) -> Option<f64> {
    get_vec_at(&self.data, self.start_pos, index)
  }
}
impl StopProfitTakingIndicator {
  pub fn new() -> Self {
    Self {
      data: Vec::new(),
      start_pos: 0,
    }
  }
  pub fn feed(&mut self, second_order_slope: &MOMIndicator, slope4h: &LinearregSlopeIndicator) {
    let (buf_a, start_pos_a) = second_order_slope.inner();
    let (buf_b, start_pos_b) = slope4h.inner();
    let buf_a_len = buf_a.len();
    assert_eq!(buf_a_len, buf_b.len());
    let data_len = self.data.len();
    if data_len == 0 {
      self.data = Vec::with_capacity(buf_a_len)
    } else {
      assert_eq!(buf_a_len, data_len);
    }
    let start_pos = start_pos_a.max(start_pos_b);
    self.start_pos = start_pos;
    unsafe {
      let mut pa = buf_a.as_ptr().add(start_pos);
      let mut pb = buf_b.as_ptr().add(start_pos);
      let mut pd = self.data.as_mut_ptr().add(start_pos);
      for _ in start_pos..buf_a_len {
        let second_order_slope = *pa;
        let slope4h = *pb;
        *pd = if second_order_slope < 0. && slope4h < 0. {
          1.
        } else if second_order_slope > 0. && slope4h > 0. {
          -1.
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
