use crate::schema::*;
use serde::{Serialize, Serializer};
use tauri::plugin::Plugin;

#[derive(Debug, Queryable, Insertable)]
pub struct FileDetail {
  pub id: Option<i32>,
  pub file_path: String,
  pub data: Vec<u8>,
  pub timestamp: i32,
}

#[derive(Debug, Queryable, Insertable, serde::Serialize)]
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
  pub file_path: String,
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
      file_path: file_diff.file_path.clone(),
      data: std::str::from_utf8(&file_diff.data).unwrap().to_string(),
      timestamp: file_diff.timestamp.clone(),
    }
  }
}