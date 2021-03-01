#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]
#[macro_use]
extern crate diesel;

use std::borrow::BorrowMut;
use std::collections::{HashMap, HashSet};
use std::fs::{metadata, File};
use std::io::{BufRead, BufReader, Error, Read};
use std::iter::FromIterator;
use std::path::PathBuf;
use std::sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender};
use std::thread;
use std::{io, str};

use bus::{Bus, BusReader};
use diesel::r2d2::{self, ConnectionManager, Pool};
use diesel::{
  insert_into, ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl, SqliteConnection,
};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{execute_promise, Webview};

use models::FileDetail;

use crate::models::{FileDiff, FileDiffResult};
use crate::schema::file_details::dsl::file_details;
use crate::schema::file_diffs::dsl::file_diffs;
use crate::tauri_bus::{
  build_file_change_listener, build_watched_file_change_listener, ez_buffer_from_file,
  handle_file_change_event, BusEvent,
};

mod db;
mod models;
mod schema;
mod tauri_bus;

fn spawn_event_bus<F>(f: F) -> (SyncSender<BusEvent>, Receiver<BusEvent>, Bus<BusEvent>)
where
  F: Fn(Bus<BusEvent>, SyncSender<BusEvent>) -> Bus<BusEvent>,
{
  // mpsc uses a multi-provider single-consumer pattern. Bus uses a single-provider, multi-consumer pattern
  // With these powers combined, we have a multi-provider, multi-consumer pattern to create a _real_ event bus.
  let (tx_prime, mix_rx) = sync_channel::<BusEvent>(100);
  let mut bus: Bus<BusEvent> = Bus::new(10);

  // Pass the bus to the closure to register any message listeners, then return it to start broadcasting
  bus = f(bus, tx_prime.clone());

  (tx_prime, mix_rx, bus)
}

// Note that this was my first time messing with cross-thread communication in rust so the code will be rough around the edges
fn main() {
  dotenv::dotenv().ok();
  let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL NOT FOUND");
  let database_pool = Pool::builder()
    .build(ConnectionManager::new(database_url))
    .unwrap();

  let primary_event_db_pool = database_pool.clone();
  // Register all listeners to the event_bus

  let tauri_database_pool = database_pool.clone();
  tauri::AppBuilder::new()
    .setup(move |webview, _source| {
      let mut webview = webview.as_mut();
      let mut webview2 = webview.clone();

      println!("BEGIN SETUP!");
      println!("{}", _source);
      let (sender, receiver, mut bus) = spawn_event_bus(|mut event_bus, sync_sender| {
        let watched_file_listener = event_bus.add_rx();
        let file_change_listener = event_bus.add_rx();
        let pool = primary_event_db_pool.clone();
        let mut webview3 = webview2.clone();

        let primary_event_listener = event_bus.add_rx();
        let primary_event_notifier = sync_sender.clone();

        let file_change_notifier = sync_sender.clone();
        let mut watcher: RecommendedWatcher =
          Watcher::new_immediate(move |res: notify::Result<notify::Event>| match res {
            Ok(event) => file_change_notifier.send(BusEvent::Base(event)).unwrap(),
            Err(e) => println!("watch error: {:?}", e),
          })
          .unwrap();

        // Primary event bus handler
        let db_pool = pool.clone();
        thread::spawn(move || {
          let mut webview4 = webview3.clone();
          let mut watched_file_set = HashSet::new();
          // let mut webview2 = webview.clone();
          for bus_event in primary_event_listener {
            match bus_event {
              BusEvent::Base(file_change_event) => handle_file_change_event(
                &db_pool,
                file_change_event,
                primary_event_notifier.clone(),
              ),
              BusEvent::AddedToWatch(path) => {
                let buffer = ez_buffer_from_file(&path, "AddedToWatch");
                db::add_file_details(&db_pool.get().unwrap(), buffer, &path);
                watcher.watch(&path, RecursiveMode::NonRecursive);
                watched_file_set.insert(path);
              }
              BusEvent::RemovedFromWatch(path) => {
                watcher.unwatch(&path);
                db::remove_all_for_path(&db_pool.get().unwrap(), &path);
                watched_file_set.remove(&path);
              }
              BusEvent::UpdateWatched(paths) => {
                let bus_update_notifier = primary_event_notifier.clone();
                let updated_watched_file_set: HashSet<String> = paths.clone().into_iter().collect();

                let files_to_remove: Vec<String> = watched_file_set
                  .difference(&updated_watched_file_set)
                  .map(|s| s.into())
                  .collect();

                for removed_file in files_to_remove {
                  bus_update_notifier.send(BusEvent::RemovedFromWatch(removed_file));
                }

                let files_to_add: Vec<String> = updated_watched_file_set
                  .difference(&watched_file_set)
                  .map(|s| s.into())
                  .collect();

                for added_file in files_to_add {
                  bus_update_notifier.send(BusEvent::AddedToWatch(added_file));
                }
                watched_file_set = updated_watched_file_set;
                tauri::event::emit(&mut webview4, "updateWatched", Some(paths));
              }
            }
          }
        });
        event_bus
      });

      thread::spawn(move || {
        for message in receiver.iter() {
          bus.broadcast(message);
          println!("MESSAGE!");
        }
      });

      println!("END SETUP!");
    })
    // .invoke_handler(|webview, arg| Ok(()))
    .build()
    .run();

  // Below cannot listen to the event bus, can only send new events with sender
}
