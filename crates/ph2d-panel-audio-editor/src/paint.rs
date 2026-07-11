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
    AEDIT_BATCH_LUFS, AEDIT_CLOSE, AEDIT_EXPORT, AEDIT_FX_PARAMS, AEDIT_LOAD, AEDIT_LOOP,
    AEDIT_NAME, AEDIT_PANEL, AEDIT_PLAY, AEDIT_STOP, AudioEditorPanel, snapshot,
};
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_centered, rect_to_vello, resolve};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEADER_CLOSE_RESERVE, PANEL_TITLE_BASELINE, paint_panel_close_button,
    paint_panel_corner_dot, paint_panel_corner_dot_bl, paint_panel_surface, paint_panel_title,
    panel_close_button_rect,
};
use ph2d_editor_core::widget::{
    AUDIO_EDITOR_SCROLLBAR_ID, SCROLLBAR_W, TextInput, TextInputState, paint_scrollbar,
    paint_text_input_with_buffer, scrollbar_is_needed, scrollbar_thumb_rect, scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

pub(crate) use crate::clipped_hits::ClippedHits;

pub(crate) const ROW_H: f32 = 28.0; // LITERAL-PX-OK: transport button row height (chrome)
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

    // The body (transport + edit ops + the whole effects rack) overflows the dock
    // height, so it is clipped + scrolled. Reserve the scrollbar rail unconditionally
    // — a layout that reflows the instant the content overflows is worse than one
    // that always leaves the rail empty.
    let pad = Spacing::Lg.px();
    let rail = SCROLLBAR_W + Spacing::Sm.px();
    let x = rect.x + pad;
    let w = (rect.w - pad * 2.0 - rail).max(1.0);
    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let bottom_pad = Spacing::Lg.px();
    let body_h = (rect.y + rect.h - body_top - bottom_pad).max(0.0);
    let body_rect = Rect::new(rect.x, body_top, rect.w, body_h);
    // Read the scroll BEFORE the mutable store/hit borrows below.
    let scroll = ctx.host.store().panel_scroll(AEDIT_PANEL);
    let y = body_top - scroll;

    // Snapshot (shell → panel).
    let loaded = snapshot::loaded();
    let playing = snapshot::playing();
    let looping = snapshot::looping();
    let pos = snapshot::position_secs();
    let dur = snapshot::duration_secs();
    let undo_ok = snapshot::can_undo();
    let redo_ok = snapshot::can_redo();
    let has_sel = snapshot::has_selection();

    sync_widget_buffers(ctx);

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
    // Anything scrolled past the body's top/bottom is hidden — and, via
    // `ClippedHits`, unclickable.
    scene.push_clip(&rect_to_vello(body_rect));
    let hit_index = &mut ClippedHits::new(ctx.host.hit_index_mut(), body_rect);

    let name_box = NameBox {
        state: name_state,
        text: name_text,
        caret: name_caret,
        anchor: name_anchor,
    };
    let y = paint_transport_section(
        y,
        x,
        w,
        Transport {
            loaded,
            playing,
            looping,
            pos,
            dur,
        },
        &name_box,
        scene,
        text_system,
        theme,
        hit_index,
    );

    // Loop points (W6) — sits under the transport (a playback/asset concept). Set
    // from the selection, snap, audition click-free, and export to the `smpl` chunk.
    let y = crate::paint_loop::paint_loop_section(
        y,
        x,
        w,
        loaded,
        has_sel,
        ROW_H,
        scene,
        text_system,
        theme,
        hit_index,
    );
    // Markers (W6) — named cue points at the playhead, exported to `cue`+`adtl`.
    let y = crate::paint_loop::paint_markers_section(
        y,
        x,
        w,
        loaded,
        ROW_H,
        scene,
        text_system,
        theme,
        hit_index,
    );

    // Edit ops (W2) — whole-clip, one-shot; each commits an undo step. Then the
    // effects rack. `final_y` is the bottom of the painted content, still carrying
    // the `- scroll` offset.
    let final_y = crate::paint_edit::paint_edit_section(
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

    // Total scrollable height in body-local coords (undo the `- scroll` offset).
    let content_h = (final_y + scroll) - body_top + bottom_pad;
    scene.pop_layer();
    paint_scroll_chrome(ctx, body_rect, scroll, content_h, body_h, theme);

    // Re-register the close button last so body widgets can't shadow it.
    ctx.host
        .hit_index_mut()
        .register(AEDIT_CLOSE, panel_close_button_rect(rect));
}

/// The shell's live transport readout, bundled so the section fits one arg list.
struct Transport {
    loaded: bool,
    playing: bool,
    looping: bool,
    pos: f64,
    dur: f64,
}

/// The clip-name `TextInput`'s live buffer, cloned out of the store so the scene
/// borrow below is free of it.
struct NameBox {
    state: TextInputState,
    text: String,
    caret: usize,
    anchor: Option<usize>,
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

/// Push shell-owned values into the widget buffers the shared dispatch also writes.
/// Both guards exist to avoid fighting the user: overwrite the name box only when a
/// NEW clip loads (and never while it holds focus), and the parameter sliders only
/// when the PANEL moved them programmatically (kind switch, reset, add, select) —
/// never on a user drag.
fn sync_widget_buffers(ctx: &mut PaintCtx) {
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
    if let Some(norms) = snapshot::fx_sliders_need_sync() {
        for (i, id) in AEDIT_FX_PARAMS.iter().enumerate() {
            if let Some(InteractiveState::Slider { value, .. }) = ctx.host.store_mut().get_mut(*id)
            {
                *value = norms[i];
            }
        }
    }
}

/// The scrollbar (only when the content overflows), plus the `content_h`/`visible_h`
/// the wheel dispatcher needs to clamp against. Then clamp any leftover over-scroll —
/// the content shrinks whenever a chain stage is removed, and a stale offset would
/// leave the body parked past its own end.
fn paint_scroll_chrome(
    ctx: &mut PaintCtx,
    body_rect: Rect,
    scroll: f32,
    content_h: f32,
    body_h: f32,
    theme: Theme,
) {
    if scrollbar_is_needed(content_h, body_h) {
        let track = scrollbar_track_rect(body_rect);
        let thumb = scrollbar_thumb_rect(track, scroll, content_h, body_h);
        let is_active = matches!(
            ctx.host.store().scrollbar_drag(),
            Some(d) if d.panel == AEDIT_PANEL
        );
        paint_scrollbar(
            body_rect, scroll, content_h, body_h, is_active, ctx.scene, theme,
        );
        // The thumb sits on the rail OUTSIDE the body's widgets, and its drag is routed
        // by `scrollbar_panel_for_id`, not by an `InteractiveState` — so it registers on
        // the raw hit index, unclipped.
        ctx.host
            .hit_index_mut()
            .register(AUDIO_EDITOR_SCROLLBAR_ID, thumb);
    }
    let store = ctx.host.store_mut();
    store.set_panel_content_h(AEDIT_PANEL, content_h);
    store.set_panel_visible_h(AEDIT_PANEL, body_h);
    let max_scroll = (content_h - body_h).max(0.0);
    if store.panel_scroll(AEDIT_PANEL) > max_scroll {
        store.set_panel_scroll(AEDIT_PANEL, max_scroll);
    }
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
    hit_index: &mut ClippedHits,
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
/// + `Text1`.
///
/// Like [`button`], a **disabled toggle registers no hit rect** — it keeps its
/// surface (so an engaged-but-inert state still reads as engaged) but loses its
/// text contrast, and cannot be clicked.
#[allow(clippy::too_many_arguments)]
pub(crate) fn toggle(
    rect: Rect,
    label: &str,
    active: bool,
    enabled: bool,
    id: NodeId,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) {
    let (bg, fg) = match (active, enabled) {
        (true, true) => (ColorToken::Accent, ColorToken::AccentFg),
        (false, true) => (ColorToken::Bg3, ColorToken::Text1),
        _ => (ColorToken::Bg3, ColorToken::Text2),
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
    if enabled {
        hit_index.register(id, rect);
    }
}
