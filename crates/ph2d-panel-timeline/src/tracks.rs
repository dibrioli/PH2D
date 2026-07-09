//! Track list (W2.E4) — the left label column of the dope-sheet: the "+Track"
//! dropdown (header row, aligned with the ruler strip) and one row per binding
//! (property label + a short object tag).
//!
//! "+Track" is a dropdown button; opening it lists the six properties as a
//! popover overlay. Picking one raises `PanelEvent::Click(<prop id>)`; the shell
//! binds the *currently selected* sprite's matching property (it owns the
//! selection) and closes the dropdown. Each row's time area paints its key
//! **diamonds** (selected keys in the accent colour), culled to the visible
//! span, and registers their [`TimelineHitKind`] hit targets (+ a `Lane`
//! background) so the dope-sheet gestures reach `interact` (E5b: click-select,
//! drag-move, clear-on-empty).

use ph2d_editor_core::interaction::{InteractiveState, TimelineHitKind};
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_text, rect_to_vello, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{Button, ButtonState, paint_button};
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::{PropKind, SelectedKey, TimelineViewSnapshot};
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::{Affine, BezPath, Brush, Fill, Stroke};

use crate::ids;

/// Width of the left label column (property + object tag).
pub(crate) const LABEL_COL_W: f32 = 132.0; // LITERAL-PX-OK: track-label column width
const DIAMOND_H: f32 = 4.5; // LITERAL-PX-OK: keyframe diamond half-size
/// Horizontal half-width of a key's clickable hit rect (larger than the visual
/// diamond so a small target is easy to grab).
const KEY_HIT_HW: f32 = 7.0; // LITERAL-PX-OK: keyframe grab half-width

/// Paint the "+ Track" dropdown button filling `header` (the label-column slice
/// aligned with the ruler strip). The property list opens as an overlay popover
/// (see [`paint_add_track_popover`], painted last).
pub(crate) fn paint_add_track(ctx: &mut PaintCtx, theme: Theme, header: Rect) {
    let state = ctx
        .host
        .store()
        .button_state(ids::TIMELINE_ADD_TRACK)
        .unwrap_or(ButtonState::Normal);
    let btn = Button::new(ids::TIMELINE_ADD_TRACK, "+ Track  \u{25be}").state(state);
    paint_button(&btn, header, ctx.scene, ctx.text_system, theme);
    ctx.host
        .hit_index_mut()
        .register(ids::TIMELINE_ADD_TRACK, header);
}

/// Paint the property dropdown as an overlay below `anchor` (the +Track button)
/// when open, and register each option's hit. Call LAST so it sits on top.
pub(crate) fn paint_add_track_popover(ctx: &mut PaintCtx, theme: Theme, anchor: Rect, open: bool) {
    if !open {
        return;
    }
    let n = ids::ADDPROP_BUTTONS.len() as f32;
    let list = Rect::new(anchor.x, anchor.y + anchor.h, anchor.w, ROW_H_PX * n);
    fill_rounded_rect(
        ctx.scene,
        list,
        Radius::Sm.px(),
        resolve(ColorToken::BgElev, theme),
    );
    stroke_rounded_rect(
        ctx.scene,
        list,
        Radius::Sm.px(),
        StrokeToken::Thin.px(),
        resolve(ColorToken::Border, theme),
    );
    let mut y = list.y;
    for (id, label) in ids::ADDPROP_BUTTONS {
        let r = Rect::new(list.x, y, list.w, ROW_H_PX);
        let state = ctx
            .host
            .store()
            .button_state(id)
            .unwrap_or(ButtonState::Normal);
        let btn = Button::new(id, label).state(state);
        paint_button(&btn, r, ctx.scene, ctx.text_system, theme);
        ctx.host.hit_index_mut().register(id, r);
        y += ROW_H_PX;
    }
}

/// Paint one row per binding in `region`: the label (left `label_w` column) and
/// the key diamonds in the time area (mapped by `time_x`/`view_start`/`px_per_s`,
/// culled to the visible span). Selected keys paint in the accent colour, shifted
/// by `preview_dx` px while a key drag is in flight.
///
/// Registers a [`TimelineHitKind::Lane`] background over the time area (click =
/// clear selection) and one [`TimelineHitKind::Key`] hit per visible diamond
/// (click = select, drag = move), so `dispatch` streams gestures the panel's
/// `interact` step drains. Hits are keyed by the key's *identity* (stable across
/// frames), not its position.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_rows(
    ctx: &mut PaintCtx,
    theme: Theme,
    region: Rect,
    label_w: f32,
    time_x: f32,
    view_start: f64,
    px_per_s: f64,
    preview_dx: f32,
    scroll_y: f32,
    snap: &TimelineViewSnapshot,
) {
    let right = region.x + region.w;
    let bottom = region.y + region.h;
    // Lane background over the time area — a click here clears the selection.
    let lane = Rect::new(time_x, region.y, (right - time_x).max(0.0), region.h);
    ctx.host.store_mut().register(
        ids::TIMELINE_LANES,
        InteractiveState::TimelineSurface {
            parent: ids::TIMELINE_PANEL,
            kind: TimelineHitKind::Lane,
            canvas: lane,
        },
    );
    ctx.host.hit_index_mut().register(ids::TIMELINE_LANES, lane);

    for (i, track) in snap.tracks.iter().enumerate() {
        // Rows scroll as a block; cull the ones outside the band entirely so a
        // long track list neither paints nor registers off-screen hits.
        let y = region.y - scroll_y + i as f32 * ROW_H_PX;
        if y + ROW_H_PX <= region.y || y >= bottom {
            continue;
        }
        // Zebra so lanes read as discrete rows.
        if i % 2 == 1 {
            fill_row(
                ctx,
                theme,
                Rect::new(region.x, y, region.w, ROW_H_PX),
                ColorToken::Bg2,
            );
        }
        let font = TypeToken::Sm.px();
        let text = format!("{}  #{}", prop_label(track.prop), track.entity % 10_000);
        let color = if track.missing {
            ColorToken::Danger
        } else {
            ColorToken::Text1
        };
        paint_text(
            ctx.text_system,
            ctx.scene,
            &text,
            region.x + Spacing::Xs.px(),
            y + (ROW_H_PX - font) * 0.5,
            font,
            label_w - Spacing::Xs.px() * 2.0,
            resolve(color, theme),
        );
        // Key diamonds + their hit targets.
        let cy = y + ROW_H_PX * 0.5;
        for k in &track.keys {
            let base_x = time_x + ((k.t_seconds - view_start) * px_per_s) as f32;
            // Selected keys ride the live drag preview.
            let kx = if k.selected {
                base_x + preview_dx
            } else {
                base_x
            };
            if kx < time_x - DIAMOND_H || kx > right + DIAMOND_H {
                continue;
            }
            let tok = if k.selected {
                ColorToken::Accent
            } else {
                ColorToken::Text1
            };
            paint_diamond(ctx, kx, cy, DIAMOND_H, resolve(tok, theme));
            // Hit target keyed by identity (stable across frames/reorders). The
            // grab rect follows the drawn (previewed) position.
            let id = ids::timeline_key_hit_id(track.target.get(), k.id.get());
            let hit = Rect::new(kx - KEY_HIT_HW, y, KEY_HIT_HW * 2.0, ROW_H_PX);
            ctx.host.store_mut().register(
                id,
                InteractiveState::TimelineSurface {
                    parent: ids::TIMELINE_PANEL,
                    kind: TimelineHitKind::Key {
                        target: track.target.get(),
                        key: k.id.get(),
                    },
                    canvas: lane,
                },
            );
            ctx.host.hit_index_mut().register(id, hit);
        }
    }
}

/// Every key whose diamond **centre** falls inside `sel` (a marquee in global
/// px). Mirrors the row/column math of [`paint_rows`] exactly — a key the user
/// can see inside the box is a key that gets selected.
///
/// Rows scrolled out of `rows` never match: `sel` is intersected with the band
/// first, so a marquee dragged past the bottom edge cannot reach into the
/// clipped rows below it.
pub(crate) fn keys_in_rect(
    rows: Rect,
    time_x: f32,
    view_start: f64,
    px_per_s: f64,
    scroll_y: f32,
    snap: &TimelineViewSnapshot,
    sel: Rect,
) -> Vec<SelectedKey> {
    let bottom = rows.y + rows.h;
    let (top_y, bot_y) = (sel.y.max(rows.y), (sel.y + sel.h).min(bottom));
    let mut out = Vec::new();
    if top_y > bot_y {
        return out;
    }
    for (i, track) in snap.tracks.iter().enumerate() {
        let y = rows.y - scroll_y + i as f32 * ROW_H_PX;
        let cy = y + ROW_H_PX * 0.5;
        if cy < top_y || cy > bot_y {
            continue;
        }
        for k in &track.keys {
            let kx = time_x + ((k.t_seconds - view_start) * px_per_s) as f32;
            if kx >= sel.x && kx <= sel.x + sel.w {
                out.push(SelectedKey::new(track.target.get(), k.id.get()));
            }
        }
    }
    out
}

/// The live box-select rubber band: marching-ants accent outline, no fill (so
/// the diamonds underneath stay readable). Painted over the rows.
pub(crate) fn paint_marquee(ctx: &mut PaintCtx, theme: Theme, rect: Rect) {
    let stroke = Stroke::new(StrokeToken::Thin.px() as f64).with_dashes(0.0, [4.0, 3.0]); // LITERAL-PX-OK: marching-ants dash pattern
    ctx.scene.inner_mut().stroke(
        &stroke,
        Affine::IDENTITY,
        &Brush::Solid(resolve(ColorToken::Accent, theme)),
        None,
        &rect_to_vello(rect),
    );
}

/// Fill a keyframe diamond centred at `(cx, cy)` with half-size `h`.
fn paint_diamond(ctx: &mut PaintCtx, cx: f32, cy: f32, h: f32, color: ph2d_vector::Color) {
    let (cx, cy, h) = (cx as f64, cy as f64, h as f64);
    let mut p = BezPath::new();
    p.move_to((cx, cy - h));
    p.line_to((cx + h, cy));
    p.line_to((cx, cy + h));
    p.line_to((cx - h, cy));
    p.close_path();
    ctx.scene.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(color),
        None,
        &p,
    );
}

/// The display label for a property (the panel's presentation of `PropKind`).
fn prop_label(p: PropKind) -> &'static str {
    match p {
        PropKind::TranslationX => "Translate X",
        PropKind::TranslationY => "Translate Y",
        PropKind::Rotation => "Rotation",
        PropKind::ScaleX => "Scale X",
        PropKind::ScaleY => "Scale Y",
        PropKind::Opacity => "Opacity",
    }
}

/// A subtle zebra fill for alternate rows.
fn fill_row(ctx: &mut PaintCtx, theme: Theme, rect: Rect, tok: ColorToken) {
    fill_rounded_rect(ctx.scene, rect, Radius::Xs.px(), resolve(tok, theme));
}
