//! Delivery panel state for the Audio Editor (W6 asset-prep): **codec + cost readout**.
//!
//! The panel owns the two things a user turns — which codec the asset ships as, and the
//! Vorbis quality — and nothing else. It cannot size a file (it has no encoder) or know
//! what the mixer spends, so the **shell publishes every number as a finished string**
//! and the panel just paints it. Same split as `loop_state` / `variation_state`.
//!
//! Thread-local: the panel and the shell bridge both run on the main thread.

use std::cell::{Cell, RefCell};

/// Where the Vorbis quality slider starts (mirrors `ph2d_audio_encode::OGG_DEFAULT_QUALITY`).
pub(crate) const DEFAULT_QUALITY_NORM: f32 = 0.5;

thread_local! {
    /// Panel → shell: which codec is selected, as an index into the shell's table.
    static CODEC: Cell<usize> = const { Cell::new(0) };
    /// Shell → panel: how many codecs there are (so the selector wraps correctly) and
    /// the selected one's display name.
    static CODEC_COUNT: Cell<usize> = const { Cell::new(1) };
    static CODEC_NAME: RefCell<String> = const { RefCell::new(String::new()) };
    /// Shell → panel: whether the selected codec is lossy — the Quality slider is inert
    /// on a lossless one, and says so by dimming rather than by lying.
    static LOSSY: Cell<bool> = const { Cell::new(false) };
    /// Panel → shell: the Vorbis quality slider, normalized `0..1`.
    static QUALITY_NORM: Cell<f32> = const { Cell::new(DEFAULT_QUALITY_NORM) };
    /// Shell → panel: the finished readout lines.
    static DISK: RefCell<String> = const { RefCell::new(String::new()) };
    static RAM: RefCell<String> = const { RefCell::new(String::new()) };
    /// Shell → panel: RAM as a fraction of the audio subsystem's budget (HR-13).
    static BUDGET_FRAC: Cell<f32> = const { Cell::new(0.0) };
    /// Shell → panel: this codec would drop the loop points / markers the clip carries
    /// (only WAV has chunks for them). The panel warns BEFORE the export, not after.
    static DROPS_META: Cell<bool> = const { Cell::new(false) };
}

/// Panel: step the codec selector by `delta`, wrapping.
pub(crate) fn cycle_codec(delta: i32) {
    let count = CODEC_COUNT.with(|c| c.get()).max(1) as i32;
    CODEC.with(|c| {
        let next = (c.get() as i32 + delta).rem_euclid(count);
        c.set(next as usize);
    });
}

/// Shell: which codec the user picked.
pub fn codec() -> usize {
    CODEC.with(|c| c.get())
}

/// Shell: publish the codec table's size + the selected one's name and lossiness.
pub fn set_codec_info(count: usize, name: &str, lossy: bool) {
    CODEC_COUNT.with(|c| c.set(count));
    CODEC_NAME.with(|c| c.borrow_mut().replace_range(.., name));
    LOSSY.with(|c| c.set(lossy));
}

pub(crate) fn codec_name() -> String {
    CODEC_NAME.with(|c| c.borrow().clone())
}

pub(crate) fn is_lossy() -> bool {
    LOSSY.with(|c| c.get())
}

/// Panel: the quality slider moved.
pub(crate) fn set_quality_norm(v: f32) {
    QUALITY_NORM.with(|c| c.set(v.clamp(0.0, 1.0)));
}

pub(crate) fn quality_norm() -> f32 {
    QUALITY_NORM.with(|c| c.get())
}

/// Shell: read the quality slider (already `0..1`, which is also Vorbis's range).
pub fn quality() -> f32 {
    QUALITY_NORM.with(|c| c.get())
}

/// Shell: publish the cost readout. `disk` / `ram` are pre-formatted; `budget_frac` is
/// what the RAM figure is worth against the audio subsystem's envelope.
pub fn set_cost(disk: &str, ram: &str, budget_frac: f32, drops_meta: bool) {
    DISK.with(|c| c.borrow_mut().replace_range(.., disk));
    RAM.with(|c| c.borrow_mut().replace_range(.., ram));
    BUDGET_FRAC.with(|c| c.set(budget_frac));
    DROPS_META.with(|c| c.set(drops_meta));
}

pub(crate) fn disk() -> String {
    DISK.with(|c| c.borrow().clone())
}

pub(crate) fn ram() -> String {
    RAM.with(|c| c.borrow().clone())
}

pub(crate) fn budget_frac() -> f32 {
    BUDGET_FRAC.with(|c| c.get())
}

pub(crate) fn drops_meta() -> bool {
    DROPS_META.with(|c| c.get())
}
