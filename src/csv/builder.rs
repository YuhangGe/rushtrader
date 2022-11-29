use std::{
  io::{self},
  path::Path,
};

use super::{
  source::CsvDataSource,
  util::{load_csv_from_file, load_csv_from_lines, load_csv_from_string, new_io_err_str},
};

pub enum CsvTimeType {
  Unknown,
  Second,
  Millsecond,
  Date(&'static str),
  Datetime(&'static str),
}

macro_rules! gen_builder {
    ($struct_name: ident, $($name: ident : $default_value: literal), +) => {
      pub struct $struct_name {
        pub(super) time_type: CsvTimeType,
        $ (
          pub(super) $name: String,
        )*
      }
      impl $struct_name {
        pub fn new() -> Self {
          Self {
            time_type: CsvTimeType::Unknown,
            $ (
              $name: $default_value.to_string(),
            )*
          }
        }
        $(
        #[doc = concat!("配置 ", stringify!($name)," 字段，忽略大小写，默认为 ", $default_value)]
        pub fn $name(mut self, field: &str) -> Self {
          self.$name = field.to_lowercase();
          self
        }
        )*
      }
      impl Default for $struct_name {
        fn default() -> Self {
          Self::new()
        }
      }
    };
}

gen_builder!(
  CsvDataSourceBuilder,
  time_field: "",
  open_field: "open",
  close_field: "close",
  high_field: "high",
  low_field: "low",
  volume_field: "volume",
  openintrest_field: "open intrest",
  adjustclose_field: "adj close"
);

impl CsvDataSourceBuilder {
  /// 指定 time 列的类型，默认为 Detect，会根据正则检测自动匹配类型。还可以指定为：
  /// Second 时间戳的秒数
  /// Millsecond 时间戳的毫秒数（标准时间戳）
  /// Date 日期字符串，需要同时指定字符串的解析格式
  /// Datetime 日期时间字符串，需要同时指定字符串的解析格式
  pub fn time_type(mut self, time_type: CsvTimeType) -> Self {
    self.time_type = time_type;
    self
  }
  fn check_config(&self) -> io::Result<()> {
    if self.time_field.is_empty() {
      return Err(new_io_err_str("time_field config missing"));
    }
    if matches!(self.time_type, CsvTimeType::Unknown) {
      return Err(new_io_err_str("time_type config missing"));
    }
    Ok(())
  }
  /// 加载全部数据到内存中。
  pub fn load_from_file(self, file: &Path) -> io::Result<CsvDataSource> {
    self.check_config()?;
    let (timestamp_vec, data_vecs) = load_csv_from_file(file, &self)?;
    Ok(CsvDataSource::inner_new(timestamp_vec, data_vecs))
  }
  pub fn load_from_string(self, content: &str) -> io::Result<CsvDataSource> {
    self.check_config()?;
    let (timestamp_vec, data_vecs) = load_csv_from_string(content, &self)?;
    Ok(CsvDataSource::inner_new(timestamp_vec, data_vecs))
  }
  pub fn load_from_lines<'a, T: Iterator<Item = &'a str>>(
    self,
    lines: T,
  ) -> io::Result<CsvDataSource> {
    self.check_config()?;
    let (timestamp_vec, data_vecs) = load_csv_from_lines(lines, &self)?;
    Ok(CsvDataSource::inner_new(timestamp_vec, data_vecs))
  }
}
