use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Thread-safe queue for `/steer` texts injected during AI output.
///
/// The queue supports concurrent push (from stdin polling) and
/// drain (from the `run_turn` conversation loop).
pub(crate) type SteerQueue = Arc<Mutex<VecDeque<String>>>;

/// Create a new empty `SteerQueue`.
pub(crate) fn new_steer_queue() -> SteerQueue {
    Arc::new(Mutex::new(VecDeque::new()))
}

/// Push a steer text into the queue.
pub(crate) fn steer_push(queue: &SteerQueue, text: String) {
    if text.trim().is_empty() {
        return;
    }
    let mut guard = queue.lock().unwrap_or_else(|e| e.into_inner());
    guard.push_back(text);
}

/// Drain all pending steer texts from the queue, returning them in FIFO order.
pub(crate) fn steer_drain(queue: &SteerQueue) -> Vec<String> {
    let mut guard = queue.lock().unwrap_or_else(|e| e.into_inner());
    guard.drain(..).collect()
}

/// Poll stdin for `/steer <text>` input using crossterm non-blocking reads.
///
/// This function checks if there is pending stdin data during AI output.
/// Currently a placeholder — the primary input path is through the idle
/// REPL prompt's rustyline handler. Will be enhanced with raw-mode stdin
/// line accumulation in a future iteration.
pub(crate) fn poll_steer_input(_queue: &SteerQueue) -> usize {
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn push_and_drain_single() {
        let queue = new_steer_queue();
        steer_push(&queue, "focus on tests".to_string());
        let drained = steer_drain(&queue);
        assert_eq!(drained, vec!["focus on tests"]);
        // Queue should be empty after drain
        assert!(steer_drain(&queue).is_empty());
    }

    #[test]
    fn push_and_drain_multiple_in_order() {
        let queue = new_steer_queue();
        steer_push(&queue, "A".to_string());
        steer_push(&queue, "B".to_string());
        let drained = steer_drain(&queue);
        assert_eq!(drained, vec!["A", "B"]);
    }

    #[test]
    fn push_ignores_empty_and_whitespace() {
        let queue = new_steer_queue();
        steer_push(&queue, "".to_string());
        steer_push(&queue, "   ".to_string());
        steer_push(&queue, "valid".to_string());
        let drained = steer_drain(&queue);
        assert_eq!(drained, vec!["valid"]);
    }

    #[test]
    fn concurrent_push_is_safe() {
        let queue = new_steer_queue();
        let handles: Vec<_> = (0..4)
            .map(|i| {
                let q = Arc::clone(&queue);
                thread::spawn(move || {
                    steer_push(&q, format!("steer-{i}"));
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        let mut drained = steer_drain(&queue);
        drained.sort();
        assert_eq!(
            drained,
            vec!["steer-0", "steer-1", "steer-2", "steer-3"]
        );
    }
}
