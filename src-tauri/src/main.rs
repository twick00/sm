#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]
#[macro_use]
extern crate diesel;

use std::collections::HashMap;
use std::fs::{metadata, File};
use std::hash::Hash;
use std::io::{BufRead, BufReader, Error, Read};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc::{channel, Sender};
use std::sync::RwLock;
use std::sync::{Arc, LockResult, RwLockReadGuard};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{io, str};

use diesel::r2d2::{self, ConnectionManager};
use diesel::{
  insert_into, ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl, SqliteConnection,
};
use notify::{RecommendedWatcher, Watcher};
use tauri::{execute_promise, Webview};

use models::FileDetail;

use crate::models::{FileDiff, FileDiffResult};
use crate::schema::file_details::dsl::file_details;
use crate::schema::file_diffs::dsl::file_diffs;
use crate::tauri_bus::{
  build_file_change_listener, build_watched_file_change_listener, ExtendedEvent,
};

mod db;
mod models;
mod schema;
mod tauri_bus;

pub type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

// Note that this was my first time messing with cross-thread communication in rust so the code will be rough around the edges
fn main() {
  // Channel to update the list of watched files
  let (watched_file_sender, watched_file_receiver) = channel::<Vec<String>>();

  // Result of notify::Event to tell us what file has changed
  let (file_change_notifier, file_change_listener) = channel::<ExtendedEvent>();

  // Copy notifier into watcher closure
  let ref_file_change_notifier = file_change_notifier.clone();

  let watcher: RecommendedWatcher =
    Watcher::new_immediate(move |res: notify::Result<notify::Event>| match res {
      Ok(event) => ref_file_change_notifier
        .send(ExtendedEvent::Base(event))
        .unwrap(),
      Err(e) => println!("watch error: {:?}", e),
    })
    .unwrap();

  // Arc: Atomic reference counter, used for passing values safely across threads
  // RwLock: Read/Write lock, somewhat like a mutex except it can have unlimited "readers"
  let mut watched_file_list: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::new(Vec::new()));

  // let conn = diesel::sqlite::SqliteConnection::establish(":memory:").unwrap();

  // let conn = setup_db(Connection::open_in_memory().unwrap());
  dotenv::dotenv().ok();
  let database_url = std::env::var("DATABASE_URL").expect("NOT FOUND");
  let database_pool = Pool::builder()
    .build(ConnectionManager::new(database_url))
    .unwrap();

  // Handles adding/removing watched files
  thread::spawn(build_file_change_listener(
    &database_pool,
    file_change_listener,
  ));

  // Handles file events for watched files
  thread::spawn(build_watched_file_change_listener(
    watcher,
    watched_file_list,
    watched_file_receiver,
    file_change_notifier,
  ));

  let tauri_database_pool = database_pool.clone();
  tauri::AppBuilder::new()
    .invoke_handler(tauri_bus::build_tauri_invoke_handler(
      tauri_database_pool,
      watched_file_sender,
    ))
    .build()
    .run();
}
