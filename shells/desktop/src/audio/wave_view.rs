//! The overlay's viewport, published once a frame so the mouse handlers can read it.
//!
//! The overlay knows where it drew the waveform (or the spectrogram — same rectangle);
//! the pointer handlers, which run in a different pass entirely, need that rectangle to
//! turn a click into a **clip frame**. Neither can call the other, so the overlay leaves
//! the rectangle here on its way past and the handlers pick it up.
//!
//! Extracted from `audio.rs` when that file crossed the shell's 600-LOC cap (HR-18). It is
//! a coherent unit on its own: everything the overlay↔pointer seam shares, and nothing else.

/// The overlay viewport (screen rect + clip length) the overlay publishes each frame, so
/// the shell's mouse handlers can hit-test a press over it and map screen-x → clip frame
/// for the selection drag.
#[derive(Clone, Copy)]
pub(crate) struct WaveView {
    pub rect: ph2d_editor::zones::Rect,
    /// The time-ruler strip above the wave — the playhead scrub hit-region (the body below
    /// it is the selection region). Same x/width as `rect`.
    pub ruler: ph2d_editor::zones::Rect,
    pub frames: u64,
}

thread_local! {
    static WAVE_VIEW: std::cell::Cell<Option<WaveView>> = const { std::cell::Cell::new(None) };
}

/// Overlay → shell: publish (or clear) the viewport for this frame.
pub(crate) fn set_wave_view(v: Option<WaveView>) {
    WAVE_VIEW.with(|c| c.set(v));
}

/// Shell mouse handlers: the current viewport, if the overlay is shown.
pub(crate) fn wave_view() -> Option<WaveView> {
    WAVE_VIEW.with(std::cell::Cell::get)
}

/// Map a screen `x` to a clip frame within `view` (clamped to the clip).
pub(crate) fn frame_at_x(view: &WaveView, x: f32) -> u64 {
    let t = ((x - view.rect.x) / view.rect.w.max(1.0)).clamp(0.0, 1.0);
    ((t as f64) * view.frames as f64) as u64
}
