//! The Loop-points section of the Audio Editor panel (W6 — asset-prep).
//!
//! A loop region defined from the waveform selection (auto-snapped to zero crossings)
//! with a **Crossfade** slider that tunes the click-free seam. It is metadata — not a
//! destructive edit — and is written to the exported WAV's `smpl` chunk so a game
//! runtime loops sample-exact. It **plays via the transport's Loop toggle + Play**
//! (no separate Audition control): Loop on + a region set → Play loops the region.
//!
//! UI-only: the panel holds the crossfade slider position and arms Set/Clear intents;
//! the shell owns the `EditClip` loop region, the crossfade DSP, playback and the
//! export. The readout `(start, end)` seconds are published by the shell
//! (`loop_state::set_loop_span`).

use crate::paint::{ClippedHits, button};
use crate::{
    AEDIT_LOOP_CLEAR, AEDIT_LOOP_SET, AEDIT_LOOP_XFADE, AEDIT_MARK_ADD, AEDIT_MARK_DEL,
    AEDIT_SPLIT, loop_state,
};
use ph2d_editor_core::paint::{paint_text_centered, resolve};
use ph2d_editor_core::widget::{Slider, SliderOrientation, paint_slider, paint_slider_track};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Paint the loop section starting at `y`; returns the `y` below it. `loaded` dims
/// everything when there is no clip; `has_sel` gates "Set" (it adopts the selection).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_loop_section(
    mut y: f32,
    x: f32,
    w: f32,
    loaded: bool,
    has_sel: bool,
    row_h: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) -> f32 {
    let gap = Spacing::Sm.px();
    let has_loop = loop_state::has_loop();

    // No title row: the section header above carries the name AND the readout. Printing
    // "Loop" twice is what turned this panel into a wall of text (Enio, 2026-07-12).
    let label_h = TypeToken::Xs.px();

    // Set (from selection) | Clear.
    let half = ((w - gap) * 0.5).max(1.0);
    button(
        Rect::new(x, y, half, row_h),
        "Set Loop",
        loaded && has_sel,
        AEDIT_LOOP_SET,
        scene,
        text_system,
        theme,
        hit_index,
    );
    button(
        Rect::new(x + half + gap, y, half, row_h),
        "Clear",
        has_loop,
        AEDIT_LOOP_CLEAR,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += row_h + gap;

    // Crossfade slider (normalized 0..1 → ms, mapped shell-side). Labelled; no numeric
    // readout — it is a feel control and the exact ms is the shell's business.
    paint_text_centered(
        text_system,
        scene,
        "Crossfade",
        Rect::new(x, y, w, label_h),
        TypeToken::Xs.px(),
        resolve(ColorToken::Text2, theme),
    );
    y += label_h + Spacing::Xs.px();
    let track = Rect::new(x, y, w, Spacing::Md.px());
    if has_loop {
        let mut slider =
            Slider::new(AEDIT_LOOP_XFADE, "Crossfade").orientation(SliderOrientation::Horizontal);
        slider.set_value(loop_state::xfade_norm());
        paint_slider(&slider, track, scene, theme);
        hit_index.register(AEDIT_LOOP_XFADE, track);
    } else {
        // Inert track (no thumb, not hit-registered) when there is nothing to audition.
        paint_slider_track(
            track,
            loop_state::xfade_norm(),
            SliderOrientation::Horizontal,
            scene,
            theme,
        );
    }
    y + Spacing::Md.px() + Spacing::Md.px()
}

/// The region readout: `1.20\u{2013}3.40s`, or `No loop` when unset.
pub(crate) fn loop_readout() -> String {
    match loop_state::loop_span() {
        Some((s, e)) => format!("{s:.2}\u{2013}{e:.2}s"),
        None => "No loop".to_string(),
    }
}

/// The Markers section (W6): **Add Marker** (at the playhead) | **Delete** (nearest). Cue points exported to the WAV `cue`+`adtl` so a
/// game runtime can react to them. Returns the `y` below it.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_markers_section(
    y: f32,
    x: f32,
    w: f32,
    loaded: bool,
    row_h: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) -> f32 {
    let mut y = y;
    let gap = Spacing::Sm.px();
    let count = loop_state::marker_count();

    // Add (at playhead) | Delete (nearest).
    let half = ((w - gap) * 0.5).max(1.0);
    button(
        Rect::new(x, y, half, row_h),
        "Add Marker",
        loaded,
        AEDIT_MARK_ADD,
        scene,
        text_system,
        theme,
        hit_index,
    );
    button(
        Rect::new(x + half + gap, y, half, row_h),
        "Delete",
        count > 0,
        AEDIT_MARK_DEL,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += row_h + gap;

    // **Split at Markers** — one file per piece. It lives here and not in Edit because the markers
    // ARE the cuts: a session of N takes with N-1 markers between them falls into N assets, and
    // `<stem>_01..NN` is exactly what the variation importer reads back as one group.
    button(
        Rect::new(x, y, w, row_h),
        "Split at Markers",
        loaded && count > 0,
        AEDIT_SPLIT,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y + row_h + Spacing::Md.px()
}

/// The Markers readout, for the section header: the panel says how many cue points the
/// clip carries without being unfolded.
pub(crate) fn markers_readout() -> String {
    match loop_state::marker_count() {
        0 => "No markers".to_string(),
        1 => "1 marker".to_string(),
        n => format!("{n} markers"),
    }
}
