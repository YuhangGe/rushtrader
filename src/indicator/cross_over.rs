extern crate ta_lib_wrapper;

use crate::{
  impl_indicator_trait, impl_indicator_without_period, indicator::util::get_vec_at, DataLine,
  DataLineFeed,
};

pub struct CrossOverIndicator {
  start_pos: usize,
  data: Vec<f64>,
}
impl_indicator_without_period!(CrossOverIndicator);
impl_indicator_trait!(CrossOverIndicator);

#[inline(always)]
fn calc_cross(a: f64, b: f64, lnzd: i8) -> (f64, i8) {
  let new_lnzd = if a > b { 1 } else { -1 };
  (
    if lnzd < 0 && new_lnzd > 0 {
      1.
    } else if lnzd > 0 && new_lnzd < 0 {
      -1.
    } else {
      0.
    },
    new_lnzd,
  )
}
impl CrossOverIndicator {
  pub fn feed<DA: DataLineFeed, DB: DataLineFeed>(&mut self, data_line_a: &DA, data_line_b: &DB) {
    let (buf_a, start_pos_a) = data_line_a.inner();
    let (buf_b, start_pos_b) = data_line_b.inner();
    // println!("{} {}", start_pos_a, start_pos_b);
    let buf_a_len = buf_a.len();
    assert_eq!(buf_a_len, buf_b.len());
    let data_len = self.data.len();
    if data_len == 0 {
      self.data = Vec::with_capacity(buf_a_len)
    } else {
      assert_eq!(buf_a_len, data_len);
    }
    let start_pos = start_pos_a.max(start_pos_b) + 1;
    self.start_pos = start_pos;
    unsafe {
      let mut pa = buf_a.as_ptr().add(start_pos - 1);
      let mut pb = buf_b.as_ptr().add(start_pos - 1);
      let mut lnzd = {
        let a = *pa;
        let b = *pb;
        if a == b {
          0
        } else if a > b {
          1
        } else {
          -1
        }
      };
      pa = pa.add(1);
      pb = pb.add(1);
      let mut pd = self.data.as_mut_ptr().add(start_pos);
      for _ in start_pos..buf_a_len {
        let a = *pa;
        let b = *pb;
        if a == b {
          *pd = 0.;
        } else {
          (*pd, lnzd) = calc_cross(a, b, lnzd);
        }
        // println!("{}, {}, {}, {:?}", *pa, *pb, *pd, lnzd);
        pa = pa.add(1);
        pb = pb.add(1);
        pd = pd.add(1);
      }
      self.data.set_len(buf_a_len);
    }
  }
}

#[test]
fn test_cross_over_indicator() {
  struct D(Vec<f64>, usize);
  impl DataLineFeed for D {
    fn inner(&self) -> (&[f64], usize) {
      (&self.0, self.1)
    }
  }
  let d1 = D(vec![0.1, 0.3, 0.2, 0.3, 0.5, 0.6], 0);
  let d2 = D(vec![0., 0.1, 0.4, 0.5, 0.5, 0.3], 1);
  let mut ind = CrossOverIndicator::new();
  ind.feed(&d1, &d2);
  assert_eq!(ind.data.len(), d1.0.len());
  assert_eq!(ind.start_pos, 1);
  assert_eq!(ind.data, vec![0., 0., 1., 0., 0., -1.]);

  // println!("{:?}", d1.0);
  // println!("{:?}", d2.0);
  // println!("{:?}", ind.data);
  // panic!("")
}
