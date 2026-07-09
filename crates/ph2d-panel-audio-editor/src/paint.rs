//! Audio Editor panel paint — **right-docked in the shared Inspector slot**
//! (mirror of the Audio Mixer / Sprite Inspector dock pattern), NOT a floating
//! panel. Reads `ctx.layout.inspector` for its rect and registers the shared
//! `INSP_*` drag/resize handles so it moves/resizes with the dock slot.
//!
//! Compact controls only: a clip readout (name · position / duration), a
//! transport (Play/Pause · Stop · Loop) and Load / Export. The spacious waveform
//! + timeline are the separate floating overlay on the canvas.

use crate::state::AudioEditorState;
use crate::{
    AEDIT_CLOSE, AEDIT_CUT, AEDIT_DC, AEDIT_EXPORT, AEDIT_FADE_IN, AEDIT_FADE_OUT, AEDIT_FX_PARAMS,
    AEDIT_GAIN_DOWN, AEDIT_GAIN_UP, AEDIT_INVERT, AEDIT_LOAD, AEDIT_LOOP, AEDIT_NAME,
    AEDIT_NORM_LUFS, AEDIT_NORMALIZE, AEDIT_PANEL, AEDIT_PLAY, AEDIT_REDO, AEDIT_REVERSE,
    AEDIT_SILENCE, AEDIT_STOP, AEDIT_TRIM, AEDIT_UNDO, AudioEditorPanel, snapshot,
};
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{HitIndex, InteractiveState};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_centered, rect_to_vello, resolve};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEADER_CLOSE_RESERVE, PANEL_TITLE_BASELINE, paint_panel_close_button,
    paint_panel_corner_dot, paint_panel_corner_dot_bl, paint_panel_surface, paint_panel_title,
    panel_close_button_rect,
};
use ph2d_editor_core::widget::{TextInput, TextInputState, paint_text_input_with_buffer};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

const ROW_H: f32 = 28.0; // LITERAL-PX-OK: transport button row height (chrome)
/// Fixed width of the docked Audio Editor panel. It sits just LEFT of the shared
/// Inspector slot so it can be open side-by-side with the Audio Mixer (which owns
/// that slot) — the transport is compact, so it needs less width than a mixer.
const PANEL_W: f32 = 240.0; // LITERAL-PX-OK: docked editor panel width (chrome)

pub(crate) fn paint(_state: &mut AudioEditorState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(AudioEditorPanel::ID) {
        ctx.host.store_mut().clear_panel_rect(AEDIT_PANEL);
        return;
    }
    // Dock just LEFT of the shared Inspector slot (which the Audio Mixer owns),
    // so MIX + WAVE can be open side-by-side. Follows the Inspector rect if the
    // user moves/resizes that dock. Its own drag/resize is NOT wired (the compact
    // controls don't need it; the movable part is the floating waveform overlay).
    let insp = ctx.layout.inspector;
    let gap = Spacing::Md.px();
    let rect = Rect::new((insp.x - PANEL_W - gap).max(0.0), insp.y, PANEL_W, insp.h);
    let theme = ctx.host.theme();
    ctx.host.store_mut().set_panel_rect(AEDIT_PANEL, rect);

    // Opaque backing (the glass surface would bleed the canvas/panel behind it).
    fill_rounded_rect(
        ctx.scene,
        rect,
        Radius::Sm.px(),
        resolve(ColorToken::BgElev, theme),
    );
    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);
    paint_panel_corner_dot_bl(rect, ctx.scene, theme);

    let title_size = paint_panel_title(
        rect,
        "Audio Editor",
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    paint_panel_close_button(
        rect,
        AEDIT_CLOSE,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );

    let pad = Spacing::Lg.px();
    let x = rect.x + pad;
    let w = (rect.w - pad * 2.0).max(1.0);
    let mut y = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();

    // Snapshot (shell → panel).
    let loaded = snapshot::loaded();
    let playing = snapshot::playing();
    let looping = snapshot::looping();
    let pos = snapshot::position_secs();
    let dur = snapshot::duration_secs();
    let undo_ok = snapshot::can_undo();
    let redo_ok = snapshot::can_redo();
    let has_sel = snapshot::has_selection();

    // Sync the name box from the loaded clip — mirror of the Inspector entity-name
    // box: overwrite the TextInput's buffer only when a NEW clip loads, and skip
    // it while the user is editing (focused), so keystrokes aren't clobbered.
    if let Some(name) = snapshot::clip_name_needs_sync()
        && ctx.host.store().focus_id() != Some(AEDIT_NAME)
    {
        if let Some(InteractiveState::TextInput {
            state,
            text,
            caret,
            selection_anchor,
        }) = ctx.host.store_mut().get_mut(AEDIT_NAME)
        {
            *state = TextInputState::Normal;
            text.clear();
            text.push_str(&name);
            *caret = text.len();
            *selection_anchor = None;
        }
        snapshot::mark_name_synced();
    }
    // Seed the parameter sliders with the selected effect's preset — once per kind
    // change, never every frame (which would fight the user's drag). Same guard
    // shape as the name box above.
    if let Some(defaults) = snapshot::fx_defaults_need_sync() {
        for (i, id) in AEDIT_FX_PARAMS.iter().enumerate() {
            if let Some(InteractiveState::Slider { value, .. }) = ctx.host.store_mut().get_mut(*id)
            {
                *value = defaults[i];
            }
            snapshot::seed_fx_norm(i, defaults[i]);
        }
        snapshot::mark_fx_synced();
    }
    // Read the name field's live buffer for painting (cloned so the scene borrow
    // below is free of the store).
    let (name_state, name_text, name_caret, name_anchor) = match ctx.host.store().get(AEDIT_NAME) {
        Some(InteractiveState::TextInput {
            state,
            text,
            caret,
            selection_anchor,
        }) => (*state, text.clone(), *caret, *selection_anchor),
        _ => (TextInputState::Normal, String::new(), 0, None),
    };

    let (scene, text_system) = (&mut *ctx.scene, &mut *ctx.text_system);
    let hit_index = ctx.host.hit_index_mut();

    // Clip name — an editable TextInput (mirror of the sprite name box). The
    // widget clips its own overflow to the field, so a long filename no longer
    // wraps/crams the header.
    let name_h = TypeToken::Sm.px() + Spacing::Sm.px() * 2.0;
    let name_rect = Rect::new(x, y, w, name_h);
    hit_index.register(AEDIT_NAME, name_rect);
    let input = TextInput::new(AEDIT_NAME, "")
        .placeholder("No clip loaded")
        .state(name_state);
    // Clip to the field: the TextInput lays its text out with word-wrap at the
    // inner width, so a long filename spills onto a 2nd line below the box. A clip
    // to the single-line box crops that overflow instead of letting it extrapolate.
    scene.push_clip(&rect_to_vello(name_rect));
    paint_text_input_with_buffer(
        &input,
        Some(name_text.as_str()),
        Some(name_caret),
        name_anchor,
        name_rect,
        scene,
        text_system,
        theme,
    );
    scene.pop_layer();
    y += name_h + Spacing::Sm.px();

    // Position / duration readout.
    let time_line = format!("{} / {}", fmt_time(pos), fmt_time(dur));
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
    let play_label = if playing { "Pause" } else { "Play" };
    toggle(
        Rect::new(x, y, w, ROW_H),
        play_label,
        playing,
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
        loaded,
        AEDIT_STOP,
        scene,
        text_system,
        theme,
        hit_index,
    );
    toggle(
        Rect::new(x + half + gap, y, half, ROW_H),
        "Loop",
        looping,
        AEDIT_LOOP,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += ROW_H + Spacing::Md.px();

    // Load | Export side by side.
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
        "Export\u{2026}",
        loaded,
        AEDIT_EXPORT,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += ROW_H + Spacing::Lg.px();

    // Edit ops (W2) — whole-clip, one-shot; each commits an undo step.
    paint_edit_section(
        y,
        x,
        w,
        loaded,
        undo_ok,
        redo_ok,
        has_sel,
        scene,
        text_system,
        theme,
        hit_index,
    );

    // Re-register the close button last so body widgets can't shadow it.
    ctx.host
        .hit_index_mut()
        .register(AEDIT_CLOSE, panel_close_button_rect(rect));
}

/// The Edit ops block: whole-clip (Undo/Redo · Normalize/LUFS · Reverse/DC ·
/// Gain−/Gain+ · Invert) then the selection range ops (Trim/Cut · Fade In/Out ·
/// Silence). Buttons dim when unavailable (no clip / no history / no selection).
#[allow(clippy::too_many_arguments)]
fn paint_edit_section(
    mut y: f32,
    x: f32,
    w: f32,
    loaded: bool,
    undo_ok: bool,
    redo_ok: bool,
    has_sel: bool,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
) {
    let gap = Spacing::Sm.px();
    let half = ((w - gap) * 0.5).max(1.0);
    // (label, id, enabled) pairs, laid out two-per-row (last row is single).
    let rows: [[(&str, NodeId, bool); 2]; 4] = [
        [("Undo", AEDIT_UNDO, undo_ok), ("Redo", AEDIT_REDO, redo_ok)],
        [
            ("Normalize", AEDIT_NORMALIZE, loaded),
            ("Norm LUFS", AEDIT_NORM_LUFS, loaded),
        ],
        [
            ("Reverse", AEDIT_REVERSE, loaded),
            ("Rm DC", AEDIT_DC, loaded),
        ],
        [
            ("Gain \u{2212}", AEDIT_GAIN_DOWN, loaded),
            ("Gain +", AEDIT_GAIN_UP, loaded),
        ],
    ];
    for row in rows {
        for (i, (label, id, enabled)) in row.into_iter().enumerate() {
            let bx = x + i as f32 * (half + gap);
            button(
                Rect::new(bx, y, half, ROW_H),
                label,
                enabled,
                id,
                scene,
                text_system,
                theme,
                hit_index,
            );
        }
        y += ROW_H + gap;
    }
    // Invert — full width.
    button(
        Rect::new(x, y, w, ROW_H),
        "Invert Polarity",
        loaded,
        AEDIT_INVERT,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += ROW_H + Spacing::Md.px();

    // Selection range ops — enabled only when a waveform selection exists (drag on
    // the overlay to make one).
    let range_rows: [[(&str, NodeId, bool); 2]; 2] = [
        [("Trim", AEDIT_TRIM, has_sel), ("Cut", AEDIT_CUT, has_sel)],
        [
            ("Fade In", AEDIT_FADE_IN, has_sel),
            ("Fade Out", AEDIT_FADE_OUT, has_sel),
        ],
    ];
    for row in range_rows {
        for (i, (label, id, enabled)) in row.into_iter().enumerate() {
            let bx = x + i as f32 * (half + gap);
            button(
                Rect::new(bx, y, half, ROW_H),
                label,
                enabled,
                id,
                scene,
                text_system,
                theme,
                hit_index,
            );
        }
        y += ROW_H + gap;
    }
    button(
        Rect::new(x, y, w, ROW_H),
        "Silence",
        has_sel,
        AEDIT_SILENCE,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += ROW_H + Spacing::Md.px();

    // Effects rack (W3 block 3a) — a selector + parameter sliders + Apply. It acts
    // on the selection, or the whole clip when there is none, so it only needs a
    // clip loaded (like the whole-clip ops above).
    crate::paint_fx::paint_fx_section(y, x, w, loaded, ROW_H, scene, text_system, theme, hit_index);
}

/// Seconds per minute — time-domain constant, not a UI metric.
const SECS_PER_MIN: f64 = 60.0; // LITERAL-PX-OK: seconds per minute (time math)

/// Format seconds as `m:ss.d` (one decimal), clamped at zero.
fn fmt_time(secs: f64) -> String {
    let s = secs.max(0.0);
    let m = (s / SECS_PER_MIN) as u64;
    let rem = s - (m as f64) * SECS_PER_MIN;
    format!("{m}:{rem:04.1}") // LITERAL-PX-OK: mm:ss.d time format spec, not a UI metric
}

/// A labeled action button: `Bg3` + `Text1` when enabled, dimmed to `Text2` when
/// not. Shared with the effects rack section (`paint_fx`).
///
/// A **disabled button does not register a hit rect**, so it cannot be clicked.
/// It used to register regardless ("disabled is a visual hint only"), which made
/// every dimmed control silently live: clicking the dimmed `Silence` with no
/// selection fell through to `target()` and zeroed the WHOLE clip (2026-07-09
/// audit). The panel dims and the seam refuses — two layers, since a dim alone is
/// cosmetic.
#[allow(clippy::too_many_arguments)]
pub(crate) fn button(
    rect: Rect,
    label: &str,
    enabled: bool,
    id: NodeId,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
) {
    let fg = if enabled {
        ColorToken::Text1
    } else {
        ColorToken::Text2
    };
    fill_rounded_rect(
        scene,
        rect,
        Radius::Sm.px(),
        resolve(ColorToken::Bg3, theme),
    );
    paint_text_centered(
        text_system,
        scene,
        label,
        rect,
        TypeToken::Sm.px(),
        resolve(fg, theme),
    );
    if enabled {
        hit_index.register(id, rect);
    }
}

/// A labeled toggle button: `Accent` tint + `AccentFg` when engaged, else `Bg3`
/// + `Text1`. Registers `id` as the hit rect.
#[allow(clippy::too_many_arguments)]
fn toggle(
    rect: Rect,
    label: &str,
    active: bool,
    id: NodeId,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
) {
    let (bg, fg) = if active {
        (ColorToken::Accent, ColorToken::AccentFg)
    } else {
        (ColorToken::Bg3, ColorToken::Text1)
    };
    fill_rounded_rect(scene, rect, Radius::Sm.px(), resolve(bg, theme));
    paint_text_centered(
        text_system,
        scene,
        label,
        rect,
        TypeToken::Sm.px(),
        resolve(fg, theme),
    );
    hit_index.register(id, rect);
}
