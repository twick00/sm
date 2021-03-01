use crate::models::*;
use crate::schema::file_details::dsl::file_details;
use crate::schema::file_diffs::dsl::file_diffs;
use diesel::{
  insert_into, ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl, SqliteConnection,
};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn add_file_details<S: Into<String>>(
  conn: &SqliteConnection,
  file_contents: Vec<u8>,
  file_path: S,
) -> usize {
  println!("create_file_details");
  let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs() as i32;

  let new_file_detail = FileDetail {
    id: None,
    file_path: file_path.into(),
    data: file_contents,
    timestamp,
  };

  diesel::replace_into(file_details)
    .values(new_file_detail)
    .execute(conn)
    .expect("Error saving new post")
}

pub fn remove_file_diffs_for_path<S: AsRef<str>>(conn: &SqliteConnection, file_path: S) -> usize {
  use crate::schema::file_diffs::dsl::file_path as diffs_file_path;
  diesel::delete(file_diffs)
    .filter(diffs_file_path.eq(file_path.as_ref()))
    .execute(conn)
    .unwrap()
}

pub fn remove_file_details_for_path<S: AsRef<str>>(conn: &SqliteConnection, file_path: S) -> usize {
  use crate::schema::file_details::dsl::file_path as details_file_path;
  diesel::delete(file_details)
    .filter(details_file_path.eq(file_path.as_ref()))
    .execute(conn)
    .unwrap()
}

pub fn remove_all_for_path<S: AsRef<str>>(conn: &SqliteConnection, file_path: S) -> (usize, usize) {
  (
    remove_file_diffs_for_path(conn, file_path.as_ref()),
    remove_file_details_for_path(conn, file_path.as_ref()),
  )
}

pub fn insert_file_diff(
  conn: &SqliteConnection,
  file_contents: Vec<u8>,
  input_file_path: &str,
  change_event: &str,
) {
  use crate::schema::file_details::dsl::*;
  use crate::schema::file_diffs::dsl::{
    change_event as fd_change_event, data as fd_data, file_path as fd_file_path, id as fd_id,
    original_file_id as fd_original_file_id, timestamp as fd_timestamp,
  };
  println!("insert_file_diff");
  let source_file_data: diesel::QueryResult<(Option<i32>, Vec<u8>)> = file_details
    .select((id, data))
    .filter(file_path.eq(input_file_path))
    .limit(1)
    .first::<(Option<i32>, Vec<u8>)>(conn);

  if source_file_data.is_err() {
    // TODO: handle this
    println!("{}", source_file_data.unwrap_err());
    unimplemented!()
  }

  let (original_file_id, original_file_contents) = source_file_data.unwrap();

  let file_diff = diffy::create_patch_bytes(&original_file_contents, &file_contents);

  let since_the_epoch = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_millis() as i32;

  let new_file_diff = FileDiff {
    id: None,
    original_file_id: Some(original_file_id.unwrap()),
    change_event: change_event.to_string(),
    file_path: input_file_path.to_string(),
    data: file_diff.to_bytes(),
    timestamp: since_the_epoch,
  };

  insert_into(file_diffs)
    .values(&new_file_diff)
    .execute(conn)
    .expect("Error on inserting to file_diffs table");
}

pub fn get_file_diffs_for_path(conn: &SqliteConnection, path: &str) -> QueryResult<Vec<FileDiff>> {
  use crate::schema::file_diffs;
  use crate::schema::file_diffs::dsl::file_path;
  use crate::schema::file_diffs::dsl::timestamp;
  file_diffs
    .filter(file_path.eq(path))
    .order(timestamp.desc())
    .limit(5)
    .load(conn)
}
