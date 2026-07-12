//! Transport bar paint (W2.E2) — the row of Play/Pause · Prev/Next-frame ·
//! seconds+frame chips · Loop/Auto-key/Snap toggles, laid out left→right at the
//! top of the panel body.
//!
//! Each control is painted from the SHARED widget source of truth and its hit
//! rect registered so the generic dispatch routes clicks/edits into
//! `event::apply_event`. The display state (play vs pause glyph, chip values,
//! toggle on/off) is mirrored from the frame's [`TimelineViewSnapshot`] into the
//! store each frame (when not focused), so the panel stays a pure view while the
//! store still drives editing.

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{paint_text, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::showcase::read_number_input;
use ph2d_editor_core::widget::{
    Button, ButtonState, Dropdown, DropdownOption, DropdownState, IconButtonStyle, IconGlyph,
    NumberInput, Toggle, ToggleState, paint_button, paint_dropdown_chip, paint_icon_button,
    paint_number_input_with_buffer, paint_toggle,
};
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::{DEFAULT_FPS, TimelineViewSnapshot};
use ph2d_tokens::{ColorToken, Density, ROW_H_PX, Radius, Spacing, StrokeToken, Theme, TypeToken};

use crate::ids;

const BTN_W: f32 = 30.0; // LITERAL-PX-OK: square transport icon-button
const ADD_MARKER_W: f32 = 40.0; // LITERAL-PX-OK: "+M" add-marker button width
const CHIP_W: f32 = 72.0; // LITERAL-PX-OK: seconds/frame number chip width
const CHIP_LABEL_W: f32 = 48.0; // LITERAL-PX-OK: "Time(s)"/"Frames" chip-label column
const TOGGLE_LABEL_W: f32 = 52.0; // LITERAL-PX-OK: "AutoKey" label column
const CLIP_DD_W: f32 = 108.0; // LITERAL-PX-OK: clip dropdown chip width

/// Paint the transport row inside `body` (top-aligned).
///
/// Returns the `y` below the row, and — when the clip dropdown is OPEN — the
/// chip's rect. The caller paints the popover LAST, after the dope sheet, or the
/// list would be drawn under the rows it overlaps
/// ([[feedback_overlay_cut_at_boundary_check_draw_order]]).
pub(crate) fn paint_bar(
    ctx: &mut PaintCtx,
    theme: Theme,
    body: Rect,
    snap: &TimelineViewSnapshot,
    speed_view: bool,
) -> (f32, Option<Rect>) {
    let gap = Spacing::Sm.px();
    let y = body.y;
    let mut x = body.x;

    // ── clip selector (W5) ───────────────────────────────────────────────────
    // Far left, ahead of the transport — the clip is the thing everything else on
    // this bar acts ON, so it reads first (and it is where Unity's animation window
    // puts it).
    let (after_clips, clip_chip) = clip_cluster(ctx, theme, x, y, snap);
    x = after_clips + gap;

    // ── transport buttons ────────────────────────────────────────────────────
    // |◀ ◀ ▶/⏸ ▶ ▶| — jump to start, step back, play/pause, step forward, jump
    // to end. The skip glyphs bracket the frame-steppers, as every transport does.
    x = icon_button(ctx, theme, x, y, ids::TIMELINE_GO_START, IconId::SkipBack) + gap * 0.5;
    x = icon_button(
        ctx,
        theme,
        x,
        y,
        ids::TIMELINE_PREV_FRAME,
        IconId::ChevronLeft,
    ) + gap * 0.5;
    let play_glyph = if snap.playing {
        IconId::Pause
    } else {
        IconId::Play
    };
    x = icon_button(ctx, theme, x, y, ids::TIMELINE_PLAY, play_glyph) + gap * 0.5;
    x = icon_button(
        ctx,
        theme,
        x,
        y,
        ids::TIMELINE_NEXT_FRAME,
        IconId::ChevronRight,
    ) + gap * 0.5;
    x = icon_button(ctx, theme, x, y, ids::TIMELINE_GO_END, IconId::SkipForward) + gap;

    // ── seconds + frame chips ────────────────────────────────────────────────
    let fps = if snap.fps > 0.0 {
        snap.fps
    } else {
        DEFAULT_FPS
    };
    label(
        ctx,
        theme,
        ph2d_i18n::tr("panel.timeline.time_seconds"),
        x,
        y,
        CHIP_LABEL_W,
    );
    x += CHIP_LABEL_W + gap * 0.5;
    x = chip(
        ctx,
        theme,
        x,
        y,
        ids::TIMELINE_TIME_NUM,
        snap.time_seconds,
        1.0 / fps,
        2,
    ) + gap;
    label(
        ctx,
        theme,
        ph2d_i18n::tr("panel.timeline.frame"),
        x,
        y,
        CHIP_LABEL_W,
    );
    x += CHIP_LABEL_W + gap * 0.5;
    x = chip(
        ctx,
        theme,
        x,
        y,
        ids::TIMELINE_FRAME_NUM,
        snap.frame as f64,
        1.0,
        0,
    ) + gap;

    // ── toggles ──────────────────────────────────────────────────────────────
    x = toggle(
        ctx,
        theme,
        x,
        y,
        ids::TIMELINE_LOOP,
        ph2d_i18n::tr("panel.timeline.loop"),
        snap.loop_range.is_some(),
    ) + gap;
    x = toggle(
        ctx,
        theme,
        x,
        y,
        ids::TIMELINE_AUTOKEY,
        ph2d_i18n::tr("panel.timeline.autokey"),
        snap.auto_key,
    ) + gap;
    // Record / performing (W5) — records the pose LIVE while playing + dragging,
    // sits next to AutoKey (its paused counterpart).
    x = toggle(
        ctx,
        theme,
        x,
        y,
        ids::TIMELINE_RECORD,
        ph2d_i18n::tr("panel.timeline.record"),
        snap.performing,
    ) + gap;
    x = toggle(
        ctx,
        theme,
        x,
        y,
        ids::TIMELINE_SNAP,
        ph2d_i18n::tr("panel.timeline.snap"),
        snap.frame_snap,
    ) + gap;
    // Speed-graph view (W5) — panel-local, NOT a document command: flips every
    // expanded graph band between the value curve and the velocity curve.
    x = toggle(
        ctx,
        theme,
        x,
        y,
        ids::TIMELINE_SPEED,
        ph2d_i18n::tr("panel.timeline.speed"),
        speed_view,
    ) + gap;

    // "+M" — drop a marker at the playhead (W4.T3). Same one-click authoring as
    // pressing M, for discoverability.
    add_marker_button(ctx, theme, x, y);

    (y + ROW_H_PX + Spacing::Sm.px(), clip_chip)
}

/// The clip cluster: `[ Main ▾ ] [+] [✎] [🗑]`.
///
/// Returns the `x` after it, and the chip rect when the dropdown is open (the
/// caller defers the popover paint — see [`paint_bar`]).
///
/// The TRASH is not painted, and — the part that matters —  **not hit-registered**,
/// while the document holds a single clip: a document must always have one to edit,
/// and a dimmed button that still dispatches is a click that silently does nothing
/// ([[feedback_disabled_button_still_dispatches]]).
fn clip_cluster(
    ctx: &mut PaintCtx,
    theme: Theme,
    x: f32,
    y: f32,
    snap: &TimelineViewSnapshot,
) -> (f32, Option<Rect>) {
    let gap = Spacing::Sm.px();
    let mut x = x;

    let chip = Rect::new(x, y, CLIP_DD_W, ROW_H_PX);
    ctx.host
        .hit_index_mut()
        .register(ids::TIMELINE_CLIP_DD, chip);
    let (state, open) = match ctx.host.store().get(ids::TIMELINE_CLIP_DD) {
        Some(InteractiveState::Dropdown { state, open, .. }) => (*state, *open),
        _ => (DropdownState::Normal, false),
    };
    let dd = Dropdown::new(ids::TIMELINE_CLIP_DD, "", clip_options(snap))
        .selected(snap.active_clip)
        .open(open)
        .state(state);
    paint_dropdown_chip(&dd, chip, ctx.scene, ctx.text_system, theme);
    x += CLIP_DD_W + gap * 0.5;

    x = icon_button(ctx, theme, x, y, ids::TIMELINE_ADD_CLIP, IconId::Plus) + gap * 0.5;
    x = icon_button(ctx, theme, x, y, ids::TIMELINE_RENAME_CLIP, IconId::Modify) + gap * 0.5;
    if snap.clips.len() > 1 {
        x = icon_button(ctx, theme, x, y, ids::TIMELINE_DELETE_CLIP, IconId::Trash);
    }

    (x, open.then_some(chip))
}

/// One dropdown option per clip: the VALUE is the clip's index (what the dispatch
/// needs) and the label is its name (what the animator reads). Truncated at the id
/// array — a clip past it could be painted but never clicked, so
/// `ph2d_timeline::MAX_CLIPS` refuses to create one and a gate holds the two equal.
pub(crate) fn clip_options(snap: &TimelineViewSnapshot) -> Vec<DropdownOption<usize>> {
    snap.clips
        .iter()
        .enumerate()
        .take(ids::TIMELINE_CLIP_OPT.len())
        .map(|(i, name)| DropdownOption::new(ids::TIMELINE_CLIP_OPT[i], i, name.clone()))
        .collect()
}

/// Paint the "+M" add-marker button + register its hit. `+` matches the
/// "+Track" convention (the affordance that ADDS something).
fn add_marker_button(ctx: &mut PaintCtx, theme: Theme, x: f32, y: f32) {
    let rect = Rect::new(x, y, ADD_MARKER_W, ROW_H_PX);
    let state = ctx
        .host
        .store()
        .button_state(ids::TIMELINE_ADD_MARKER)
        .unwrap_or(ButtonState::Normal);
    let btn = Button::new(
        ids::TIMELINE_ADD_MARKER,
        ph2d_i18n::tr("panel.timeline.add_marker"),
    )
    .state(state);
    paint_button(&btn, rect, ctx.scene, ctx.text_system, theme);
    ctx.host
        .hit_index_mut()
        .register(ids::TIMELINE_ADD_MARKER, rect);
}

/// Paint one square icon-button; register its hit rect. Returns the right edge.
fn icon_button(
    ctx: &mut PaintCtx,
    theme: Theme,
    x: f32,
    y: f32,
    id: ph2d_a11y::NodeId,
    glyph: IconId,
) -> f32 {
    let rect = Rect::new(x, y, BTN_W, ROW_H_PX);
    let state = ctx
        .host
        .store()
        .button_state(id)
        .unwrap_or(ButtonState::Normal);
    paint_icon_button(
        rect,
        IconGlyph::Builtin(glyph),
        IconButtonStyle::Chip,
        state,
        ctx.scene,
        theme,
    );
    ctx.host.hit_index_mut().register(id, rect);
    x + BTN_W
}

/// Paint one number chip (value mirrored from the snapshot when unfocused);
/// register its `[0, ∞)` range + hit. Returns the right edge.
#[allow(clippy::too_many_arguments)]
fn chip(
    ctx: &mut PaintCtx,
    theme: Theme,
    x: f32,
    y: f32,
    id: ph2d_a11y::NodeId,
    value: f64,
    step: f64,
    decimals: usize,
) -> f32 {
    let rect = Rect::new(x, y, CHIP_W, ROW_H_PX);
    {
        let store = ctx.host.store_mut();
        mirror_number(store, id, value, decimals);
        store.set_number_range(id, 0.0, f64::from(u16::MAX), step);
    }
    let (state, _v, buf, caret, anchor) = read_number_input(ctx.host.store(), id);
    let buf = buf.to_string();
    let input = NumberInput::new(id, "", value).step(step).state(state);
    paint_number_input_with_buffer(
        &input,
        Some(&buf),
        caret,
        anchor,
        rect,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    ctx.host.hit_index_mut().register(id, rect);
    x + CHIP_W
}

/// Paint a `label | switch` toggle in the exact Widget-Gallery form: a label
/// column then a `TypeToken::Xl3`-wide, `Density::Compact`-tall pill switch
/// (proper 2:1 track + thumb), vertically centred in the row. Register the
/// switch hit. Returns the right edge.
fn toggle(
    ctx: &mut PaintCtx,
    theme: Theme,
    x: f32,
    y: f32,
    id: ph2d_a11y::NodeId,
    text: &str,
    on: bool,
) -> f32 {
    let sw = TypeToken::Xl3.px();
    let sh = Density::Compact.row_h_px();
    let pad = Spacing::Xs.px();
    // Outlined cell grouping [label | switch] so each toggle is demarcated from
    // its neighbours (Enio 2026-07-08).
    let cell_w = pad + TOGGLE_LABEL_W + pad + sw + pad;
    let cell = Rect::new(x, y, cell_w, ROW_H_PX);
    stroke_rounded_rect(
        ctx.scene,
        cell,
        Radius::Sm.px(),
        StrokeToken::Thin.px(),
        resolve(ColorToken::Border, theme),
    );
    label(ctx, theme, text, x + pad, y, TOGGLE_LABEL_W);
    let sx = x + pad + TOGGLE_LABEL_W + pad;
    let rect = Rect::new(sx, y + (ROW_H_PX - sh) * 0.5, sw, sh);
    // Mirror the snapshot's on-state into the store (when not focused) so the
    // painted switch reflects the document and the edit baseline stays correct.
    {
        let store = ctx.host.store_mut();
        if store.focus_id() != Some(id)
            && let Some(InteractiveState::Toggle { on: store_on, .. }) = store.get_mut(id)
        {
            *store_on = on;
        }
    }
    let state = ctx
        .host
        .store()
        .toggle(id)
        .map(|(s, _)| s)
        .unwrap_or(ToggleState::Normal);
    // Gallery builder form (label rendered separately, above).
    let widget = Toggle::new(id, "").on(on).state(state);
    paint_toggle(&widget, rect, ctx.scene, theme);
    ctx.host.hit_index_mut().register(id, rect);
    x + cell_w
}

/// A short left-aligned, vertically-centred label of width `w`.
fn label(ctx: &mut PaintCtx, theme: Theme, text: &str, x: f32, y: f32, w: f32) {
    let font = TypeToken::Sm.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        text,
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        w,
        resolve(ColorToken::Text2, theme),
    );
}

/// Mirror a committed numeric `value` into the store's chip when it isn't being
/// edited (mirror of the Inspector/painter number-field sync).
fn mirror_number(store: &mut WidgetStore, id: ph2d_a11y::NodeId, value: f64, decimals: usize) {
    if store.focus_id() == Some(id) {
        return;
    }
    let text = if decimals == 0 {
        format!("{}", value.round() as i64)
    } else {
        format!("{value:.decimals$}")
    };
    if let Some(InteractiveState::NumberInput {
        value: v,
        buffer,
        caret,
        last_committed,
        ..
    }) = store.get_mut(id)
    {
        *v = value;
        buffer.clear();
        buffer.push_str(&text);
        *caret = buffer.len();
        *last_committed = value;
    }
}
