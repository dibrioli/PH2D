//! Audio Editor panel retained state.
//!
//! Docked in the shared Inspector slot (reads `ctx.layout.inspector` each
//! frame), so it holds no floating geometry — the dock rect + drag/resize live
//! on the shared `INSP_*` handles. The transport intents + live display values
//! live in the thread-local [`crate::snapshot`] bridge, not here.

/// Per-instance retained state, owned by `ErasedPanel<AudioEditorPanel>`.
#[derive(Clone, Debug, Default)]
pub struct AudioEditorState;
