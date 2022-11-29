use crate::{impl_indicator_trait, impl_indicator_without_period, DataLine, DataLineFeed};

use super::util::get_vec_at;

pub struct MaxIndicator {
  start_pos: usize,
  data: Vec<f64>,
}
impl_indicator_without_period!(MaxIndicator);
impl_indicator_trait!(MaxIndicator);

impl MaxIndicator {
  pub fn feed<DA: DataLineFeed, DB: DataLineFeed>(&mut self, data_line_a: &DA, data_line_b: &DB) {
    let (buf_a, start_pos_a) = data_line_a.inner();
    let (buf_b, start_pos_b) = data_line_b.inner();

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
        *pd = (*pa).max(*pb);
        pa = pa.add(1);
        pb = pb.add(1);
        pd = pd.add(1);
      }
      self.data.set_len(buf_a_len);
    }
  }
}

#[test]
fn test_max_indicator() {
  struct D(Vec<f64>, usize);
  impl DataLineFeed for D {
    fn inner(&self) -> (&[f64], usize) {
      (&self.0, self.1)
    }
  }
  let d1 = D(vec![1.; 4], 1);
  let d2 = D(vec![2.; 4], 0);
  let mut ind = MaxIndicator::new();
  ind.feed(&d1, &d2);
  assert_eq!(ind.data.len(), 4);
  assert_eq!(ind.inner().1, 1);
  assert_eq!(ind.at(0), None);
  assert_eq!(ind.at(1), Some(2.));

  let d3 = D(vec![3.; 4], 0);
  ind.feed(&d1, &d3);
  assert_eq!(ind.data.len(), 4);
  assert_eq!(ind.data, vec![0., 3., 3., 3.]);
}
