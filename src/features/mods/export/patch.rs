use std::sync::{mpsc::{self, Receiver, Sender}, Mutex};
use std::thread;

use crate::features::mods::logic::state::ModState;

pub enum ExportEvent {
    Log(String),
    Success(String), // Passing the final success message
    Error(String),
}

pub static EVENT_RECEIVER: Mutex<Option<Receiver<ExportEvent>>> = Mutex::new(None);

pub fn spawn_log_adapter(event_tx: Sender<ExportEvent>) -> Sender<String> {
    let (str_tx, str_rx) = mpsc::channel();
    thread::spawn(move || {
        for msg in str_rx {
            let _ = event_tx.send(ExportEvent::Log(msg));
        }
    });
    str_tx
}

pub fn process_events(state: &mut ModState) -> bool {
    let mut busy = state.export.is_busy;
    if let Ok(guard) = EVENT_RECEIVER.try_lock() {
        if let Some(rx) = guard.as_ref() {
            while let Ok(event) = rx.try_recv() {
                match event {
                    ExportEvent::Log(msg) => state.export.log_content.push_str(&format!("{}\n", msg)),
                    ExportEvent::Success(msg) => {
                        state.export.log_content.push_str(&format!("{}\n", msg));
                        state.export.status_message = "Complete!".to_string();
                        state.export.is_busy = false;
                        busy = false;
                    },
                    ExportEvent::Error(err) => {
                        state.export.log_content.push_str(&format!("!! ERROR: {}\n", err));
                        state.export.status_message = "Failed".to_string();
                        state.export.is_busy = false;
                        busy = false;
                    }
                }
            }
        }
    }
    busy
}