#[inline(always)]
pub fn get_vec_at(data: &[f64], start_pos: usize, index: usize) -> Option<f64> {
  if index < start_pos {
    None
  } else {
    data.get(index).copied()
  }
}
