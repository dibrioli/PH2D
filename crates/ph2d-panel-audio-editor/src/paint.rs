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
    AEDIT_CLOSE, AEDIT_EXPORT, AEDIT_LOAD, AEDIT_LOOP, AEDIT_NAME, AEDIT_PANEL, AEDIT_PLAY,
    AEDIT_STOP, AudioEditorPanel, snapshot,
};
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::{HitIndex, InteractiveState};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_centered, resolve};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEADER_CLOSE_RESERVE, PANEL_HEADER_H_DEFAULT, PANEL_TITLE_BASELINE,
    paint_panel_close_button, paint_panel_corner_dot, paint_panel_corner_dot_bl,
    paint_panel_surface, paint_panel_title, panel_close_button_rect, panel_drag_handle_rect,
    panel_resize_handle_rect, panel_resize_handle_rect_bl,
};
use ph2d_editor_core::widget::{TextInput, TextInputState, paint_text_input_with_buffer};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

const ROW_H: f32 = 28.0; // LITERAL-PX-OK: transport button row height (chrome)

pub(crate) fn paint(_state: &mut AudioEditorState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(AudioEditorPanel::ID) {
        ctx.host.store_mut().clear_panel_rect(AEDIT_PANEL);
        return;
    }
    let rect: Rect = ctx.layout.inspector;
    let theme = ctx.host.theme();
    ctx.host.store_mut().set_panel_rect(AEDIT_PANEL, rect);

    // Opaque backing (the shared dock slot's glass surface would bleed the
    // Inspector behind it otherwise).
    fill_rounded_rect(
        ctx.scene,
        rect,
        Radius::Sm.px(),
        resolve(ColorToken::BgElev, theme),
    );
    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);
    paint_panel_corner_dot_bl(rect, ctx.scene, theme);

    // Shared dock drag/resize handles (Inspector right-dock canon).
    {
        let drag_rect =
            panel_drag_handle_rect(rect, PANEL_HEADER_H_DEFAULT, PANEL_HEADER_CLOSE_RESERVE);
        let resize_rect = panel_resize_handle_rect(rect);
        let resize_bl_rect = panel_resize_handle_rect_bl(rect);
        let hit_index = ctx.host.hit_index_mut();
        hit_index.register(core_ids::INSP_DRAG_HANDLE, drag_rect);
        hit_index.register(core_ids::INSP_RESIZE_HANDLE, resize_rect);
        hit_index.register(core_ids::INSP_RESIZE_HANDLE_BL, resize_bl_rect);
    }

    let title_size = paint_panel_title(
        rect,
        "Audio Editor",
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    paint_panel_close_button(rect, AEDIT_CLOSE, ctx.host.hit_index_mut(), ctx.scene, theme);

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

    // Re-register the close button last so body widgets can't shadow it.
    ctx.host
        .hit_index_mut()
        .register(AEDIT_CLOSE, panel_close_button_rect(rect));
}

/// Format seconds as `m:ss.d` (one decimal), clamped at zero.
fn fmt_time(secs: f64) -> String {
    let s = secs.max(0.0);
    let m = (s / 60.0) as u64;
    let rem = s - (m as f64) * 60.0;
    format!("{m}:{rem:04.1}")
}

/// A labeled action button: `Bg3` + `Text1` when enabled, dimmed to `Text2`
/// when not. Registers `id` as the hit rect regardless (disabled is a visual
/// hint only in W1).
#[allow(clippy::too_many_arguments)]
fn button(
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
    fill_rounded_rect(scene, rect, Radius::Sm.px(), resolve(ColorToken::Bg3, theme));
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
