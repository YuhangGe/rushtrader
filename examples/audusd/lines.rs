use std::str::Lines;

pub(super) struct VLines<'a>(Lines<'a>, bool);

impl<'a> Iterator for VLines<'a> {
  type Item = &'a str;
  fn next(&mut self) -> Option<Self::Item> {
    if self.1 {
      for line in self.0.by_ref() {
        let line = line.trim();
        if !line.is_empty() {
          self.1 = false;
          return Some(line);
        }
      }
    } else {
      while let Some(line) = self.0.next_back() {
        let line = line.trim();
        if !line.is_empty() {
          return Some(line);
        }
      }
    }
    None
  }
}

impl<'a> VLines<'a> {
  pub(super) fn new(lines: Lines<'a>) -> Self {
    Self(lines, true)
  }
}
