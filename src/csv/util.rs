use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use std::{
  fs::read_to_string,
  io::{self, Error, ErrorKind},
  path::Path,
};

use super::builder::{CsvDataSourceBuilder, CsvTimeType};

#[inline(always)]
pub(super) fn parse_f64(v: &str) -> io::Result<f64> {
  v.parse::<f64>().map_err(|e| new_io_err(e.to_string()))
}
#[inline(always)]
fn new_io_err(e: String) -> Error {
  Error::new(ErrorKind::Other, e)
}
#[inline(always)]
pub(super) fn new_io_err_str(e: &'static str) -> Error {
  Error::new(ErrorKind::Other, e)
}
pub(super) fn parse_time_field(ty: &CsvTimeType, v: &str) -> io::Result<DateTime<Utc>> {
  match ty {
    CsvTimeType::Millsecond => v
      .parse::<i64>()
      .map(|v| {
        let secs = v / 1000;
        let nsecs = (v % 1000) * 1_000_000;
        DateTime::from_utc(
          NaiveDateTime::from_timestamp_opt(secs, nsecs as u32).unwrap(),
          Utc,
        )
      })
      .map_err(|e| new_io_err(e.to_string())),
    CsvTimeType::Second => v
      .parse::<i64>()
      .map(|v| DateTime::from_utc(NaiveDateTime::from_timestamp_opt(v, 0).unwrap(), Utc))
      .map_err(|e| new_io_err(e.to_string())),
    CsvTimeType::Date(fmt) => NaiveDate::parse_from_str(v, fmt)
      .map_err(|e| new_io_err(e.to_string()))
      .map(|v| DateTime::from_utc(v.and_hms_opt(0, 0, 0).unwrap(), Utc)),
    CsvTimeType::Datetime(fmt) => NaiveDateTime::parse_from_str(v, fmt)
      .map(|v| DateTime::from_utc(v, Utc))
      .map_err(|e| new_io_err(e.to_string())),
    _ => panic!("impossible"),
  }
}

// pub(super) fn load_csv_with_stream(
//   builder: &CsvDataSourceBuilder,
// ) -> io::Result<(Lines<BufReader<File>>, [i8; 8])> {
//   let mut lines = BufReader::new(File::open(&builder.file)?).lines();
//   let header_line = loop {
//     if let Some(line) = lines.next() {
//       let line = line?;
//       if !line.trim().is_empty() {
//         break line;
//       }
//     } else {
//       return Err(new_io_err_str("csv file missing header line"));
//     }
//   };
//   let idx_arr = get_column_indexies(builder, &header_line)?;
//   Ok((lines, idx_arr))
//   // .map(|l| l.trim()).filter(|l| !l.is_empty());
// }

fn get_column_indexies(builder: &CsvDataSourceBuilder, header_line: &str) -> io::Result<[i8; 8]> {
  // field indeies of: timestamp, open, close, high, low, volumn, openintrest, adjustclose
  let mut idx_arr = [-1i8, -1, -1, -1, -1, -1, -1, -1];
  header_line
    .split(',')
    .map(|s| s.trim().to_lowercase())
    .enumerate()
    .for_each(|(idx, s)| {
      // FIXME: 使用 macro 或者表驱动简化代码？？
      if builder.time_field.eq(&s) {
        idx_arr[0] = idx as i8;
      } else if builder.open_field.eq(&s) {
        idx_arr[1] = idx as i8;
      } else if builder.close_field.eq(&s) {
        idx_arr[2] = idx as i8;
      } else if builder.high_field.eq(&s) {
        idx_arr[3] = idx as i8;
      } else if builder.low_field.eq(&s) {
        idx_arr[4] = idx as i8;
      } else if builder.volume_field.eq(&s) {
        idx_arr[5] = idx as i8;
      } else if builder.openintrest_field.eq(&s) {
        idx_arr[6] = idx as i8;
      } else if builder.adjustclose_field.eq(&s) {
        idx_arr[7] = idx as i8;
      }
    });
  if idx_arr[0] < 0 {
    Err(new_io_err_str("csv header miss timestamp field"))
  } else {
    Ok(idx_arr)
  }
}

type LoadResult = (Vec<DateTime<Utc>>, [Vec<f64>; 7]);
pub(super) fn load_csv_from_lines<'a, T: Iterator<Item = &'a str>>(
  mut lines: T,
  builder: &CsvDataSourceBuilder,
) -> io::Result<LoadResult> {
  let header_line = lines.next().unwrap();
  // field indeies of: timestamp, open, close, high, low, volumn, openintrest, adjustclose
  let idx_arr = get_column_indexies(builder, header_line)?;

  let mut timestamp_vec = Vec::new();
  let mut data_vecs: [Vec<f64>; 7] = [(); 7].map(|_| Vec::new());
  for line in lines {
    // let mut dp = DataPoint::default();
    for (idx, seg) in line.trim().split(',').enumerate() {
      let idx = idx as i8; // 不考虑处理 csv 的 column 大于 127 列的 csv，as i8 直接 panic
      if idx == idx_arr[0] {
        match parse_time_field(&builder.time_type, seg) {
          Ok(v) => {
            timestamp_vec.push(v);
          }
          Err(e) => {
            panic!(
              "timestamp parse failed due to {}, please check 'time_type' config.",
              e
            );
          }
        }
        continue;
      }

      for (i, vec) in data_vecs.iter_mut().enumerate() {
        if idx == idx_arr[i + 1] {
          vec.push(parse_f64(seg)?);
        }
      }
    }
  }
  Ok((timestamp_vec, data_vecs))
}

#[inline]
pub(super) fn load_csv_from_file(
  file: &Path,
  builder: &CsvDataSourceBuilder,
) -> io::Result<LoadResult> {
  let content = read_to_string(file)?;
  load_csv_from_string(&content, builder)
}

#[inline]
pub(super) fn load_csv_from_string(
  content: &str,
  builder: &CsvDataSourceBuilder,
) -> io::Result<LoadResult> {
  let lines = content.lines().map(|l| l.trim()).filter(|l| !l.is_empty());
  load_csv_from_lines(lines, builder)
}
