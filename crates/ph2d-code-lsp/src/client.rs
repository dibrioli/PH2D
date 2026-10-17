//! A small LSP client for `luau-lsp` over stdio: one reader thread demultiplexes responses (by id)
//! from notifications; server requests (`workspace/configuration`, …) are answered on that thread.
//! [`LspClient::request`] blocks; [`LspClient::request_async`] hands the answer to a callback on the
//! reader thread (the editor's worker never waits). A dead child surfaces as
//! [`LspError::ServerDied`] and [`LspClient::restart`] brings it back with the open documents re-sent.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::framing::write_message;

mod hooks;
mod json;
mod reader;

use crate::position::{Position, Range, position_to_byte};
pub use hooks::{hooks_in_definitions, is_unused_engine_hook};
use json::{hover_text, initialize_params, range_json, text_position};
use reader::reader_loop;

/// The luau-lsp lint `FunctionUnused` (its number in `code`).
const LINT_FUNCTION_UNUSED: &str = "8";

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
    /// `--docs=<json>`: the documentation base (signature and parameter docs). Only the command line
    /// reaches the server — `types.documentationFiles` is ignored with a null `rootUri` (I0b §3).
    pub docs: Option<PathBuf>,
    pub request_timeout: Duration,
}

impl ServerConfig {
    pub fn new(binary: PathBuf) -> Self {
        Self {
            binary,
            definitions: Vec::new(),
            solver: Solver::Old,
            base_luaurc: None,
            docs: None,
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
        if let Some(docs) = &self.docs {
            a.push(format!("--docs={}", docs.display()));
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
    ServerDied {
        status: Option<ExitStatus>,
        stderr_tail: String,
    },
    Timeout {
        method: String,
    },
    Server {
        method: String,
        code: i64,
        message: String,
    },
    Protocol(String),
}

impl fmt::Display for LspError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn(e) => write!(f, "could not start luau-lsp: {e}"),
            Self::ServerDied {
                status,
                stderr_tail,
            } => {
                write!(f, "luau-lsp died ({status:?}); last stderr: {stderr_tail}")
            }
            Self::Timeout { method } => write!(f, "luau-lsp did not answer {method} in time"),
            Self::Server {
                method,
                code,
                message,
            } => {
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
    /// The Luau code as text: `"1002"` (type error), `"8"` (lint), `"1014"` (syntax error — the
    /// message starts with `SyntaxError:`; measured, `docs/CodeEditor/03_onda2.md` I0c). luau-lsp
    /// sends numbers; strings are kept as they come.
    pub code: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PublishedDiagnostics {
    pub uri: String,
    /// The document version these diagnostics are for (luau-lsp 1.70.1 echoes the `didOpen` /
    /// `didChange` version — measured 16/10); `None` when the server omits it.
    pub version: Option<i64>,
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
    Other {
        method: String,
        params: Value,
    },
    /// The reader hit end of stream: the server is gone.
    Exited,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionItem {
    pub label: String,
    pub kind: Option<u32>,
    pub detail: Option<String>,
}

/// Who receives one response: a channel (blocking `request`) or a callback (`request_async`). It
/// runs ON THE READER THREAD, so it must only hand the value on.
type Waiter = Box<dyn FnOnce(Result<Value, LspError>) + Send>;
type Pending = Arc<Mutex<BTreeMap<i64, Waiter>>>;
type Writer = Arc<Mutex<Box<dyn Write + Send>>>;

/// Called on the reader thread after every notification is queued (and when the server goes away),
/// so a worker blocked on its own inbox learns there is something to [`LspClient::pump`].
pub type Wake = Arc<dyn Fn() + Send + Sync>;

const STDERR_TAIL: usize = 200;

pub struct LspClient {
    config: ServerConfig,
    /// `None` when connected to an in-process server ([`LspClient::connect`]).
    child: Option<Child>,
    stdin: Writer,
    pending: Pending,
    wake: Option<Wake>,
    alive: Arc<AtomicBool>,
    notifications: Receiver<Notification>,
    stderr_tail: Arc<Mutex<VecDeque<String>>>,
    next_id: i64,
    documents: BTreeMap<String, (i32, String)>,
    diagnostics: BTreeMap<String, PublishedDiagnostics>,
    messages: Vec<ServerMessage>,
    server_info: Value,
    engine_hooks: BTreeSet<String>,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|p| p.into_inner())
}

impl LspClient {
    /// Spawns the server and completes `initialize` / `initialized`.
    pub fn start(config: ServerConfig) -> Result<Self, LspError> {
        Self::start_with_wake(config, None)
    }

    /// [`Self::start`], with `wake` called whenever a notification arrives.
    pub fn start_with_wake(config: ServerConfig, wake: Option<Wake>) -> Result<Self, LspError> {
        let mut child = Command::new(&config.binary)
            .args(config.args())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(LspError::Spawn)?;
        let stdin = child.stdin.take().expect("piped stdin");
        let stdout = child.stdout.take().expect("piped stdout");
        let stderr = child.stderr.take().expect("piped stderr");
        let stderr_tail: Arc<Mutex<VecDeque<String>>> = Arc::default();
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
        Self::over(
            config,
            Box::new(stdin),
            Box::new(stdout),
            Some(child),
            stderr_tail,
            wake,
        )
    }

    /// Talks LSP over an arbitrary pipe pair instead of the pinned binary — an in-process server
    /// (the slow/dying fakes of the editor's tests). [`Self::restart`] is refused: there is no
    /// process to start again.
    pub fn connect(
        config: ServerConfig,
        to_server: Box<dyn Write + Send>,
        from_server: Box<dyn Read + Send>,
        wake: Option<Wake>,
    ) -> Result<Self, LspError> {
        Self::over(config, to_server, from_server, None, Arc::default(), wake)
    }

    fn over(
        config: ServerConfig,
        to_server: Box<dyn Write + Send>,
        from_server: Box<dyn Read + Send>,
        child: Option<Child>,
        stderr_tail: Arc<Mutex<VecDeque<String>>>,
        wake: Option<Wake>,
    ) -> Result<Self, LspError> {
        let stdin: Writer = Arc::new(Mutex::new(to_server));
        let pending: Pending = Arc::default();
        let alive = Arc::new(AtomicBool::new(true));
        let (tx, rx) = mpsc::channel();
        {
            let (stdin, pending, alive) = (stdin.clone(), pending.clone(), alive.clone());
            let wake = wake.clone();
            thread::Builder::new()
                .name("luau-lsp-reader".into())
                .spawn(move || {
                    reader_loop(
                        BufReader::new(from_server),
                        &stdin,
                        &pending,
                        &alive,
                        &tx,
                        wake.as_deref(),
                    );
                })
                .map_err(LspError::Spawn)?;
        }

        let mut engine_hooks = BTreeSet::new();
        for (_, path) in &config.definitions {
            let text = std::fs::read_to_string(path).unwrap_or_default();
            engine_hooks.extend(hooks_in_definitions(&text));
        }
        let mut client = Self {
            config,
            child,
            stdin,
            pending,
            wake,
            alive,
            notifications: rx,
            stderr_tail,
            next_id: 0,
            documents: BTreeMap::new(),
            diagnostics: BTreeMap::new(),
            messages: Vec::new(),
            server_info: Value::Null,
            engine_hooks,
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
        self.alive.load(Ordering::SeqCst)
            && self
                .child
                .as_mut()
                .is_none_or(|c| matches!(c.try_wait(), Ok(None)))
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
        let Some(child) = self.child.as_mut() else {
            return Err(LspError::Protocol(
                "an in-process server cannot be restarted".into(),
            ));
        };
        let _ = child.kill();
        let _ = child.wait();
        let mut fresh = Self::start_with_wake(self.config.clone(), self.wake.clone())?;
        for (uri, (version, text)) in &self.documents {
            fresh.did_open_at(uri, *version, text)?;
        }
        fresh.messages = std::mem::take(&mut self.messages);
        *self = fresh;
        Ok(())
    }

    fn died(&mut self) -> LspError {
        self.alive.store(false, Ordering::SeqCst);
        let status = self
            .child
            .as_mut()
            .and_then(|c| c.try_wait().ok().flatten());
        LspError::ServerDied {
            status,
            stderr_tail: self.stderr_tail().join("\n"),
        }
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

    /// Sends a request; `done` receives its response ON THE READER THREAD (or `ServerDied` if the
    /// server goes away first). Returns the id, for [`Self::cancel`].
    pub fn request_async(
        &mut self,
        method: &str,
        params: Value,
        done: impl FnOnce(Result<Value, LspError>) + Send + 'static,
    ) -> Result<i64, LspError> {
        self.next_id += 1;
        let id = self.next_id;
        lock(&self.pending).insert(id, Box::new(done));
        let body = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        if let Err(e) = self.send(&body) {
            lock(&self.pending).remove(&id);
            return Err(e);
        }
        Ok(id)
    }

    /// Forgets a pending request (its callback never runs) and tells the server (`$/cancelRequest`).
    pub fn cancel(&mut self, id: i64) -> Result<(), LspError> {
        if lock(&self.pending).remove(&id).is_none() {
            return Ok(());
        }
        self.notify("$/cancelRequest", json!({ "id": id }))
    }

    /// Sends a request and blocks until its response (or the timeout / the server's death).
    pub fn request(&mut self, method: &str, params: Value) -> Result<Value, LspError> {
        let (tx, rx) = mpsc::channel();
        let id = self.request_async(method, params, move |r| {
            let _ = tx.send(r);
        })?;
        match rx.recv_timeout(self.config.request_timeout) {
            Ok(Ok(v)) => Ok(v),
            Ok(Err(LspError::Server { code, message, .. })) => Err(LspError::Server {
                method: method.to_owned(),
                code,
                message,
            }),
            Ok(Err(LspError::ServerDied { .. })) | Err(RecvTimeoutError::Disconnected) => {
                Err(self.died())
            }
            Ok(Err(e)) => Err(e),
            Err(RecvTimeoutError::Timeout) => {
                lock(&self.pending).remove(&id);
                Err(LspError::Timeout {
                    method: method.to_owned(),
                })
            }
        }
    }

    pub fn did_open(&mut self, uri: &str, text: &str) -> Result<(), LspError> {
        self.did_open_at(uri, 1, text)
    }

    /// `didOpen` at an explicit `version` (the editor uses its buffer revision: the server echoes
    /// it in `publishDiagnostics`, measured with luau-lsp 1.70.1).
    pub fn did_open_at(&mut self, uri: &str, version: i32, text: &str) -> Result<(), LspError> {
        self.documents
            .insert(uri.to_owned(), (version, text.to_owned()));
        self.notify(
            "textDocument/didOpen",
            json!({ "textDocument": { "uri": uri, "languageId": "luau", "version": version, "text": text } }),
        )
    }

    /// Replaces `range` with `new_text` and sets the document to `version` (which must grow).
    pub fn did_change_at(
        &mut self,
        uri: &str,
        version: i32,
        range: Range,
        new_text: &str,
    ) -> Result<(), LspError> {
        let (v, text) = self
            .documents
            .get_mut(uri)
            .ok_or_else(|| LspError::Protocol(format!("didChange on unopened {uri}")))?;
        let a = position_to_byte(text, range.start);
        let b = position_to_byte(text, range.end).max(a);
        text.replace_range(a..b, new_text);
        *v = version;
        self.notify(
            "textDocument/didChange",
            json!({ "textDocument": { "uri": uri, "version": version },
                    "contentChanges": [{ "range": range_json(range), "text": new_text }] }),
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
        self.notify(
            "textDocument/didClose",
            json!({ "textDocument": { "uri": uri } }),
        )
    }

    /// The client's copy of an open document.
    pub fn text(&self, uri: &str) -> Option<&str> {
        self.documents.get(uri).map(|(_, t)| t.as_str())
    }

    pub fn completion(&mut self, uri: &str, at: Position) -> Result<Vec<CompletionItem>, LspError> {
        let r = self.request("textDocument/completion", text_position(uri, at))?;
        let items = match &r {
            Value::Array(a) => a.as_slice(),
            Value::Object(o) => o
                .get("items")
                .and_then(Value::as_array)
                .map_or(&[][..], Vec::as_slice),
            _ => &[],
        };
        Ok(items
            .iter()
            .map(|i| CompletionItem {
                label: i
                    .get("label")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
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
            out.push(self.take(n));
        }
        out
    }

    /// Blocks until a NEW `publishDiagnostics` for `uri` arrives (call [`Self::pump`] before the
    /// edit so an older one is not taken for it).
    pub fn wait_diagnostics(
        &mut self,
        uri: &str,
        timeout: Duration,
    ) -> Result<PublishedDiagnostics, LspError> {
        let deadline = Instant::now() + timeout;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            match self.notifications.recv_timeout(left) {
                Ok(n) => match self.take(n) {
                    Notification::Diagnostics(d) if d.uri == uri => return Ok(d),
                    Notification::Exited => return Err(self.died()),
                    _ => {}
                },
                Err(RecvTimeoutError::Timeout) => {
                    return Err(LspError::Timeout {
                        method: format!("publishDiagnostics {uri}"),
                    });
                }
                Err(RecvTimeoutError::Disconnected) => return Err(self.died()),
            }
        }
    }

    /// Filters (engine hooks) and records one notification.
    fn take(&mut self, mut n: Notification) -> Notification {
        match &mut n {
            Notification::Diagnostics(d) => {
                if let Some((_, text)) = self.documents.get(&d.uri) {
                    d.diagnostics
                        .retain(|x| !is_unused_engine_hook(x, text, &self.engine_hooks));
                }
                self.diagnostics.insert(d.uri.clone(), d.clone());
            }
            Notification::Message(m) => self.messages.push(m.clone()),
            Notification::Exited => self.alive.store(false, Ordering::SeqCst),
            Notification::Other { .. } => {}
        }
        n
    }

    /// The engine hooks read from the definitions files (`-- @hook <name>:` lines).
    pub fn engine_hooks(&self) -> &BTreeSet<String> {
        &self.engine_hooks
    }

    /// `shutdown` + `exit`, then waits briefly for the process (kills it if it lingers).
    pub fn shutdown(mut self) -> Result<(), LspError> {
        let r = self.request("shutdown", Value::Null).map(drop);
        let _ = self.notify("exit", Value::Null);
        let deadline = Instant::now() + Duration::from_secs(2);
        let Some(child) = self.child.as_mut() else {
            return r;
        };
        while Instant::now() < deadline {
            if matches!(child.try_wait(), Ok(Some(_))) {
                return r;
            }
            thread::sleep(Duration::from_millis(10));
        }
        let _ = child.kill();
        let _ = child.wait();
        r
    }

    /// The child's pid (for RSS sampling in probes); `0` for an in-process server.
    pub fn pid(&self) -> u32 {
        self.child.as_ref().map_or(0, Child::id)
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        if let Some(c) = self.child.as_mut()
            && matches!(c.try_wait(), Ok(None))
        {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}
