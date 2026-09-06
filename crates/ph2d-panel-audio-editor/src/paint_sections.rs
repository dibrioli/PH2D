//! The Audio Editor panel's **sections**: the collapsible blocks the body is made of,
//! and the chrome that separates them.
//!
//! The panel had grown into one unbroken column of controls. The app's canonical answer
//! is a [`SectionHeader`] (chevron + uppercase label, darker plate when folded) with
//! `paint_section_separator` — the 1 px **accent-coloured** rule — between blocks. Both
//! come from the Widget Gallery, which is the single source of truth for chrome
//! (DIRETRIZ §5.2); this file exists so the panel wears the app's clothes rather than
//! its own.
//!
//! Split out of `paint.rs` to keep that file (and its `paint` fn) under the panel LOC
//! caps.

use crate::paint::{ClippedHits, button_in_group, fmt_time, toggle_in_group};
use crate::{
    AEDIT_BATCH_LUFS, AEDIT_EXPORT, AEDIT_LOAD, AEDIT_LOOP, AEDIT_NAME, AEDIT_PLAY,
    AEDIT_SEC_DELIVERY, AEDIT_SEC_EDIT, AEDIT_SEC_FX, AEDIT_SEC_LOOP, AEDIT_SEC_MARKERS,
    AEDIT_SEC_SPECTRAL, AEDIT_SEC_TRANSPORT, AEDIT_SEC_VARIATIONS, AEDIT_STOP,
};
use ph2d_a11y::NodeId;
use ph2d_editor_core::paint::{paint_text, paint_text_centered, rect_to_vello, resolve};
use ph2d_editor_core::widget::section_cards::{SectionCards, with_section_cards};
use ph2d_editor_core::widget::{
    SectionFold, SectionHeader, TextInput, TextInputState, block_cells, grid_height,
    paint_section_header, paint_text_input_with_buffer,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

use crate::paint::ROW_H;

/// Everything the body needs for one frame, bundled so the walk fits one arg list.
pub(crate) struct Body {
    /// Fold state per section, in [`SECTIONS`] order: **o par** `(aberta?, t VIVO)`.
    ///
    /// ⚠️ **Um array de PARES e não dois arrays paralelos** — duas listas indexadas pela mesma
    /// posição são duas cópias do mesmo facto, e a segunda é a que alguém esquece de reordenar.
    pub open: [(bool, f32); 8],
    pub loaded: bool,
    pub undo_ok: bool,
    pub redo_ok: bool,
    pub has_sel: bool,
    pub transport: Transport,
    pub name: NameBox,
}

impl Body {
    /// O par que o cabeçalho veste: o estado semântico e o `t` VIVO da dobra.
    ///
    /// **By id, never by index**: `open[2]` means whatever the third entry of [`SECTIONS`]
    /// happens to be today, so reordering the panel would silently hand every section somebody
    /// else's fold state — a bug that paints perfectly and is invisible until a user clicks the
    /// wrong chevron.
    fn fold(&self, id: NodeId) -> (bool, f32) {
        SECTIONS
            .iter()
            .position(|s| *s == id)
            .map_or((false, 0.0), |i| self.open[i])
    }
}

#[path = "paint_sections_chrome.rs"]
mod chrome;
#[cfg(test)]
use chrome::section_h;
use chrome::{end_fold, section, separator};

/// Walk the sections: header, then the block if it is open, then the accent separator.
/// Returns the `y` at the bottom of the painted content.
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
    with_section_cards(scene, theme, y, |scene, cards| {
        let ny = paint_sound_sections(y, x, w, b, scene, text_system, theme, hit_index, cards);
        paint_asset_sections(ny, x, w, b, scene, text_system, theme, hit_index, cards)
    })
}

/// **Working on the sound**: transport, the loop region, the edit ops, the effects rack.
///
/// The loop sits here, right under the transport, because it is a *playback* thing —
/// Loop-on + Play is how you audition it (Enio, 2026-07-12). Only it starts folded, and
/// only because an unset loop has nothing to show; its header still says so.
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
    cards: &mut SectionCards,
) -> f32 {
    let (fold, ny) = section(
        y,
        x,
        w,
        AEDIT_SEC_TRANSPORT,
        "Transport",
        None,
        b.fold(AEDIT_SEC_TRANSPORT),
        scene,
        text_system,
        theme,
        hit_index,
    );
    y = ny;
    cards.skip_header(y);
    if let Some(fold) = fold {
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
        y = end_fold(fold, y, scene, hit_index);
    }
    y = separator(y, x, w, scene, cards);

    let loop_read = crate::paint_loop::loop_readout();
    let (fold, ny) = section(
        y,
        x,
        w,
        AEDIT_SEC_LOOP,
        "Loop",
        Some(&loop_read),
        b.fold(AEDIT_SEC_LOOP),
        scene,
        text_system,
        theme,
        hit_index,
    );
    y = ny;
    cards.skip_header(y);
    if let Some(fold) = fold {
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
        y = end_fold(fold, y, scene, hit_index);
    }
    y = separator(y, x, w, scene, cards);

    let (fold, ny) = section(
        y,
        x,
        w,
        AEDIT_SEC_EDIT,
        "Edit",
        None,
        b.fold(AEDIT_SEC_EDIT),
        scene,
        text_system,
        theme,
        hit_index,
    );
    y = ny;
    cards.skip_header(y);
    if let Some(fold) = fold {
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
        y = end_fold(fold, y, scene, hit_index);
    }
    y = separator(y, x, w, scene, cards);

    // Spectral sits between Edit and Effects: it IS editing (destructive, undoable), it
    // just edits in a domain the waveform cannot show. Reach for it after the cuts and
    // before the rack.
    let (fold, ny) = section(
        y,
        x,
        w,
        AEDIT_SEC_SPECTRAL,
        "Spectral",
        Some(crate::paint_spectral::spectral_readout()),
        b.fold(AEDIT_SEC_SPECTRAL),
        scene,
        text_system,
        theme,
        hit_index,
    );
    y = ny;
    cards.skip_header(y);
    if let Some(fold) = fold {
        y = crate::paint_spectral::paint_spectral_section(
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
        y = end_fold(fold, y, scene, hit_index);
    }
    y = separator(y, x, w, scene, cards);

    let (fold, ny) = section(
        y,
        x,
        w,
        AEDIT_SEC_FX,
        "Effects",
        None,
        b.fold(AEDIT_SEC_FX),
        scene,
        text_system,
        theme,
        hit_index,
    );
    y = ny;
    cards.skip_header(y);
    if let Some(fold) = fold {
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
        y = end_fold(fold, y, scene, hit_index);
    }
    y = separator(y, x, w, scene, cards);

    y
}

/// **Preparing the asset**: cue markers, variation sets, delivery. Reached for once per
/// asset rather than once per edit, so these start FOLDED — which is what keeps the panel
/// a panel instead of a wall.
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
    cards: &mut SectionCards,
) -> f32 {
    let mark_read = crate::paint_loop::markers_readout();
    let (fold, ny) = section(
        y,
        x,
        w,
        AEDIT_SEC_MARKERS,
        "Markers",
        Some(&mark_read),
        b.fold(AEDIT_SEC_MARKERS),
        scene,
        text_system,
        theme,
        hit_index,
    );
    y = ny;
    cards.skip_header(y);
    if let Some(fold) = fold {
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
        y = end_fold(fold, y, scene, hit_index);
    }
    y = separator(y, x, w, scene, cards);

    let var_read = crate::paint_variation::variation_readout();
    let (fold, ny) = section(
        y,
        x,
        w,
        AEDIT_SEC_VARIATIONS,
        "Variations",
        Some(&var_read),
        b.fold(AEDIT_SEC_VARIATIONS),
        scene,
        text_system,
        theme,
        hit_index,
    );
    y = ny;
    cards.skip_header(y);
    if let Some(fold) = fold {
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
        y = end_fold(fold, y, scene, hit_index);
    }
    y = separator(y, x, w, scene, cards);

    let del_read = crate::paint_delivery::delivery_readout();
    let (fold, ny) = section(
        y,
        x,
        w,
        AEDIT_SEC_DELIVERY,
        "Delivery",
        Some(&del_read),
        b.fold(AEDIT_SEC_DELIVERY),
        scene,
        text_system,
        theme,
        hit_index,
    );
    y = ny;
    cards.skip_header(y);
    if let Some(fold) = fold {
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
        y = end_fold(fold, y, scene, hit_index);
    }
    y
}

/// Every collapsible section, in paint order. The fold state is read from the store as
/// one array before the paint borrows, so the body never has to reach back into it.
pub(crate) const SECTIONS: [NodeId; 8] = [
    AEDIT_SEC_TRANSPORT,
    AEDIT_SEC_LOOP,
    AEDIT_SEC_EDIT,
    AEDIT_SEC_SPECTRAL,
    AEDIT_SEC_FX,
    AEDIT_SEC_MARKERS,
    AEDIT_SEC_VARIATIONS,
    AEDIT_SEC_DELIVERY,
];

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
    /// ⚠️ **Quanto do hover está presente.** Vem no snapshot e não é lido aqui do store porque
    /// este pintor não tem um — quem sabe é quem monta o `NameBox`.
    pub hover_t: f32,
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
        .visual((name.state, name.hover_t));
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

    // ⭐⭐ **As QUATRO fileiras do transporte são um corpo só** (`1 · 2 · 2 · 1`) — Enio,
    //    2026-09-06: *«na vertical ainda tem muito espaço ainda»*. Elas fazem a mesma coisa
    //    (comandar o clipe), logo encostam, e só os quatro cantos do BLOCO arredondam.
    let block = block_cells(Rect::new(x, y, w, 0.0), &[1, 2, 2, 1], ROW_H);
    // Transport: Play/Pause (full width toggle, active while playing).
    let play_label = if t.playing { "Pause" } else { "Play" };
    toggle_in_group(
        block[0][0].0,
        block[0][0].1,
        play_label,
        t.playing,
        t.loaded,
        AEDIT_PLAY,
        scene,
        text_system,
        theme,
        hit_index,
    );

    // Stop | Loop side by side.
    let seg = &block[1];
    button_in_group(
        seg[0].0,
        "Stop",
        t.loaded,
        AEDIT_STOP,
        seg[0].1,
        scene,
        text_system,
        theme,
        hit_index,
    );
    toggle_in_group(
        seg[1].0,
        seg[1].1,
        "Loop",
        t.looping,
        true,
        AEDIT_LOOP,
        scene,
        text_system,
        theme,
        hit_index,
    );

    // Load | Export WAV side by side.
    let seg = &block[2];
    button_in_group(
        seg[0].0,
        "Load\u{2026}",
        true,
        AEDIT_LOAD,
        seg[0].1,
        scene,
        text_system,
        theme,
        hit_index,
    );
    button_in_group(
        seg[1].0,
        "Export WAV\u{2026}",
        t.loaded,
        AEDIT_EXPORT,
        seg[1].1,
        scene,
        text_system,
        theme,
        hit_index,
    );

    // Batch LUFS — a FOLDER op (independent of the loaded clip), so always enabled.
    button_in_group(
        block[3][0].0,
        "Batch LUFS\u{2026}",
        true,
        AEDIT_BATCH_LUFS,
        block[3][0].1,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y + grid_height(4, ROW_H) + Spacing::Lg.px()
}

#[cfg(test)]
mod tests;
