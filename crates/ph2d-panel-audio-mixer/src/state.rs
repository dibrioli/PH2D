//! Audio Mixer panel retained state.
//!
//! The panel is right-docked in the shared Inspector slot (it reads
//! `ctx.layout.inspector` each frame), so it holds no floating geometry — the
//! dock rect + drag/resize live on the shared `INSP_*` handles.

/// Per-instance retained state, owned by `ErasedPanel<AudioMixerPanel>`.
#[derive(Clone, Debug, Default)]
pub struct AudioMixerState;
