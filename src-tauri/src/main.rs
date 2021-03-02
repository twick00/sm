#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]
#[macro_use]
extern crate diesel;

use anyhow::Result;
use crossbeam::channel::{bounded, unbounded};

use diesel::r2d2::{ConnectionManager};

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use std::collections::{HashSet};







use crate::models::{FileDiff};


use crate::tauri_bus::{
  build_tauri_invoke_handler, build_tauri_setup_handler,
  ez_buffer_from_file, register_file_watcher, BusEvent, Pool, RequestEvent, ResponseEvent,
};

mod db;
mod models;
mod schema;
mod tauri_bus;

// Note that this was my first time messing with cross-thread communication in rust so the code will be rough around the edges
fn main() -> Result<()> {
  dotenv::dotenv().ok();
  let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL NOT FOUND");
  let db_pool: Pool = Pool::builder().build(ConnectionManager::new(database_url))?;

  let (sender, receiver) = unbounded::<BusEvent>();
  let (_request_sender, _request_receiver) = bounded::<RequestEvent>(5);
  let (response_sender, response_receiver) = bounded::<ResponseEvent>(5);

  let mut watcher: RecommendedWatcher = register_file_watcher(sender.clone())?;

  let tauri_build = tauri::AppBuilder::new()
    .setup(build_tauri_setup_handler(
      sender.clone(),
      response_sender.clone(),
      response_receiver,
    ))
    .invoke_handler(build_tauri_invoke_handler(sender.clone(), receiver.clone()))
    .build();

  std::thread::spawn(move || {
    let mut watched_file_set = HashSet::new();
    let pool = db_pool.clone();
    for bus_event in receiver {
      match bus_event {
        BusEvent::Base(event) => match event.kind {
          EventKind::Any => {
            //TODO: this
          }
          EventKind::Access(_) => {
            //TODO: this
          }
          EventKind::Create(_) => {
            //TODO: this
          }
          EventKind::Modify(_) => {
            //TODO: this
          }
          EventKind::Remove(_) => {
            //TODO: this
          }
          EventKind::Other => {
            //TODO: this
          }
        },
        BusEvent::AddedToWatch(path) => {
          let buffer = ez_buffer_from_file(&path, "AddedToWatch");
          db::add_file_details(&pool.get().unwrap(), buffer, &path);
          watcher.watch(&path, RecursiveMode::NonRecursive);
          watched_file_set.insert(path);
        }
        BusEvent::RemovedFromWatch(path) => {
          watcher.unwatch(&path);
          db::remove_all_for_path(&pool.get().unwrap(), &path);
          watched_file_set.remove(&path);
        }

        BusEvent::UpdateWatched(paths) => {
          println!("UpdateWatched! {:?}", paths);
          let sender = sender.clone();
          let updated_watched_file_set: HashSet<String> = paths.clone().into_iter().collect();

          let files_to_remove: Vec<String> = watched_file_set
            .difference(&updated_watched_file_set)
            .map(|s| s.into())
            .collect();

          for removed_file in files_to_remove {
            sender
              .send(BusEvent::RemovedFromWatch(removed_file))
              .unwrap();
          }

          let files_to_add: Vec<String> = updated_watched_file_set
            .difference(&watched_file_set)
            .map(|s| s.into())
            .collect();

          for added_file in files_to_add {
            sender.send(BusEvent::AddedToWatch(added_file)).unwrap();
          }
          watched_file_set = updated_watched_file_set;
        }
        BusEvent::Request(request) => match request {
          RequestEvent::WatchedFileList() => {
            println!("FOUND: RequestEvent::WatchedFileList");
            let watched_file_list: Vec<String> = watched_file_set.clone().into_iter().collect();
            response_sender.send(ResponseEvent::WatchedFileList(watched_file_list));
          }
          RequestEvent::SelectFile(_path) => {
            // Testing RequestEvent::SelectFile
            let t1 = FileDiff {
              id: Some(1),
              original_file_id: Some(123),
              change_event: "TestEvent".to_string(),
              file_path: "/test/file/file1".to_string(),
              data: vec![1, 2, 3],
              timestamp: 0,
            };
            let t2 = FileDiff {
              id: Some(2),
              original_file_id: Some(123),
              change_event: "TestEvent".to_string(),
              file_path: "/test/file/file1".to_string(),
              data: vec![1, 2, 3, 4, 5, 6],
              timestamp: 0,
            };
            response_sender.send(ResponseEvent::SelectFile(vec![t1, t2]));
            unimplemented!();
          }
        },
        BusEvent::Response(response) => {
          // Forward the response to the next receiver
          sender.send(BusEvent::Response(response));
        }
      }
    }
  });

  tauri_build.run();
  Ok(())
  // Below cannot listen to the event bus, can only send new events with sender
}
