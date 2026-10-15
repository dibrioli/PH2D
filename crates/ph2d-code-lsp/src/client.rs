//! A small, synchronous LSP client for `luau-lsp` over stdio: one reader thread demultiplexes
//! responses (by id) from notifications; server requests (`workspace/configuration`, …) are answered
//! on that thread. A dead child surfaces as [`LspError::ServerDied`] and [`LspClient::restart`]
//! brings it back with the open documents re-sent.

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fmt;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::framing::{read_message, write_message};
use crate::position::{Position, Range, position_to_byte};

/// Which Luau type solver the server runs. luau-lsp 1.70.1 defaults to the old one; the new one is
/// switched on by the `LuauSolverV2` FFlag (the `fflags.enableNewSolver` setting answered through
/// `workspace/configuration` was measured to change nothing).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Solver {
    Old,
    New,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub binary: PathBuf,
    /// `(name, path)` pairs → `--definitions:<name>=<path>`; PH2D uses `@ph2d`.
    pub definitions: Vec<(String, PathBuf)>,
    pub solver: Solver,
    /// `--base-luaurc` (lint/mode defaults for every file).
    pub base_luaurc: Option<PathBuf>,
    pub request_timeout: Duration,
}

impl ServerConfig {
    pub fn new(binary: PathBuf) -> Self {
        Self {
            binary,
            definitions: Vec::new(),
            solver: Solver::Old,
            base_luaurc: None,
            request_timeout: Duration::from_secs(30),
        }
    }

    pub fn with_definitions(mut self, name: &str, path: PathBuf) -> Self {
        self.definitions.push((name.to_owned(), path));
        self
    }

    fn args(&self) -> Vec<String> {
        let mut a = vec!["lsp".to_owned(), "--stdio".to_owned()];
        for (name, path) in &self.definitions {
            a.push(format!("--definitions:{name}={}", path.display()));
        }
        if let Some(rc) = &self.base_luaurc {
            a.push(format!("--base-luaurc={}", rc.display()));
        }
        if self.solver == Solver::New {
            a.push("--flag:LuauSolverV2=true".to_owned());
        }
        a
    }
}

/// The settings PH2D answers to `workspace/configuration` (section `luau-lsp`): a standard Luau
/// host, and no fetching of Roblox's FFlags (reproducible across machines).
pub fn ph2d_settings() -> Value {
    json!({ "platform": { "type": "standard" }, "fflags": { "sync": false } })
}

#[derive(Debug)]
pub enum LspError {
    Spawn(io::Error),
    ServerDied { status: Option<ExitStatus>, stderr_tail: String },
    Timeout { method: String },
    Server { method: String, code: i64, message: String },
    Protocol(String),
}

impl fmt::Display for LspError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn(e) => write!(f, "could not start luau-lsp: {e}"),
            Self::ServerDied { status, stderr_tail } => {
                write!(f, "luau-lsp died ({status:?}); last stderr: {stderr_tail}")
            }
            Self::Timeout { method } => write!(f, "luau-lsp did not answer {method} in time"),
            Self::Server { method, code, message } => {
                write!(f, "luau-lsp refused {method}: {code} {message}")
            }
            Self::Protocol(m) => write!(f, "luau-lsp protocol error: {m}"),
        }
    }
}

impl std::error::Error for LspError {}

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub range: Range,
    /// 1 error · 2 warning · 3 information · 4 hint.
    pub severity: Option<u8>,
    /// `"1002"` (type error code), `"8"` (lint code) or `"SyntaxError"` — luau-lsp mixes numbers
    /// and strings, so both are kept as text.
    pub code: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PublishedDiagnostics {
    pub uri: String,
    pub diagnostics: Vec<Diagnostic>,
    pub received: Instant,
}

/// A `window/logMessage` (`shown == false`) or `window/showMessage` (`shown == true`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerMessage {
    pub shown: bool,
    /// 1 error · 2 warning · 3 info · 4 log.
    pub level: u8,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Notification {
    Diagnostics(PublishedDiagnostics),
    Message(ServerMessage),
    Other { method: String, params: Value },
    /// The reader hit end of stream: the server is gone.
    Exited,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionItem {
    pub label: String,
    pub kind: Option<u32>,
    pub detail: Option<String>,
}

type Pending = Arc<Mutex<HashMap<i64, Sender<Result<Value, LspError>>>>>;

const STDERR_TAIL: usize = 200;

pub struct LspClient {
    config: ServerConfig,
    child: Child,
    stdin: Arc<Mutex<ChildStdin>>,
    pending: Pending,
    alive: Arc<AtomicBool>,
    notifications: Receiver<Notification>,
    stderr_tail: Arc<Mutex<VecDeque<String>>>,
    next_id: i64,
    documents: BTreeMap<String, (i32, String)>,
    diagnostics: BTreeMap<String, PublishedDiagnostics>,
    messages: Vec<ServerMessage>,
    server_info: Value,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|p| p.into_inner())
}

impl LspClient {
    /// Spawns the server and completes `initialize` / `initialized`.
    pub fn start(config: ServerConfig) -> Result<Self, LspError> {
        let mut child = Command::new(&config.binary)
            .args(config.args())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(LspError::Spawn)?;
        let stdin = Arc::new(Mutex::new(child.stdin.take().expect("piped stdin")));
        let stdout = child.stdout.take().expect("piped stdout");
        let stderr = child.stderr.take().expect("piped stderr");
        let pending: Pending = Arc::default();
        let alive = Arc::new(AtomicBool::new(true));
        let stderr_tail: Arc<Mutex<VecDeque<String>>> = Arc::default();
        let (tx, rx) = mpsc::channel();

        {
            let tail = stderr_tail.clone();
            thread::Builder::new()
                .name("luau-lsp-stderr".into())
                .spawn(move || {
                    for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                        let mut t = lock(&tail);
                        if t.len() == STDERR_TAIL {
                            t.pop_front();
                        }
                        t.push_back(line);
                    }
                })
                .map_err(LspError::Spawn)?;
        }
        {
            let (stdin, pending, alive) = (stdin.clone(), pending.clone(), alive.clone());
            thread::Builder::new()
                .name("luau-lsp-reader".into())
                .spawn(move || reader_loop(BufReader::new(stdout), &stdin, &pending, &alive, &tx))
                .map_err(LspError::Spawn)?;
        }

        let mut client = Self {
            config,
            child,
            stdin,
            pending,
            alive,
            notifications: rx,
            stderr_tail,
            next_id: 0,
            documents: BTreeMap::new(),
            diagnostics: BTreeMap::new(),
            messages: Vec::new(),
            server_info: Value::Null,
        };
        let result = client.request("initialize", initialize_params())?;
        client.server_info = result.get("serverInfo").cloned().unwrap_or(Value::Null);
        client.notify("initialized", json!({}))?;
        Ok(client)
    }

    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    /// `serverInfo` from `initialize` (name + version).
    pub fn server_info(&self) -> &Value {
        &self.server_info
    }

    /// False once the server process ended (or its stdout closed).
    pub fn is_alive(&mut self) -> bool {
        self.alive.load(Ordering::SeqCst) && matches!(self.child.try_wait(), Ok(None))
    }

    /// The last lines the server wrote to stderr.
    pub fn stderr_tail(&self) -> Vec<String> {
        lock(&self.stderr_tail).iter().cloned().collect()
    }

    /// Every `window/logMessage` / `window/showMessage` pumped so far.
    pub fn messages(&self) -> &[ServerMessage] {
        &self.messages
    }

    /// The latest diagnostics published for `uri`, if any arrived.
    pub fn diagnostics(&self, uri: &str) -> Option<&PublishedDiagnostics> {
        self.diagnostics.get(uri)
    }

    /// The latest diagnostics of every uri the server published for (definitions files included).
    pub fn all_diagnostics(&self) -> impl Iterator<Item = &PublishedDiagnostics> {
        self.diagnostics.values()
    }

    /// Kills the server (if still running) and starts a new one with the same config, re-opening
    /// every tracked document at its current text.
    pub fn restart(&mut self) -> Result<(), LspError> {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let mut fresh = Self::start(self.config.clone())?;
        for (uri, (_, text)) in &self.documents {
            fresh.did_open(uri, text)?;
        }
        fresh.messages = std::mem::take(&mut self.messages);
        *self = fresh;
        Ok(())
    }

    fn died(&mut self) -> LspError {
        self.alive.store(false, Ordering::SeqCst);
        let status = self.child.try_wait().ok().flatten();
        LspError::ServerDied { status, stderr_tail: self.stderr_tail().join("\n") }
    }

    fn send(&mut self, body: &Value) -> Result<(), LspError> {
        if !self.alive.load(Ordering::SeqCst) {
            return Err(self.died());
        }
        let r = write_message(&mut *lock(&self.stdin), body);
        r.map_err(|_| self.died())
    }

    pub fn notify(&mut self, method: &str, params: Value) -> Result<(), LspError> {
        self.send(&json!({ "jsonrpc": "2.0", "method": method, "params": params }))
    }

    /// Sends a request and blocks until its response (or the timeout / the server's death).
    pub fn request(&mut self, method: &str, params: Value) -> Result<Value, LspError> {
        self.next_id += 1;
        let id = self.next_id;
        let (tx, rx) = mpsc::channel();
        lock(&self.pending).insert(id, tx);
        let body = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        if let Err(e) = self.send(&body) {
            lock(&self.pending).remove(&id);
            return Err(e);
        }
        match rx.recv_timeout(self.config.request_timeout) {
            Ok(Ok(v)) => Ok(v),
            Ok(Err(LspError::Server { code, message, .. })) => {
                Err(LspError::Server { method: method.to_owned(), code, message })
            }
            Ok(Err(LspError::ServerDied { .. })) | Err(RecvTimeoutError::Disconnected) => {
                Err(self.died())
            }
            Ok(Err(e)) => Err(e),
            Err(RecvTimeoutError::Timeout) => {
                lock(&self.pending).remove(&id);
                Err(LspError::Timeout { method: method.to_owned() })
            }
        }
    }

    pub fn did_open(&mut self, uri: &str, text: &str) -> Result<(), LspError> {
        self.documents.insert(uri.to_owned(), (1, text.to_owned()));
        self.notify(
            "textDocument/didOpen",
            json!({ "textDocument": { "uri": uri, "languageId": "luau", "version": 1, "text": text } }),
        )
    }

    /// Replaces the whole text.
    pub fn did_change_full(&mut self, uri: &str, text: &str) -> Result<(), LspError> {
        let version = self.bump(uri, |t| *t = text.to_owned())?;
        self.notify(
            "textDocument/didChange",
            json!({ "textDocument": { "uri": uri, "version": version },
                    "contentChanges": [{ "text": text }] }),
        )
    }

    /// Replaces `range` with `new_text` (incremental sync); the tracked copy is edited the same way.
    pub fn did_change(&mut self, uri: &str, range: Range, new_text: &str) -> Result<(), LspError> {
        let version = self.bump(uri, |t| {
            let a = position_to_byte(t, range.start);
            let b = position_to_byte(t, range.end).max(a);
            t.replace_range(a..b, new_text);
        })?;
        self.notify(
            "textDocument/didChange",
            json!({ "textDocument": { "uri": uri, "version": version },
                    "contentChanges": [{ "range": range_json(range), "text": new_text }] }),
        )
    }

    fn bump(&mut self, uri: &str, edit: impl FnOnce(&mut String)) -> Result<i32, LspError> {
        let (v, text) = self
            .documents
            .get_mut(uri)
            .ok_or_else(|| LspError::Protocol(format!("didChange on unopened {uri}")))?;
        *v += 1;
        edit(text);
        Ok(*v)
    }

    pub fn did_close(&mut self, uri: &str) -> Result<(), LspError> {
        self.documents.remove(uri);
        self.notify("textDocument/didClose", json!({ "textDocument": { "uri": uri } }))
    }

    /// The client's copy of an open document.
    pub fn text(&self, uri: &str) -> Option<&str> {
        self.documents.get(uri).map(|(_, t)| t.as_str())
    }

    pub fn completion(&mut self, uri: &str, at: Position) -> Result<Vec<CompletionItem>, LspError> {
        let r = self.request("textDocument/completion", text_position(uri, at))?;
        let items = match &r {
            Value::Array(a) => a.as_slice(),
            Value::Object(o) => o.get("items").and_then(Value::as_array).map_or(&[][..], Vec::as_slice),
            _ => &[],
        };
        Ok(items
            .iter()
            .map(|i| CompletionItem {
                label: i.get("label").and_then(Value::as_str).unwrap_or_default().to_owned(),
                kind: i.get("kind").and_then(Value::as_u64).map(|k| k as u32),
                detail: i.get("detail").and_then(Value::as_str).map(str::to_owned),
            })
            .collect())
    }

    /// The hover text (markdown or plain), `None` when the server has nothing there.
    pub fn hover(&mut self, uri: &str, at: Position) -> Result<Option<String>, LspError> {
        let r = self.request("textDocument/hover", text_position(uri, at))?;
        Ok(r.get("contents").map(hover_text).filter(|s| !s.is_empty()))
    }

    /// Pumps notifications without blocking.
    pub fn pump(&mut self) -> Vec<Notification> {
        let mut out = Vec::new();
        while let Ok(n) = self.notifications.try_recv() {
            self.record(&n);
            out.push(n);
        }
        out
    }

    /// Blocks until a NEW `publishDiagnostics` for `uri` arrives (call [`Self::pump`] before the
    /// edit so an older one is not taken for it).
    pub fn wait_diagnostics(&mut self, uri: &str, timeout: Duration) -> Result<PublishedDiagnostics, LspError> {
        let deadline = Instant::now() + timeout;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            match self.notifications.recv_timeout(left) {
                Ok(n) => {
                    self.record(&n);
                    match n {
                        Notification::Diagnostics(d) if d.uri == uri => return Ok(d),
                        Notification::Exited => return Err(self.died()),
                        _ => {}
                    }
                }
                Err(RecvTimeoutError::Timeout) => {
                    return Err(LspError::Timeout { method: format!("publishDiagnostics {uri}") });
                }
                Err(RecvTimeoutError::Disconnected) => return Err(self.died()),
            }
        }
    }

    fn record(&mut self, n: &Notification) {
        match n {
            Notification::Diagnostics(d) => {
                self.diagnostics.insert(d.uri.clone(), d.clone());
            }
            Notification::Message(m) => self.messages.push(m.clone()),
            Notification::Exited => self.alive.store(false, Ordering::SeqCst),
            Notification::Other { .. } => {}
        }
    }

    /// `shutdown` + `exit`, then waits briefly for the process (kills it if it lingers).
    pub fn shutdown(mut self) -> Result<(), LspError> {
        let r = self.request("shutdown", Value::Null).map(drop);
        let _ = self.notify("exit", Value::Null);
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if matches!(self.child.try_wait(), Ok(Some(_))) {
                return r;
            }
            thread::sleep(Duration::from_millis(10));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
        r
    }

    /// The child's pid (for RSS sampling in probes).
    pub fn pid(&self) -> u32 {
        self.child.id()
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

fn reader_loop(
    mut stdout: impl BufRead,
    stdin: &Mutex<ChildStdin>,
    pending: &Pending,
    alive: &AtomicBool,
    tx: &Sender<Notification>,
) {
    loop {
        let msg = match read_message(&mut stdout) {
            Ok(Some(m)) => m,
            Ok(None) | Err(_) => break,
        };
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
                let Some(waiter) = lock(pending).remove(&id) else { continue };
                let r = match msg.get("error") {
                    Some(e) => Err(LspError::Server {
                        method: String::new(),
                        code: e.get("code").and_then(Value::as_i64).unwrap_or(0),
                        message: e.get("message").and_then(Value::as_str).unwrap_or_default().to_owned(),
                    }),
                    None => Ok(msg.get("result").cloned().unwrap_or(Value::Null)),
                };
                let _ = waiter.send(r);
            }
            (None, Some(method)) => {
                let _ = tx.send(parse_notification(method, msg.get("params").cloned().unwrap_or(Value::Null)));
            }
            (None, None) => {}
        }
    }
    alive.store(false, Ordering::SeqCst);
    for (_, waiter) in lock(pending).drain() {
        let _ = waiter.send(Err(LspError::ServerDied { status: None, stderr_tail: String::new() }));
    }
    let _ = tx.send(Notification::Exited);
}

fn answer_server_request(method: &str, params: Option<&Value>) -> Value {
    match method {
        "workspace/configuration" => {
            let items = params.and_then(|p| p.get("items")).and_then(Value::as_array);
            Value::Array(
                items
                    .into_iter()
                    .flatten()
                    .map(|it| match it.get("section").and_then(Value::as_str) {
                        Some("luau-lsp") => ph2d_settings(),
                        _ => Value::Null,
                    })
                    .collect(),
            )
        }
        _ => Value::Null,
    }
}

fn parse_notification(method: &str, params: Value) -> Notification {
    match method {
        "textDocument/publishDiagnostics" => Notification::Diagnostics(PublishedDiagnostics {
            uri: params.get("uri").and_then(Value::as_str).unwrap_or_default().to_owned(),
            diagnostics: params
                .get("diagnostics")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .map(parse_diagnostic)
                .collect(),
            received: Instant::now(),
        }),
        "window/logMessage" | "window/showMessage" => Notification::Message(ServerMessage {
            shown: method == "window/showMessage",
            level: params.get("type").and_then(Value::as_u64).unwrap_or(4) as u8,
            text: params.get("message").and_then(Value::as_str).unwrap_or_default().to_owned(),
        }),
        _ => Notification::Other { method: method.to_owned(), params },
    }
}

fn parse_diagnostic(d: &Value) -> Diagnostic {
    Diagnostic {
        range: d.get("range").map(parse_range).unwrap_or_default(),
        severity: d.get("severity").and_then(Value::as_u64).map(|s| s as u8),
        code: d.get("code").and_then(|c| match c {
            Value::String(s) => Some(s.clone()),
            Value::Number(n) => Some(n.to_string()),
            _ => None,
        }),
        message: d.get("message").and_then(Value::as_str).unwrap_or_default().to_owned(),
    }
}

fn parse_position(p: &Value) -> Position {
    let n = |k| p.get(k).and_then(Value::as_u64).unwrap_or(0) as u32;
    Position::new(n("line"), n("character"))
}

fn parse_range(r: &Value) -> Range {
    Range {
        start: r.get("start").map(parse_position).unwrap_or_default(),
        end: r.get("end").map(parse_position).unwrap_or_default(),
    }
}

fn range_json(r: Range) -> Value {
    json!({ "start": { "line": r.start.line, "character": r.start.character },
            "end": { "line": r.end.line, "character": r.end.character } })
}

fn text_position(uri: &str, at: Position) -> Value {
    json!({ "textDocument": { "uri": uri }, "position": { "line": at.line, "character": at.character } })
}

fn hover_text(contents: &Value) -> String {
    match contents {
        Value::String(s) => s.clone(),
        Value::Object(o) => o.get("value").and_then(Value::as_str).unwrap_or_default().to_owned(),
        Value::Array(a) => a.iter().map(hover_text).collect::<Vec<_>>().join("\n"),
        _ => String::new(),
    }
}

fn initialize_params() -> Value {
    json!({
        "processId": std::process::id(),
        "rootUri": null,
        "capabilities": {
            "general": { "positionEncodings": ["utf-16"] },
            "workspace": { "configuration": true },
            "textDocument": {
                "synchronization": { "didSave": false },
                "publishDiagnostics": {},
                "completion": { "completionItem": { "snippetSupport": false } },
                "hover": { "contentFormat": ["markdown", "plaintext"] }
            }
        },
        "initializationOptions": {}
    })
}
