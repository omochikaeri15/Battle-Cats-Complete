use std::sync::{mpsc, Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::{Duration, Instant};

// STRICT BOUNDARY: Only importing the public aggregate roots
use nyanko::graphics::animation::{Unit, Anim};
use crate::animation::export::state::LoopStatus;

const TIMEOUT_SECONDS: u64 = 180;

pub fn start_search(
    unit: Arc<Unit>,
    animation: Arc<Anim>,
    tolerance: f32,
    minimum_loop_length: i32,
    maximum_loop_length: Option<i32>,
    status_sender: mpsc::Sender<LoopStatus>,
    abort_signal: Arc<AtomicBool>
) {
    thread::spawn(move || {
        let start_time = Instant::now();

        // Delegate the heavy lifting to the library, passing a callback for orchestrator logic
        let cycle_result = unit.calculate_cycle(
            &animation,
            tolerance,
            Some(minimum_loop_length),
            maximum_loop_length,
            |current_frame| {
                // Return false to instruct the nyanko engine to abort the search
                if abort_signal.load(Ordering::Relaxed) {
                    return false;
                }

                if start_time.elapsed().as_secs() > TIMEOUT_SECONDS {
                    let _ = status_sender.send(LoopStatus::Error("Timed out (3 mins)".to_string()));
                    return false;
                }

                // GUI updates
                if current_frame % 5 == 0 {
                    let _ = status_sender.send(LoopStatus::Searching(current_frame));
                }

                // Yield thread execution briefly to prevent locking the GUI host
                if current_frame % 100 == 0 {
                    thread::sleep(Duration::from_millis(1));
                }

                true // Continue searching
            }
        );

        match cycle_result {
            Some((start_frame, end_frame)) => {
                let _ = status_sender.send(LoopStatus::Found(start_frame, end_frame));
            }
            None => {
                if !abort_signal.load(Ordering::Relaxed) && start_time.elapsed().as_secs() <= TIMEOUT_SECONDS {
                    let _ = status_sender.send(LoopStatus::Error("No loop found within limits".to_string()));
                }
            }
        }
    });
}