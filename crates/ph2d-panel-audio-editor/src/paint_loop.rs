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

use crate::paint::{ClippedHits, button, button_in_group, buttons_block};
use crate::{
    AEDIT_LOOP_BAKE, AEDIT_LOOP_CLEAR, AEDIT_LOOP_SET, AEDIT_LOOP_XFADE, AEDIT_MARK_ADD,
    AEDIT_MARK_DEL, AEDIT_SPLIT, loop_state,
};

use ph2d_editor_core::widget::{button_label_font, segment_rects_for};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, Theme};
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
    let gap = Spacing::Xs.px();
    let has_loop = loop_state::has_loop();

    // No title row: the section header above carries the name AND the readout. Printing
    // "Loop" twice is what turned this panel into a wall of text (Enio, 2026-07-12).

    // Set (from selection) | Clear.
    // ⭐ A fileira mede as PALAVRAS (`segment_rects_for`): em partes iguais, a peça mais larga
    //    podia ser cortada com a fileira a caber inteira.
    let seg = segment_rects_for(
        Rect::new(x, y, w, row_h),
        &[
            tr("panel.audio_editor.loop.set_loop"),
            tr("panel.audio_editor.loop.clear"),
        ],
        button_label_font(),
        text_system,
    );
    button_in_group(
        seg[0].0,
        tr("panel.audio_editor.loop.set_loop"),
        loaded && has_sel,
        AEDIT_LOOP_SET,
        seg[0].1,
        scene,
        text_system,
        theme,
        hit_index,
    );
    button_in_group(
        seg[1].0,
        tr("panel.audio_editor.loop.clear"),
        has_loop,
        AEDIT_LOOP_CLEAR,
        seg[1].1,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += row_h + gap;

    // Crossfade (normalized 0..1 → ms, mapped shell-side), na CAIXA ÚNICA do app.
    //
    // ⚠️ **A nota anterior dizia *«no numeric readout — it is a feel control e o ms exacto é
    //    assunto da shell»*, e o que ela descrevia era a AUSÊNCIA de uma leitura, não uma
    //    decisão sobre qual.** A caixa mostra a FRACÇÃO, que é o que este painel tem: ela não
    //    promete `ms` nenhum e diz onde o dedo está — *um controlo de sensação continua a ser um
    //    controlo, e um que não diz nada obriga a arrastar para descobrir onde estava*.
    //
    // ⛔ Sem laço a fileira é pintada e NÃO registada — ver [`crate::fileira_de_param`].
    y = crate::fileira_de_param::fileira_de_param(
        y,
        x,
        w,
        tr("panel.audio_editor.loop.crossfade"),
        loop_state::xfade_norm(),
        None,
        AEDIT_LOOP_XFADE,
        has_loop,
        scene,
        text_system,
        theme,
        hit_index,
    ) + ph2d_tokens::control_gap_px();

    // **Crossfade Loop** — bake the seam into the audio.
    //
    // A runtime loop *jumps*; it does not crossfade. The slider used to sweeten a preview buffer
    // that only the editor ever heard, so the loop was clean on screen and clicked in the game. The
    // slider is the amount now, and this is the verb: it writes the seam into the samples, using the
    // intro as pre-roll. Dim without a loop, without a crossfade, or when the loop starts at frame 0
    // (nothing before it to fade from — that is what the zero-crossing snap is for).
    button(
        Rect::new(x, y, w, row_h),
        tr("panel.audio_editor.loop.crossfade_loop"),
        loop_state::can_bake(),
        AEDIT_LOOP_BAKE,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y + row_h + ph2d_tokens::control_gap_px()
}

/// The region readout: `1.20\u{2013}3.40s`, or `No loop` when unset.
pub(crate) fn loop_readout() -> String {
    match loop_state::loop_span() {
        Some((s, e)) => format!("{s:.2}\u{2013}{e:.2}s"),
        None => tr("panel.audio_editor.loop.no_loop").to_string(),
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
    let count = loop_state::marker_count();

    // ⭐⭐ **Os três marcadores são UM corpo** (wave 20, o dono nomeou esta secção: *«na seção
    //    Markers os botões podem ser agrupados pois se referem ao mesmo assunto»*). Antes eram
    //    dois controlos: o par `Add | Delete` agrupado na horizontal, e o `Split` **fora** dele,
    //    separado por um `y += row_h + gap` escrito à mão.
    //
    // **Split at Markers** — cut the clip at every marker, and nothing else. It lives here and not
    // in Edit because the markers ARE the cuts: a session of N takes with N-1 markers between them
    // falls into N pieces you can then select, drag and stretch.
    //
    // It used to encode those pieces to disk and adopt them as a variation set — an emitting verb
    // wearing an edit verb's name. That is **Export Pieces** now, in Delivery, where emitting
    // lives.
    let y = buttons_block(
        Rect::new(x, y, w, row_h),
        &[2, 1],
        &[
            (
                tr("panel.audio_editor.loop.add_marker"),
                loaded,
                AEDIT_MARK_ADD,
            ),
            (
                tr("panel.audio_editor.loop.delete"),
                count > 0,
                AEDIT_MARK_DEL,
            ),
            (
                tr("panel.audio_editor.loop.split_at_markers"),
                loaded && count > 0,
                AEDIT_SPLIT,
            ),
        ],
        scene,
        text_system,
        theme,
        hit_index,
    );
    y + ph2d_tokens::control_gap_px()
}

/// The Markers readout, for the section header: the panel says how many cue points the
/// clip carries without being unfolded.
pub(crate) fn markers_readout() -> String {
    match loop_state::marker_count() {
        0 => tr("panel.audio_editor.loop.no_markers").to_string(),
        1 => tr("panel.audio_editor.loop.one_marker").to_string(),
        n => ph2d_i18n::tr_with("panel.audio_editor.loop.n_markers", &[("n", &n)]),
    }
}
