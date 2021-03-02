use crate::schema::*;

use std::path::Component;
use std::path::Path;


#[derive(Debug, Queryable, Insertable)]
pub struct FileDetail {
  pub id: Option<i32>,
  pub file_path: String,
  pub data: Vec<u8>,
  pub timestamp: i32,
}

#[derive(Clone, Debug, Queryable, Insertable, serde::Serialize)]
pub struct FileDiff {
  pub id: Option<i32>,
  pub original_file_id: Option<i32>,
  pub change_event: String,
  pub file_path: String,
  pub data: Vec<u8>,
  pub timestamp: i32,
}

#[derive(Debug, serde::Serialize)]
pub struct FileDiffResult {
  pub id: Option<i32>,
  pub original_file_id: Option<i32>,
  pub change_event: String,
  pub file_path: Vec<String>,
  pub data: String,
  pub timestamp: i32,
}

impl std::convert::From<&FileDiff> for FileDiffResult {
  // Convert data from Vec<u8> to String before passing it to the FE
  fn from(file_diff: &FileDiff) -> Self {
    FileDiffResult {
      id: file_diff.id,
      original_file_id: file_diff.original_file_id.clone(),
      change_event: file_diff.change_event.clone(),
      // Javascript isn't great at parsing path components in a cross-platform way so let rust handle it
      file_path: Path::new(&file_diff.file_path)
        .components()
        .into_iter()
        .fold(Vec::new(), |mut a: Vec<String>, b: Component| {
          match b {
            Component::Normal(segment) => a.push(segment.to_str().unwrap().to_string()),
            _ => {}
          }
          a
        }),
      data: std::str::from_utf8(&file_diff.data).unwrap().to_string(),
      timestamp: file_diff.timestamp.clone(),
    }
  }
}
