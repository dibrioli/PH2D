//! LLM-driven vector authoring in the live editor (Inovação P4, ADR-0061).
//!
//! Turns the shipped `ph2d-vector-llm-client` + sanitizer into a **live** editor
//! feature: a natural-language prompt becomes an **editable** [`VectorNetwork`]
//! committed to the document (`App::committed_vector_pen_paths`), undoable with
//! one Ctrl+Z, exactly like a Pen/Pencil/Shape commit.
//!
//! ## Off the UI thread (the one real constraint)
//!
//! The shell's frame loop is synchronous; the LLM call blocks up to
//! [`LLM_TIMEOUT_SECS`](ph2d_vector_llm_client::LLM_TIMEOUT_SECS). So
//! [`LlmVectorEngine::submit`] runs the blocking client on a **worker thread**
//! and hands the result back over a channel; [`App::poll_llm_vector`] drains it
//! once per frame (the loop already runs at `ControlFlow::Poll`, so there is
//! nothing to wake). At most one generation is in flight at a time, so holding
//! the cache lock across the call is contention-free.
//!
//! ## Security is inherited
//!
//! The worker calls `generate_shape`, whose `build_from_json` runs the
//! bounds-before-allocation sanitizer — an out-of-bounds blob is rejected and
//! surfaced as a toast, never materialised. A transport failure (timeout /
//! network) degrades gracefully to the cached shape for that `(prompt, seed)`.

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::{Arc, Mutex};

use ph2d_editor::toast::Toast;
use ph2d_vector::{Ph2dVectorAsset, StyleTable, VectorNetwork};
use ph2d_vector_llm::ResultCache;
use ph2d_vector_llm_client::{AnthropicTransport, LlmClient};

use crate::App;

/// Fallback-cache capacity (entries keyed by `(prompt, seed)`). A handful is
/// plenty for interactive authoring; eviction is deterministic.
const CACHE_CAP: usize = 32;

/// What a finished background generation reports back to the UI thread.
pub(crate) enum LlmJobOutcome {
    /// A validated, editable network ready to commit, plus the **raw LLM4SVG
    /// blob the model returned** (so the UI can show what the model decided);
    /// the blob is `None` on a cache-fallback (no live response). Boxed because a
    /// `VectorNetwork` (inline SmallVec budgets) dwarfs the other variants.
    Ready(Box<VectorNetwork>, Option<String>),
    /// The fetch failed with no cached fallback, or the blob was rejected by the
    /// sanitizer. Carries a human-readable reason for the toast.
    Failed(String),
    /// No API key was found (neither env var nor config file) at request time.
    NoKey,
}

/// The editor's LLM-vector subsystem: at most one in-flight background
/// generation + a persistent fallback cache. The API key is resolved **lazily**
/// per request (env var or config file — see [`resolve_api_key`]), so a key
/// dropped in while the editor is running is picked up on the next generation.
/// Lives on [`App`]; constructed empty.
pub(crate) struct LlmVectorEngine {
    cache: Arc<Mutex<ResultCache>>,
    pending: Option<Receiver<LlmJobOutcome>>,
    /// Monotonic per-generation seed (also the cache key axis).
    next_seed: u64,
}

impl LlmVectorEngine {
    pub(crate) fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(ResultCache::new(CACHE_CAP))),
            pending: None,
            next_seed: 0,
        }
    }

    /// Kick off a background generation for `prompt`. Returns `false` (no-op) if
    /// a generation is already in flight. Resolves the API key first (env or
    /// config file); with no key it reports [`LlmJobOutcome::NoKey`] without
    /// spawning a thread. Otherwise the blocking ≤15 s client runs on a worker
    /// thread and [`Self::poll`] collects the result.
    pub(crate) fn submit(&mut self, prompt: String) -> bool {
        if self.pending.is_some() {
            return false;
        }
        let seed = self.next_seed;
        self.next_seed += 1;
        let (tx, rx) = std::sync::mpsc::channel();
        match resolve_api_key() {
            // No key → no network, no thread: report immediately for the toast.
            None => {
                let _ = tx.send(LlmJobOutcome::NoKey);
            }
            Some(key) => {
                // The worker captures only `Send` data (an `Arc` clone + owned
                // strings) — never `self` — so the thread is `'static`.
                let cache = Arc::clone(&self.cache);
                std::thread::spawn(move || {
                    let client = LlmClient::new(AnthropicTransport::with_key(key));
                    // Single in-flight job ⇒ holding the lock across the ≤15 s
                    // call is contention-free, and `generate_shape` needs
                    // `&mut cache` for both the fallback read and success write.
                    let outcome = match cache.lock() {
                        Ok(mut guard) => {
                            match client.generate_shape_with_blob(&prompt, seed, &mut guard) {
                                Ok((net, blob)) => LlmJobOutcome::Ready(Box::new(net), blob),
                                Err(e) => LlmJobOutcome::Failed(e.to_string()),
                            }
                        }
                        Err(_) => LlmJobOutcome::Failed("cache lock poisoned".to_string()),
                    };
                    let _ = tx.send(outcome);
                });
            }
        }
        self.pending = Some(rx);
        true
    }

    /// Non-blocking: collect a finished generation, if the worker has reported.
    /// Clears the in-flight slot on completion or disconnect.
    pub(crate) fn poll(&mut self) -> Option<LlmJobOutcome> {
        match self.pending.as_ref()?.try_recv() {
            Ok(outcome) => {
                self.pending = None;
                Some(outcome)
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.pending = None;
                None
            }
        }
    }
}

/// Resolve the Anthropic API key, lazily, at request time. Order:
/// 1. the `ANTHROPIC_API_KEY` environment variable (works when the editor was
///    launched from a shell that exported it);
/// 2. a key file — the means for a GUI launch that does **not** inherit the
///    shell env. Its path is [`api_key_file_path`]; the file holds just the key
///    (surrounding whitespace trimmed).
///
/// Never hardcoded, never logged. `None` if nothing yields a non-empty key.
fn resolve_api_key() -> Option<String> {
    if let Ok(k) = std::env::var("ANTHROPIC_API_KEY") {
        let k = k.trim().to_string();
        if !k.is_empty() {
            return Some(k);
        }
    }
    // Dev/test default (DEBUG BUILDS ONLY — never compiled into a release
    // binary): a key file at the repo root so the feature works without per-run
    // setup. `docs/_api-claude.md` is gitignored so the secret never enters the
    // repo; whatever is in it (key / pasted curl) is reduced to the sk-ant token.
    #[cfg(debug_assertions)]
    {
        if let Ok(raw) = std::fs::read_to_string("docs/_api-claude.md") {
            let k = extract_api_key(&raw);
            if !k.is_empty() {
                return Some(k);
            }
        }
    }
    let path = api_key_file_path()?;
    let k = std::fs::read_to_string(&path).ok()?.trim().to_string();
    (!k.is_empty()).then_some(k)
}

/// Where [`resolve_api_key`] looks for a key file. Overridable via
/// `ANTHROPIC_API_KEY_FILE`; otherwise the platform config dir +
/// `ph2d/anthropic_api_key` (`$XDG_CONFIG_HOME` or `~/.config` on unix,
/// `%APPDATA%` on Windows).
fn api_key_file_path() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("ANTHROPIC_API_KEY_FILE")
        && !p.is_empty()
    {
        return Some(PathBuf::from(p));
    }
    let base = if cfg!(windows) {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
    }?;
    Some(base.join("ph2d").join("anthropic_api_key"))
}

/// The toast shown when no key is found — names the two ways to set it, with the
/// exact file path so it is actionable.
fn api_key_hint() -> String {
    match api_key_file_path() {
        Some(p) => format!(
            "No Anthropic API key. Set ANTHROPIC_API_KEY, or write your key to {}, then try again.",
            p.display()
        ),
        None => "No Anthropic API key. Set ANTHROPIC_API_KEY, then try again.".to_string(),
    }
}

/// Persist `key` to the config file ([`api_key_file_path`]), creating the parent
/// dir. The next generation resolves it via [`resolve_api_key`]. An empty/blank
/// `key` clears the stored key (writes an empty file → `resolve` returns `None`).
/// On unix the file is `chmod 600` (owner-only) since it holds a secret.
pub(crate) fn save_api_key(key: &str) -> std::io::Result<()> {
    let path = api_key_file_path().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "no user config directory")
    })?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, extract_api_key(key))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

/// Pull the Anthropic key out of whatever the user pasted. If an `sk-ant-…`
/// token is present (they pasted a whole curl command, a `--header` line, or the
/// key with stray quotes/newlines) extract just that token; otherwise use the
/// trimmed input. Guards against a multi-line / quoted value producing an
/// invalid `x-api-key` header.
fn extract_api_key(raw: &str) -> String {
    for tok in raw.split(|c: char| c.is_whitespace() || c == '"' || c == '\'') {
        if tok.starts_with("sk-ant-") {
            return tok.to_string();
        }
    }
    raw.trim().to_string()
}

/// Collapse the raw LLM4SVG blob to one capped line for a toast — whitespace
/// flattened, capped so a long spec doesn't sprawl. The full blob still goes to
/// the terminal log.
fn summarize_blob(blob: &str) -> String {
    let oneline = blob.split_whitespace().collect::<Vec<_>>().join(" ");
    let max = 200;
    if oneline.chars().count() > max {
        let head: String = oneline.chars().take(max - 1).collect();
        format!("{head}\u{2026}")
    } else {
        oneline
    }
}

/// Clamp an error string to one short line for a toast — drops multi-line detail
/// (e.g. a transport error that echoes the whole request) so it neither sprawls
/// across the canvas nor leaks a pasted secret.
fn truncate_for_toast(s: &str) -> String {
    let line = s.lines().next().unwrap_or(s).trim();
    let max = 100;
    if line.chars().count() > max {
        let head: String = line.chars().take(max - 1).collect();
        format!("{head}\u{2026}")
    } else {
        line.to_string()
    }
}

/// Append a freshly-generated network to the live vector document as a new
/// editable asset, snapshotting undo first — mirrors the Pen-tool commit
/// (`vector_pen_bridge`). A free function so it is unit-testable without
/// constructing the whole [`App`]; the next-frame `vector_scene::reconcile`
/// rebuilds the ECS mirror and renders it.
pub(crate) fn inject_generated_vector(
    undo: &mut Vec<Vec<Ph2dVectorAsset>>,
    redo: &mut Vec<Vec<Ph2dVectorAsset>>,
    committed: &mut Vec<Ph2dVectorAsset>,
    net: VectorNetwork,
) {
    crate::input_dispatch::vector_undo::checkpoint(undo, redo, committed);
    committed.push(Ph2dVectorAsset::from_network(net, StyleTable::default()));
}

impl App {
    /// Open the centered LLM prompt dialog (Cmd/Ctrl+Shift+G). The user types a
    /// shape description and clicks Generate, which raises
    /// `EditorAction::GenerateVectorFromPrompt` — drained into
    /// [`submit_vector_prompt`](Self::submit_vector_prompt).
    pub(crate) fn open_vector_prompt_dialog(&mut self) {
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.store
                .open_context_menu(ph2d_editor::ContextMenuRequest {
                    x: 0.0,
                    y: 0.0,
                    kind: ph2d_editor::ContextMenuKind::VectorPromptDialog,
                });
        }
    }

    /// Drain a finished background generation (if any) and commit its network to
    /// the live document. Called once per frame from `run_render_frame`, in the
    /// clean-borrow region before the `gfx` split. Non-blocking.
    pub(crate) fn poll_llm_vector(&mut self) {
        let Some(outcome) = self.llm_vector.poll() else {
            return;
        };
        let msg = match outcome {
            LlmJobOutcome::Ready(net, blob) => {
                let n = net.vertices.len();
                inject_generated_vector(
                    &mut self.vector_undo_stack,
                    &mut self.vector_redo_stack,
                    &mut self.committed_vector_pen_paths,
                    *net,
                );
                match blob {
                    Some(b) => {
                        // The model's response — full to the terminal (a persistent
                        // log), summarized in the toast.
                        println!("[ph2d] LLM4SVG response ({n} vertices): {b}");
                        format!("Added {n} vertices — model: {}", summarize_blob(&b))
                    }
                    None => format!("Added {n} vertices (from cache) — editable + undoable."),
                }
            }
            LlmJobOutcome::Failed(reason) => {
                format!("Vector generation failed: {}", truncate_for_toast(&reason))
            }
            LlmJobOutcome::NoKey => api_key_hint(),
        };
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.toasts.push(Toast::info(msg));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_api_key_pulls_token_from_pasted_curl() {
        let curl = "curl https://api.anthropic.com/v1/messages \\\n  --header \"x-api-key: sk-ant-api03-AbC123_def\"\n  --data ...";
        assert_eq!(extract_api_key(curl), "sk-ant-api03-AbC123_def");
        // A plain key round-trips (trimmed); non-key input falls back to trimmed.
        assert_eq!(extract_api_key("  sk-ant-xyz  "), "sk-ant-xyz");
        assert_eq!(extract_api_key("  no key here  "), "no key here");
        assert_eq!(extract_api_key("   "), "");
    }

    #[test]
    fn truncate_for_toast_takes_one_short_line() {
        let multi = "LLM network error: bad header\nx-api-key: sk-ant-secret\ncurl ...";
        let out = truncate_for_toast(multi);
        assert_eq!(out, "LLM network error: bad header");
        assert!(
            !out.contains("sk-ant-"),
            "later lines (the secret) are dropped"
        );
        let long = "x".repeat(250);
        assert!(truncate_for_toast(&long).chars().count() <= 100);
    }

    #[test]
    fn inject_appends_editable_asset_and_checkpoints() {
        let mut undo: Vec<Vec<Ph2dVectorAsset>> = Vec::new();
        let mut redo: Vec<Vec<Ph2dVectorAsset>> = vec![vec![]]; // non-empty: must be cleared
        let mut committed: Vec<Ph2dVectorAsset> = Vec::new();
        let net = ph2d_vector_llm::build_network_from_json(
            r#"{ "shape_type": "polygon", "params": { "sides": 6, "radius": 80 } }"#,
        )
        .expect("valid blob lowers");

        inject_generated_vector(&mut undo, &mut redo, &mut committed, net);

        assert_eq!(
            committed.len(),
            1,
            "the generated shape is now in the document"
        );
        assert!(
            committed[0].network.validate().is_ok(),
            "and it is a valid, editable network"
        );
        assert_eq!(
            committed[0].network.vertices.len(),
            6,
            "the hexagon, six vertices"
        );
        assert_eq!(
            undo.len(),
            1,
            "one snapshot pushed → one Ctrl+Z reverts the whole add"
        );
        assert!(redo.is_empty(), "a new action clears the redo stack");
    }
}
