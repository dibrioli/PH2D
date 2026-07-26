//! The **Buffer-Curves chips** (§5, the Unreal A/B toggle) — the per-band Store /
//! Swap buttons in the graph editor. Split from `graph_paint` under the panel LOC
//! cap; a CHILD module (`use super::*`) so it shares the parent's paint imports and
//! band geometry. The ghost of the buffered curve itself is drawn by `graph_paint`
//! (it is a curve, and `buffer_ghost.is_some()` is the one fact that both draws the
//! ghost AND shows the Swap chip here).

use super::*;
// The two names the parent no longer uses itself (they moved here), so `use super::*`
// no longer provides them: import them directly.
use ph2d_editor_core::interaction::BufferAction;
use ph2d_editor_core::paint::paint_text_centered;

/// Buffer-curve chip: a fixed-width box so "Store" and "Swap" align; both words fit
/// at the Xs type token.
const BUF_BTN_W: f32 = 38.0; // LITERAL-PX-OK: buffer chip width
const BUF_BTN_H: f32 = 15.0; // LITERAL-PX-OK: buffer chip height
const BUF_BTN_GAP: f32 = 4.0; // LITERAL-PX-OK: buffer chip gap + band inset

/// The per-band Buffer-Curves chips (§5), top-right of the graph band: **Store**
/// always, **Swap** only on the track that owns the buffer (its `buffer_ghost` is
/// `Some`). Painting Swap on non-owning rows would be a no-op under the mouse; the
/// one-fact rule (`buffer_ghost.is_some()`) keeps chip and ghost in lockstep.
///
/// A CLICK surface each ([`TimelineHitKind::GraphBufferButton`]) — the panel's
/// `interact::dispatch_primary` turns the Click into a Store/Swap intent, exactly
/// as the twirl's click becomes an expand. Chips stack RIGHT-to-left from the
/// band's right edge so Store keeps its place whether or not Swap is shown.
pub(super) fn paint_buffer_buttons(
    ctx: &mut PaintCtx,
    theme: Theme,
    band_rect: Rect,
    track: &TrackView,
) {
    let target = track.target.get();
    let top = band_rect.y + BUF_BTN_GAP;
    let mut right = band_rect.x + band_rect.w - BUF_BTN_GAP;
    if track.buffer_ghost.is_some() {
        right = paint_buffer_chip(ctx, theme, "Swap", right, top, target, BufferAction::Swap);
    }
    paint_buffer_chip(ctx, theme, "Store", right, top, target, BufferAction::Store);
}

/// Paint one buffer chip right-aligned at `right`, register its click surface, and
/// return the `right` for the next chip to its left. The Swap chip tints accent to
/// signal the live A/B buffer (derived from `action`, never a 2nd param); hover
/// brightens the border (via `hot_id`).
fn paint_buffer_chip(
    ctx: &mut PaintCtx,
    theme: Theme,
    label: &str,
    right: f32,
    top: f32,
    target: u64,
    action: BufferAction,
) -> f32 {
    let accent = matches!(action, BufferAction::Swap);
    let rect = Rect::new(right - BUF_BTN_W, top, BUF_BTN_W, BUF_BTN_H);
    let id = ids::timeline_buffer_button_id(target, action as u8);
    let hot = ctx.host.store().hot_id() == Some(id);
    fill_rounded_rect(
        ctx.scene,
        rect,
        Radius::Xs.px(),
        resolve(ColorToken::BgElev, theme),
    );
    stroke_rounded_rect(
        ctx.scene,
        rect,
        Radius::Xs.px(),
        StrokeToken::Thin.px(),
        resolve(
            if hot {
                ColorToken::Accent
            } else {
                ColorToken::Border
            },
            theme,
        ),
    );
    let text = if accent {
        ColorToken::Accent
    } else {
        ColorToken::Text2
    };
    paint_text_centered(
        ctx.text_system,
        ctx.scene,
        label,
        rect,
        TypeToken::Xs.px(),
        resolve(text, theme),
    );
    ctx.host.store_mut().register(
        id,
        InteractiveState::TimelineSurface {
            parent: ids::TIMELINE_PANEL,
            kind: TimelineHitKind::GraphBufferButton { target, action },
            canvas: rect,
        },
    );
    ctx.host.hit_index_mut().register(id, rect);
    rect.x - BUF_BTN_GAP
}
