//! The Audio Editor panel's **sections**: the collapsible blocks the body is made of,
//! and the chrome that separates them.
//!
//! The panel had grown into one unbroken column of controls. The app's canonical answer
//! is a [`SectionHeader`] (chevron + uppercase label, darker plate when folded) with a
//! [`Divider`] between blocks — exactly what the Sprite Inspector and the Audio Mixer
//! do, and the reason this file exists rather than a bespoke layout.
//!
//! Split out of `paint.rs` to keep that file (and its `paint` fn) under the panel LOC
//! caps.

use crate::paint::{ClippedHits, button, fmt_time, toggle};
use crate::{
    AEDIT_BATCH_LUFS, AEDIT_EXPORT, AEDIT_LOAD, AEDIT_LOOP, AEDIT_NAME, AEDIT_PANEL, AEDIT_PLAY,
    AEDIT_SEC_DELIVERY, AEDIT_SEC_EDIT, AEDIT_SEC_FX, AEDIT_SEC_LOOP, AEDIT_SEC_MARKERS,
    AEDIT_SEC_TRANSPORT, AEDIT_SEC_VARIATIONS, AEDIT_STOP,
};
use ph2d_a11y::NodeId;
use ph2d_editor_core::paint::{paint_text, paint_text_centered, rect_to_vello, resolve};
use ph2d_editor_core::widget::{
    Divider, SectionHeader, TextInput, TextInputState, paint_divider, paint_section_header,
    paint_text_input_with_buffer,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

use crate::paint::ROW_H;

/// Everything the body needs for one frame, bundled so the walk fits one arg list.
pub(crate) struct Body {
    /// Fold state per section, in [`SECTIONS`] order.
    pub open: [bool; 7],
    pub loaded: bool,
    pub undo_ok: bool,
    pub redo_ok: bool,
    pub has_sel: bool,
    pub transport: Transport,
    pub name: NameBox,
}

/// Walk the sections: header, then the block if it is open, then a divider. Returns the
/// `y` at the bottom of the painted content.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_body(
    y: f32,
    x: f32,
    w: f32,
    b: &Body,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) -> f32 {
    let y = paint_sound_sections(y, x, w, b, scene, text_system, theme, hit_index);
    paint_asset_sections(y, x, w, b, scene, text_system, theme, hit_index)
}

/// **Working on the sound**: transport, the edit ops, the effects rack. These open by
/// default — they are the ones you are in on every pass.
#[allow(clippy::too_many_arguments)]
fn paint_sound_sections(
    mut y: f32,
    x: f32,
    w: f32,
    b: &Body,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) -> f32 {
    // when folded) with the block's readout on its right, then the block, then a
    // `Divider`. The same chrome the Sprite Inspector and the Audio Mixer use — this
    // panel had simply grown past the point where one unbroken column still reads.
    //
    // Order groups the two jobs: **work on the sound** (transport, edit, effects) first,
    // then **prepare the asset** (loop, markers, variations, delivery), which is the half
    // that starts folded.

    let (o, ny) = section(
        y,
        x,
        w,
        AEDIT_SEC_TRANSPORT,
        "Transport",
        None,
        b.open[0],
        scene,
        text_system,
        theme,
        hit_index,
    );
    y = ny;
    if o {
        y = paint_transport_section(
            y,
            x,
            w,
            b.transport,
            &b.name,
            scene,
            text_system,
            theme,
            hit_index,
        );
    }
    y = divider(y, x, w, scene, theme);

    let (o, ny) = section(
        y,
        x,
        w,
        AEDIT_SEC_EDIT,
        "Edit",
        None,
        b.open[1],
        scene,
        text_system,
        theme,
        hit_index,
    );
    y = ny;
    if o {
        y = crate::paint_edit::paint_edit_section(
            y,
            x,
            w,
            b.loaded,
            b.undo_ok,
            b.redo_ok,
            b.has_sel,
            scene,
            text_system,
            theme,
            hit_index,
        );
    }
    y = divider(y, x, w, scene, theme);

    let (o, ny) = section(
        y,
        x,
        w,
        AEDIT_SEC_FX,
        "Effects",
        None,
        b.open[2],
        scene,
        text_system,
        theme,
        hit_index,
    );
    y = ny;
    if o {
        y = crate::paint_fx::paint_fx_section(
            y,
            x,
            w,
            b.loaded,
            ROW_H,
            scene,
            text_system,
            theme,
            hit_index,
        );
    }
    y = divider(y, x, w, scene, theme);

    y
}

/// **Preparing the asset**: loop points, cue markers, variation sets, delivery. Reached
/// for once per asset rather than once per edit, so these start FOLDED — which is what
/// keeps the panel a panel instead of a wall.
#[allow(clippy::too_many_arguments)]
fn paint_asset_sections(
    mut y: f32,
    x: f32,
    w: f32,
    b: &Body,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) -> f32 {
    let loop_read = crate::paint_loop::loop_readout();
    let (o, ny) = section(
        y,
        x,
        w,
        AEDIT_SEC_LOOP,
        "Loop",
        Some(&loop_read),
        b.open[3],
        scene,
        text_system,
        theme,
        hit_index,
    );
    y = ny;
    if o {
        y = crate::paint_loop::paint_loop_section(
            y,
            x,
            w,
            b.loaded,
            b.has_sel,
            ROW_H,
            scene,
            text_system,
            theme,
            hit_index,
        );
    }
    y = divider(y, x, w, scene, theme);

    let mark_read = crate::paint_loop::markers_readout();
    let (o, ny) = section(
        y,
        x,
        w,
        AEDIT_SEC_MARKERS,
        "Markers",
        Some(&mark_read),
        b.open[4],
        scene,
        text_system,
        theme,
        hit_index,
    );
    y = ny;
    if o {
        y = crate::paint_loop::paint_markers_section(
            y,
            x,
            w,
            b.loaded,
            ROW_H,
            scene,
            text_system,
            theme,
            hit_index,
        );
    }
    y = divider(y, x, w, scene, theme);

    let var_read = crate::paint_variation::variation_readout();
    let (o, ny) = section(
        y,
        x,
        w,
        AEDIT_SEC_VARIATIONS,
        "Variations",
        Some(&var_read),
        b.open[5],
        scene,
        text_system,
        theme,
        hit_index,
    );
    y = ny;
    if o {
        y = crate::paint_variation::paint_variation_section(
            y,
            x,
            w,
            ROW_H,
            scene,
            text_system,
            theme,
            hit_index,
        );
    }
    y = divider(y, x, w, scene, theme);

    let del_read = crate::paint_delivery::delivery_readout();
    let (o, ny) = section(
        y,
        x,
        w,
        AEDIT_SEC_DELIVERY,
        "Delivery",
        Some(&del_read),
        b.open[6],
        scene,
        text_system,
        theme,
        hit_index,
    );
    y = ny;
    if o {
        y = crate::paint_delivery::paint_delivery_section(
            y,
            x,
            w,
            b.loaded,
            ROW_H,
            scene,
            text_system,
            theme,
            hit_index,
        );
    }
    y
}

/// Every collapsible section, in paint order. The fold state is read from the store as
/// one array before the paint borrows, so the body never has to reach back into it.
pub(crate) const SECTIONS: [NodeId; 7] = [
    AEDIT_SEC_TRANSPORT,
    AEDIT_SEC_EDIT,
    AEDIT_SEC_FX,
    AEDIT_SEC_LOOP,
    AEDIT_SEC_MARKERS,
    AEDIT_SEC_VARIATIONS,
    AEDIT_SEC_DELIVERY,
];

/// Height of a section header band (matches the Sprite Inspector's).
fn section_h() -> f32 {
    TypeToken::Md.px() + Spacing::Md.px()
}

/// Paint one collapsible section header — the app's canonical chrome: chevron +
/// UPPERCASE label, and a darker plate when it is folded — with the block's readout
/// right-aligned in the same band. Registers the click that folds it (the dispatch does
/// the folding, via the `mark_collapsible_section` set) and returns `(open, y below)`.
///
/// The readout rides **in** the header rather than on a row of its own: "Variations" and
/// "3 clips" are one fact, and a folded section still has to say what is inside it.
#[allow(clippy::too_many_arguments)]
fn section(
    y: f32,
    x: f32,
    w: f32,
    id: NodeId,
    label: &str,
    readout: Option<&str>,
    open: bool,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) -> (bool, f32) {
    let h = section_h();
    let rect = Rect::new(x, y, w, h);
    paint_section_header(
        &SectionHeader::new(id, label).collapsible(open),
        rect,
        scene,
        text_system,
        theme,
    );
    hit_index.register(id, rect);

    if let Some(text) = readout.filter(|t| !t.is_empty()) {
        // Right-aligned: the label grows from the left, so anything centred would sooner
        // or later collide with it. Measure, then place against the right edge.
        let font = TypeToken::Xs.px();
        let tw = text_system.layout(text, font, w).width();
        let pad = Spacing::Md.px();
        paint_text(
            text_system,
            scene,
            text,
            (x + w - pad - tw).max(x),
            y + (h - font) * 0.5,
            font,
            w,
            resolve(ColorToken::Text2, theme),
        );
    }
    (open, y + h + Spacing::Xs.px())
}

/// A 1 px rule between sections — the canonical `Divider`. Returns the `y` below it.
fn divider(y: f32, x: f32, w: f32, scene: &mut VectorScene, theme: Theme) -> f32 {
    let gap = Spacing::Sm.px();
    let band = Rect::new(x, y + gap, w, 1.0); // LITERAL-PX-OK: the Divider widget's own 1 px rule
    paint_divider(&Divider::new(AEDIT_PANEL), band, scene, theme);
    y + gap * 2.0 + 1.0
}

/// The shell's live transport readout, bundled so the section fits one arg list.
#[derive(Clone, Copy)]
pub(crate) struct Transport {
    pub loaded: bool,
    pub playing: bool,
    pub looping: bool,
    pub pos: f64,
    pub dur: f64,
}

/// The clip-name `TextInput`'s live buffer, cloned out of the store so the scene
/// borrow below is free of it.
pub(crate) struct NameBox {
    pub state: TextInputState,
    pub text: String,
    pub caret: usize,
    pub anchor: Option<usize>,
}

/// Clip name · position/duration readout · Play/Pause · Stop | Loop · Load | Export.
/// Returns the `y` below the block.
#[allow(clippy::too_many_arguments)]
fn paint_transport_section(
    mut y: f32,
    x: f32,
    w: f32,
    t: Transport,
    name: &NameBox,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) -> f32 {
    // Clip name — an editable TextInput (mirror of the sprite name box). The widget
    // clips its own overflow to the field, so a long filename no longer wraps/crams
    // the header.
    let name_h = TypeToken::Sm.px() + Spacing::Sm.px() * 2.0;
    let name_rect = Rect::new(x, y, w, name_h);
    hit_index.register(AEDIT_NAME, name_rect);
    let input = TextInput::new(AEDIT_NAME, "")
        .placeholder("No clip loaded")
        .state(name.state);
    // Clip to the field: the TextInput lays its text out with word-wrap at the inner
    // width, so a long filename spills onto a 2nd line below the box. A clip to the
    // single-line box crops that overflow instead of letting it extrapolate.
    scene.push_clip(&rect_to_vello(name_rect));
    paint_text_input_with_buffer(
        &input,
        Some(name.text.as_str()),
        Some(name.caret),
        name.anchor,
        name_rect,
        scene,
        text_system,
        theme,
    );
    scene.pop_layer();
    y += name_h + Spacing::Sm.px();

    // Position / duration readout.
    let time_line = format!("{} / {}", fmt_time(t.pos), fmt_time(t.dur));
    paint_text_centered(
        text_system,
        scene,
        &time_line,
        Rect::new(x, y, w, TypeToken::Xs.px()),
        TypeToken::Xs.px(),
        resolve(ColorToken::Text2, theme),
    );
    y += TypeToken::Xs.px() + Spacing::Md.px();

    // Transport: Play/Pause (full width toggle, active while playing).
    let play_label = if t.playing { "Pause" } else { "Play" };
    toggle(
        Rect::new(x, y, w, ROW_H),
        play_label,
        t.playing,
        t.loaded,
        AEDIT_PLAY,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += ROW_H + Spacing::Sm.px();

    // Stop | Loop side by side.
    let gap = Spacing::Sm.px();
    let half = ((w - gap) * 0.5).max(1.0);
    button(
        Rect::new(x, y, half, ROW_H),
        "Stop",
        t.loaded,
        AEDIT_STOP,
        scene,
        text_system,
        theme,
        hit_index,
    );
    toggle(
        Rect::new(x + half + gap, y, half, ROW_H),
        "Loop",
        t.looping,
        true,
        AEDIT_LOOP,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += ROW_H + Spacing::Md.px();

    // Load | Export WAV side by side.
    button(
        Rect::new(x, y, half, ROW_H),
        "Load\u{2026}",
        true,
        AEDIT_LOAD,
        scene,
        text_system,
        theme,
        hit_index,
    );
    button(
        Rect::new(x + half + gap, y, half, ROW_H),
        "Export WAV\u{2026}",
        t.loaded,
        AEDIT_EXPORT,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += ROW_H + Spacing::Sm.px();

    // Batch LUFS — a FOLDER op (independent of the loaded clip), so always enabled.
    button(
        Rect::new(x, y, w, ROW_H),
        "Batch LUFS\u{2026}",
        true,
        AEDIT_BATCH_LUFS,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y + ROW_H + Spacing::Lg.px()
}

#[cfg(test)]
mod tests;
