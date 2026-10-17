//! The reader thread: answers the server's own requests, hands each response to its waiter and
//! queues notifications (moved out of `client.rs` by the workspace LOC cap).

use super::json::{answer_server_request, parse_notification};
use super::{LspError, Notification, Pending, lock};
use crate::framing::{read_message, write_message};
use serde_json::{Value, json};
use std::io::{BufRead, Write};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;

pub(super) fn reader_loop(
    mut stdout: impl BufRead,
    stdin: &Mutex<Box<dyn Write + Send>>,
    pending: &Pending,
    alive: &AtomicBool,
    tx: &Sender<Notification>,
    wake: Option<&(dyn Fn() + Send + Sync)>,
) {
    let wake = || {
        if let Some(w) = wake {
            w();
        }
    };
    while let Ok(Some(msg)) = read_message(&mut stdout) {
        let method = msg.get("method").and_then(Value::as_str);
        match (msg.get("id"), method) {
            (Some(id), Some(method)) => {
                let result = answer_server_request(method, msg.get("params"));
                let reply = json!({ "jsonrpc": "2.0", "id": id, "result": result });
                if write_message(&mut *lock(stdin), &reply).is_err() {
                    break;
                }
            }
            (Some(id), None) => {
                let Some(id) = id.as_i64() else { continue };
                let Some(waiter) = lock(pending).remove(&id) else {
                    continue;
                };
                let r = match msg.get("error") {
                    Some(e) => Err(LspError::Server {
                        method: String::new(),
                        code: e.get("code").and_then(Value::as_i64).unwrap_or(0),
                        message: e
                            .get("message")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_owned(),
                    }),
                    None => Ok(msg.get("result").cloned().unwrap_or(Value::Null)),
                };
                waiter(r);
            }
            (None, Some(method)) => {
                let _ = tx.send(parse_notification(
                    method,
                    msg.get("params").cloned().unwrap_or(Value::Null),
                ));
                wake();
            }
            (None, None) => {}
        }
    }
    alive.store(false, Ordering::SeqCst);
    for (_, waiter) in std::mem::take(&mut *lock(pending)) {
        waiter(Err(LspError::ServerDied {
            status: None,
            stderr_tail: String::new(),
        }));
    }
    let _ = tx.send(Notification::Exited);
    wake();
}
