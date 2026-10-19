//! Formatting off the UI thread: one thread per request (a stuck program never holds the next save),
//! the answer as edits against the text that was sent, tagged with the caller's key and ticket — the
//! caller decides whether it still applies.

use crate::diff::{Edit, changes};
use crate::run::{Program, Refusal};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The edits to the text that was sent (empty = it was already formatted).
    Changes(Vec<Edit>),
    Refused(Refusal),
}

#[derive(Debug)]
pub struct Done<K> {
    pub key: K,
    pub ticket: u64,
    pub outcome: Outcome,
}

pub struct Worker<K> {
    tx: Sender<Done<K>>,
    rx: Receiver<Done<K>>,
    in_flight: usize,
    /// What [`Worker::wait`] received, for the next [`Worker::poll`].
    ready: Vec<Done<K>>,
}

impl<K: Send + 'static> Default for Worker<K> {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self {
            tx,
            rx,
            in_flight: 0,
            ready: Vec::new(),
        }
    }
}

impl<K: Clone + Send + 'static> Worker<K> {
    /// Formats `text` with `program` on a new thread.
    pub fn submit(&mut self, program: &Arc<Program>, key: K, ticket: u64, text: String) {
        let (tx, program, mine) = (self.tx.clone(), Arc::clone(program), key.clone());
        let spawned = std::thread::Builder::new()
            .name("ph2d-format".into())
            .spawn(move || {
                let outcome = match program.format(&text) {
                    Ok(new) => Outcome::Changes(changes(&text, &new)),
                    Err(r) => Outcome::Refused(r),
                };
                let _ = tx.send(Done {
                    key: mine,
                    ticket,
                    outcome,
                });
            });
        self.in_flight += 1;
        if let Err(e) = spawned {
            let _ = self.tx.send(Done {
                key,
                ticket,
                outcome: Outcome::Refused(Refusal::Failed(e.to_string())),
            });
        }
    }

    /// The answers that arrived (never blocks).
    pub fn poll(&mut self) -> Vec<Done<K>> {
        let mut got = std::mem::take(&mut self.ready);
        got.extend(self.rx.try_iter());
        self.in_flight -= got.len();
        got
    }

    /// Waits until every request answered or `timeout` passed (tests and probes); the next
    /// [`Worker::poll`] hands the answers out.
    pub fn wait(&mut self, timeout: Duration) {
        let end = Instant::now() + timeout;
        while self.in_flight > self.ready.len() {
            let left = end.saturating_duration_since(Instant::now());
            match self.rx.recv_timeout(left) {
                Ok(d) => self.ready.push(d),
                Err(_) => break,
            }
        }
    }

    pub fn in_flight(&self) -> usize {
        self.in_flight
    }
}
