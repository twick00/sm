use std::sync::mpsc::{Receiver, Sender};

use diesel::r2d2::{self, ConnectionManager};
use diesel::{
  insert_into, ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl, SqliteConnection,
};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Deserialize;
use serde_json::Map;
use tauri::{execute_promise, Webview};

use crate::db;
use crate::models::{FileDiff, FileDiffResult};
use crate::schema::file_details::dsl::file_details;
use crate::schema::file_diffs::dsl::file_diffs;
use std::fs::{metadata, File};
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::thread;

pub type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

pub mod cmd {
  use serde::Deserialize;
  use serde_json::Map;

  #[allow(non_snake_case)]
  #[derive(Deserialize)]
  #[serde(tag = "cmd", rename_all = "camelCase")]
  pub enum Cmd {
    // your custom commands
    SelectFile {
      selectFile: String,
      callback: String,
      error: String,
    },

    AddWatchedFiles {
      watchedFiles: Vec<String>,
      callback: String,
      error: String,
    },
  }
}

pub enum ExtendedEvent {
  Base(Event),
  AddedToWatch(PathBuf),
  RemovedFromWatch(PathBuf),
}

fn handle_file_select(conn: &SqliteConnection, path: String) -> Vec<FileDiff> {
  db::get_file_diffs_for_path(conn, &path).unwrap()
}

pub fn build_tauri_invoke_handler(
  tauri_database_pool: Pool,
  watched_file_sender: Sender<Vec<String>>,
) -> impl FnMut(&mut Webview, &str) -> Result<(), String> {
  move |_webview: &mut Webview, arg: &str| {
    use cmd::Cmd::*;
    match serde_json::from_str(arg) {
      Err(e) => Err(e.to_string()),
      Ok(command) => {
        match command {
          // definitions for your custom commands from Cmd here
          AddWatchedFiles {
            watchedFiles: watched_files,
            callback,
            error,
          } => {
            watched_file_sender.send(watched_files);
          }
          SelectFile {
            selectFile: select_file,
            callback,
            error,
          } => {
            println!("selectFile!");
            let conn = tauri_database_pool.get().unwrap();
            let result = handle_file_select(&conn, select_file.clone());
            execute_promise(
              _webview,
              move || {
                Ok(
                  result
                    .iter()
                    .map(|file_diff| file_diff.into())
                    .collect::<Vec<FileDiffResult>>(),
                )
              },
              callback,
              error,
            );
            println!("{}", select_file);
          }
        }
        Ok(())
      }
    }
  }
}

// fn event_to_str(wrapped_event: Option<DebouncedEvent>) -> &'static str {
//   match wrapped_event {
//     None => "None",
//     Some(event) => match event {
//       DebouncedEvent::NoticeWrite(_) => "NoticeWrite",
//       DebouncedEvent::NoticeRemove(_) => "NoticeRemove",
//       DebouncedEvent::Create(_) => "Create",
//       DebouncedEvent::Write(_) => "Write",
//       DebouncedEvent::Chmod(_) => "Chmod",
//       DebouncedEvent::Remove(_) => "Remove",
//       DebouncedEvent::Rename(_, _) => "Rename",
//       DebouncedEvent::Rescan => "Rescan",
//       DebouncedEvent::Error(_, _) => "Error",
//     },
//   }
// }

pub fn build_watched_file_change_listener(
  mut watcher: RecommendedWatcher,
  watched_file_list: Arc<RwLock<Vec<String>>>,
  watched_file_receiver: Receiver<Vec<String>>,
  file_change_notifier: Sender<ExtendedEvent>,
) -> impl FnMut() {
  move || {
    // Wait forever for new messages
    for updated_file_list in watched_file_receiver.iter() {
      println!("updated_file_list");
      let mut has_changed = false;
      // Add non-watched files to watcher
      for path in updated_file_list.iter() {
        if !watched_file_list.read().unwrap().contains(path) {
          let path_metadata = metadata(path).unwrap();

          if path_metadata.is_dir() {
            watcher.watch(path, RecursiveMode::Recursive);
          } else if path_metadata.is_file() {
            watcher.watch(path, RecursiveMode::NonRecursive);

            // Manually file `Create` event since the Watcher doesn't do it for us on adding the path
            file_change_notifier.send(ExtendedEvent::AddedToWatch(PathBuf::from(path)));
          }
          watched_file_list.write().unwrap().push(path.clone());
          has_changed = true;
        }
      }

      // Need to clone to remove the indexes from the inside the loop below
      let cloned_watched_file_list = watched_file_list.read().unwrap().clone();

      // Remove dropped files from watcher
      for (index, watched_path) in cloned_watched_file_list.iter().enumerate() {
        if !&updated_file_list.contains(watched_path) {
          watcher.unwatch(&watched_path);
          watched_file_list.write().unwrap().remove(index);
        }
      }
    }
  }
}

fn ez_buffer_from_file(path: PathBuf, event: &str) -> Vec<u8> {
  let mut f = File::open(&path).unwrap();
  let file_size = metadata(&path).unwrap().len() as usize;
  let mut buffer = vec![0; file_size];
  f.read(&mut buffer).expect(&*format!(
    "Error Occurred for event \"{}\": buffer overflow",
    event
  ));
  buffer
}

pub fn build_file_change_listener(
  pool: &Pool,
  file_change_listener: Receiver<ExtendedEvent>,
) -> impl Fn() {
  let file_change_pool = pool.clone().get().unwrap();
  move || {
    for file_change_event in file_change_listener.iter() {
      match file_change_event {
        // DebouncedEvent::Create(path) => {
        //   let path_clone = path.as_os_str().to_str().unwrap();
        //   println!("Found Create event file at path: {}", path_clone);
        //   let mut f = File::open(path.clone()).unwrap();
        //   let file_size = metadata(&path).unwrap().len() as usize;
        //   let mut buffer = vec![0; file_size];
        //   f.read(&mut buffer)
        //     .expect("Error Occurred for event \"Create\": buffer overflow");
        //   db::create_file_details(&file_change_pool, buffer, path_clone);
        //   // let last_row_id = insert_or_update_file_source(&buffer, path_clone, &conn);
        // }
        // DebouncedEvent::Write(path) => {
        //   let path_clone = path.as_os_str().to_str().unwrap();
        //   println!("Found Write event file at path: {}", path_clone);
        //   let mut f = File::open(path.clone()).unwrap();
        //   let file_size = metadata(&path).unwrap().len() as usize;
        //   let mut buffer = vec![0; file_size];
        //   f.read(&mut buffer)
        //     .expect("Error Occurred for event \"Write\": buffer overflow");
        //
        //   db::insert_file_diff(&file_change_pool, buffer, path_clone, "Write");
        // }
        ExtendedEvent::Base(event) => match event.kind {
          EventKind::Any => println!("Event fired: Any"),
          EventKind::Access(_) => {}
          EventKind::Create(_) => {}
          EventKind::Modify(p) => {}
          EventKind::Remove(_) => {}
          EventKind::Other => {}
        },
        ExtendedEvent::AddedToWatch(path) => {
          let buffer = ez_buffer_from_file(path.clone(), "AddedToWatch");
          db::add_file_details(&file_change_pool, buffer, &path.to_str().unwrap());
        }
        ExtendedEvent::RemovedFromWatch(path) => {
          db::remove_all_for_path(&file_change_pool, &path.to_str().unwrap());
        }
      }
    }
  }
}
