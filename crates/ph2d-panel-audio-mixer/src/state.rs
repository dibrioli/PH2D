//! Audio Mixer panel retained state.

use ph2d_editor_core::zones::Rect;

/// Per-instance retained state, owned by `ErasedPanel<AudioMixerPanel>`.
#[derive(Clone, Debug, Default)]
pub struct AudioMixerState {
    /// Panel rect in viewport pixels; lazily set on first paint.
    pub rect: Option<Rect>,
}
