//! Spectral panel state (W5): which view the overlay shows, and what the repair tools have
//! to work with.
//!
//! Same split as `delivery_state`: the panel owns the two things the user turns (the view
//! toggle and the denoise amount), and the shell — which is the only side that can see the
//! clip, run an FFT, or know whether a region is selected — **publishes every fact back as
//! something already decided**. The panel never computes; it paints.
//!
//! Thread-local: panel and bridge both run on the main thread.

use std::cell::{Cell, RefCell};

/// Where the denoise Amount slider starts. Enough to hear, gentle enough not to hollow a
/// voice out on the first click — the point at which a user learns what the control does.
pub(crate) const DEFAULT_AMOUNT: f32 = 0.5;

thread_local! {
    /// Panel → overlay: draw the spectrogram instead of the waveform.
    static VIEW: Cell<bool> = const { Cell::new(false) };
    /// Panel → shell: how hard to denoise.
    static AMOUNT: Cell<f32> = const { Cell::new(DEFAULT_AMOUNT) };
    /// Shell → panel: a noise profile has been learned, so Denoise can run. Without one
    /// the button is dimmed — denoising against a profile you never taught it would
    /// either do nothing or gut the clip, and neither is a thing to find out by clicking.
    static HAS_PROFILE: Cell<bool> = const { Cell::new(false) };
    /// Shell → panel: the selection is a time-AND-frequency region (drawn in the
    /// spectrogram), which is the only thing Repair can act on. A plain time selection is
    /// not enough: it says *when*, and repair also needs to know *what*.
    static HAS_BAND: Cell<bool> = const { Cell::new(false) };
    /// Shell → panel: the finished status line (what is selected, what was learned).
    static STATUS: RefCell<String> = const { RefCell::new(String::new()) };
    /// Shell → panel: the shell was built with the `audio-ml` feature, so the AI Denoise
    /// (DeepFilterNet, W7) button has a home to dispatch to. Default `false`: without the
    /// feature the shell never flips this, the button is never painted, and nothing dead is
    /// shown — the panel crate stays feature-agnostic, the shell owns the one switch.
    static ML_AVAILABLE: Cell<bool> = const { Cell::new(false) };
}

/// Panel: flip between waveform and spectrogram.
pub(crate) fn toggle_view() {
    VIEW.with(|c| c.set(!c.get()));
}

/// Overlay / shell: is the spectrogram showing?
pub fn view() -> bool {
    VIEW.with(Cell::get)
}

/// Panel: the Amount slider moved.
pub(crate) fn set_amount(v: f32) {
    AMOUNT.with(|c| c.set(v.clamp(0.0, 1.0)));
}

/// Shell + panel: how hard to denoise, `0..1`.
pub fn amount() -> f32 {
    AMOUNT.with(Cell::get)
}

/// Shell: publish what the tools have to work with, and the status line to show for it.
pub fn set_ready(has_profile: bool, has_band: bool, status: &str) {
    HAS_PROFILE.with(|c| c.set(has_profile));
    HAS_BAND.with(|c| c.set(has_band));
    STATUS.with(|c| c.borrow_mut().replace_range(.., status));
}

pub(crate) fn has_profile() -> bool {
    HAS_PROFILE.with(Cell::get)
}

pub(crate) fn has_band() -> bool {
    HAS_BAND.with(Cell::get)
}

pub(crate) fn status() -> String {
    STATUS.with(|c| c.borrow().clone())
}

/// Shell: whether AI Denoise (DeepFilterNet, W7) is compiled in. The shell publishes this once
/// it knows its own build features (`cfg!(feature = "audio-ml")`); the panel paints the button
/// only when it is `true`.
pub fn set_ml_available(v: bool) {
    ML_AVAILABLE.with(|c| c.set(v));
}

/// Panel: is AI Denoise available in this build?
pub(crate) fn ml_available() -> bool {
    ML_AVAILABLE.with(Cell::get)
}
