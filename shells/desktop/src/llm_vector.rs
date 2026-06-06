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
    /// A validated, editable network ready to commit. Boxed because a
    /// `VectorNetwork` (inline SmallVec budgets) dwarfs the other two variants.
    Ready(Box<VectorNetwork>),
    /// The fetch failed with no cached fallback, or the blob was rejected by the
    /// sanitizer. Carries a human-readable reason for the toast.
    Failed(String),
    /// `ANTHROPIC_API_KEY` was not set when the editor started.
    NoKey,
}

/// The editor's LLM-vector subsystem: at most one in-flight background
/// generation, a persistent fallback cache, and the API key read once from the
/// environment. Lives on [`App`]; constructed key-less + empty.
pub(crate) struct LlmVectorEngine {
    api_key: Option<String>,
    cache: Arc<Mutex<ResultCache>>,
    pending: Option<Receiver<LlmJobOutcome>>,
    /// Monotonic per-generation seed (also the cache key axis).
    next_seed: u64,
}

impl LlmVectorEngine {
    pub(crate) fn new() -> Self {
        Self {
            api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
            cache: Arc::new(Mutex::new(ResultCache::new(CACHE_CAP))),
            pending: None,
            next_seed: 0,
        }
    }

    /// Kick off a background generation for `prompt`. Returns `false` (no-op) if
    /// a generation is already in flight. The blocking ≤15 s client runs on a
    /// worker thread; [`Self::poll`] collects the result.
    pub(crate) fn submit(&mut self, prompt: String) -> bool {
        if self.pending.is_some() {
            return false;
        }
        let seed = self.next_seed;
        self.next_seed += 1;
        let (tx, rx) = std::sync::mpsc::channel();
        // The worker captures only `Send` data (an `Arc` clone + owned strings) —
        // never `self` — so the thread is `'static`.
        let cache = Arc::clone(&self.cache);
        let key = self.api_key.clone();
        std::thread::spawn(move || {
            let outcome = match key {
                None => LlmJobOutcome::NoKey,
                Some(k) => {
                    let client = LlmClient::new(AnthropicTransport::with_key(k));
                    // Single in-flight job ⇒ holding the lock across the ≤15 s call
                    // is contention-free, and `generate_shape` needs `&mut cache`
                    // for both the fallback read and the success write.
                    match cache.lock() {
                        Ok(mut guard) => match client.generate_shape(&prompt, seed, &mut guard) {
                            Ok(net) => LlmJobOutcome::Ready(Box::new(net)),
                            Err(e) => LlmJobOutcome::Failed(e.to_string()),
                        },
                        Err(_) => LlmJobOutcome::Failed("cache lock poisoned".to_string()),
                    }
                }
            };
            let _ = tx.send(outcome);
        });
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
    /// Smoke trigger (Cmd/Ctrl+Shift+G): fire a background LLM generation of a
    /// demo shape into the live document, so the whole pipeline — fetch →
    /// sanitize → inject → render → undo — is verifiable end-to-end now. The
    /// user-facing prompt modal that replaces this fixed prompt is the next
    /// increment. Needs `ANTHROPIC_API_KEY` in the environment.
    pub(crate) fn submit_llm_vector_smoke(&mut self) {
        const DEMO_PROMPT: &str = "a crisp six-pointed star centred at the origin";
        let started = self.llm_vector.submit(DEMO_PROMPT.to_string());
        if let Some(gfx) = self.gfx.as_mut() {
            let msg = if started {
                "Generating a vector shape from a prompt…"
            } else {
                "A vector generation is already in progress."
            };
            gfx.toasts.push(Toast::info(msg.to_string()));
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
            LlmJobOutcome::Ready(net) => {
                let n = net.vertices.len();
                inject_generated_vector(
                    &mut self.vector_undo_stack,
                    &mut self.vector_redo_stack,
                    &mut self.committed_vector_pen_paths,
                    *net,
                );
                format!("Vector shape added ({n} vertices) — editable + undoable.")
            }
            LlmJobOutcome::Failed(reason) => format!("Vector generation failed: {reason}"),
            LlmJobOutcome::NoKey => {
                "Set ANTHROPIC_API_KEY to generate vector shapes from prompts.".to_string()
            }
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
