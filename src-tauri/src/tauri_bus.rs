use std::fs::{metadata, File};
use std::io::Read;

use std::time::Duration;

use anyhow::Result;
use crossbeam::channel::{after, never, select, Receiver, Sender};

use diesel::r2d2::{self, ConnectionManager};
use diesel::SqliteConnection;
use notify::{Event, EventKind, RecommendedWatcher, Watcher};

use tauri::{Webview, WebviewMut};

use crate::db;
use crate::models::FileDiff;

pub type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

pub mod cmd {
  use serde::Deserialize;

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

#[derive(Clone, Debug)]
pub enum RequestEvent {
  WatchedFileList(),
  SelectFile(String),
}

#[derive(Clone, Debug)]
pub enum ResponseEvent {
  WatchedFileList(Vec<String>),
  SelectFile(Vec<FileDiff>),
  Error(String),
}

#[derive(Clone)]
pub enum BusEvent {
  Base(Event),
  AddedToWatch(String),
  RemovedFromWatch(String),
  UpdateWatched(Vec<String>),
  Request(RequestEvent),
  Response(ResponseEvent),
}

fn handle_file_select(conn: &SqliteConnection, path: String) -> Vec<FileDiff> {
  db::get_file_diffs_for_path(conn, &path).unwrap()
}

fn ez_event_response<F>(
  receiver: Receiver<ResponseEvent>,
  sender: Sender<BusEvent>,
  webview: WebviewMut,
  req: RequestEvent,
  mut func: F,
) where
  F: FnMut(WebviewMut, ResponseEvent),
{
  sender.send(BusEvent::Request(req)).unwrap();
  let duration = Some(Duration::from_millis(100));
  let timeout = duration.map(|d| after(d)).unwrap_or(never());
  select! {
    recv(receiver) -> msg => {
      func(webview, msg.unwrap())
    }
    recv(timeout) -> _ => {
      func(webview, ResponseEvent::Error("TIMEOUT".to_string()))
    }
  }
}

pub fn build_tauri_setup_handler(
  sender: Sender<BusEvent>,
  _response_sender: Sender<ResponseEvent>,
  response_receiver: Receiver<ResponseEvent>,
) -> impl FnMut(&mut Webview, String) {
  move |webview: &mut Webview, _source_window: String| {
    let webview = webview.as_mut();

    // refreshWatchedFileListRequest handler
    let response_receiver_clone = response_receiver.clone();
    let sender_clone = sender.clone();
    let webview_clone = webview.clone();
    tauri::event::listen("refreshWatchedFileListRequest", move |_| {
      ez_event_response(
        response_receiver_clone.clone(),
        sender_clone.clone(),
        webview_clone.clone(),
        RequestEvent::WatchedFileList(),
        move |mut webview, response| match response {
          ResponseEvent::WatchedFileList(watched_file_list) => {
            tauri::event::emit(
              &mut webview,
              "refreshWatchedFileListResponse",
              Some(watched_file_list),
            )
            .unwrap();
          }
          ResponseEvent::Error(error_message) => {
            println!("REQUEST ERROR: {}", error_message);
            tauri::event::emit(&mut webview, "error", Some(error_message)).unwrap();
          }
          _ => {}
        },
      );
    });

    // selectFileRequest handler
    let response_receiver_clone = response_receiver.clone();
    let sender_clone = sender.clone();
    let webview_clone = webview.clone();
    tauri::event::listen("selectFileRequest", move |file_name| {
      ez_event_response(
        response_receiver_clone.clone(),
        sender_clone.clone(),
        webview_clone.clone(),
        RequestEvent::SelectFile(file_name.clone().unwrap()),
        move |mut webview, response| match response {
          ResponseEvent::SelectFile(watched_file_list) => {
            tauri::event::emit(
              &mut webview,
              "refreshWatchedFileListResponse",
              Some(watched_file_list),
            )
            .unwrap();
          }
          ResponseEvent::Error(error_message) => {
            println!("REQUEST ERROR: {}", error_message);
            tauri::event::emit(&mut webview, "error", Some(error_message)).unwrap();
          }
          _ => {}
        },
      );
    })
  }
}

pub fn build_tauri_invoke_handler(
  sender: Sender<BusEvent>,
  _receiver: Receiver<BusEvent>,
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
            callback: _,
            error: _,
          } => {
            println!("AddWatchedFiles: {:?}", watched_files);
            sender.send(BusEvent::UpdateWatched(watched_files));
          }
          SelectFile {
            selectFile: _select_file,
            callback: _,
            error: _,
          } => {}
        }
        Ok(())
      }
    }
  }
}

pub fn ez_buffer_from_file<S: AsRef<str>>(path: S, event: &str) -> Vec<u8> {
  let mut f = File::open(path.as_ref()).unwrap();
  let file_size = metadata(path.as_ref()).unwrap().len() as usize;
  let mut buffer = vec![0; file_size];
  f.read(&mut buffer).expect(&*format!(
    "Error Occurred for event \"{}\": buffer overflow",
    event
  ));
  buffer
}

pub fn register_file_watcher(sender: Sender<BusEvent>) -> notify::Result<RecommendedWatcher> {
  Watcher::new_immediate(move |res| {
    match res {
      Ok(event) => {
        sender.send(BusEvent::Base(event));
      }
      Err(e) => println!("watch error: {:?}", e),
    };
  })
}

pub fn build_file_change_listener(
  pool: &Pool,
  file_change_listener: Receiver<BusEvent>,
) -> impl FnMut() {
  let file_change_pool = pool.clone();
  move || {
    for file_change_event in file_change_listener.iter() {
      match file_change_event {
        BusEvent::Base(event) => match event.kind {
          EventKind::Any => println!("Event fired: Any"),
          EventKind::Access(_) => {}
          EventKind::Create(_) => {}
          EventKind::Modify(_p) => {}
          EventKind::Remove(_) => {}
          EventKind::Other => {}
        },
        BusEvent::AddedToWatch(path) => {
          let buffer = ez_buffer_from_file(path.clone(), "AddedToWatch");
          db::add_file_details(&file_change_pool.get().unwrap(), buffer, path);
        }
        BusEvent::RemovedFromWatch(path) => {
          db::remove_all_for_path(&file_change_pool.get().unwrap(), &path.as_str());
        }
        _ => {}
      }
    }
  }
}
