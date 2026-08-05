//! **The transient overlays of the graph canvas** — the in-progress wire ghost, the probe
//! readout, the knife stroke, the rubber band and the add-menu. Split from `paint` for the
//! 200-LOC/fn + 600-LOC/file caps; `super` is `paint`. Drawn over the cards and INSIDE the
//! canvas clip, under the split chrome.

use crate::geom::{self, View};
use crate::paint::{BAND_W, KNIFE_W, draw_menu, draw_wire_ghost, draws_wire_ghost};
use crate::state::{Interaction, MotionGraphPanelState};
use ph2d_editor_core::paint::{fill_rounded_rect, resolve, stroke_polyline, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Theme};

/// **The transient overlays, INSIDE the canvas clip** — split from `paint` for the 200-LOC/fn
/// cap. Everything here is drawn over the cards and under the split chrome, and every one of
/// them is a *gesture in flight* or a HUD: the in-progress wire ghost (it tracks a CAPTURED
/// pointer, which routinely leaves the panel), the probe readout, the knife stroke, the rubber
/// band, and the add-menu (already clamped on-canvas).
///
/// It registers no hits, by design: the menu's rows are hit-tested against the full-canvas
/// `Background` shield that `paint` pushes AFTER this call, and the other four are pure
/// feedback about a gesture the interaction layer already owns.
pub(super) fn draw_canvas_overlays(
    ctx: &mut PaintCtx,
    state: &MotionGraphPanelState,
    snap: &crate::snapshot::GraphViewSnapshot,
    view: &View,
    theme: Theme,
    rect: Rect,
) {
    if draws_wire_ghost(&state.interaction) {
        draw_wire_ghost(ctx, snap, state, view, theme);
    }
    // The probe readout, over the cards (it is a HUD, not part of the graph).
    if let Some(p) = &snap.probe
        && let Some(n) = snap.nodes.iter().find(|n| n.id == p.node)
    {
        crate::probe::draw(ctx, p, n, view, theme);
    }
    // The knife stroke — Danger, because it is one: what it crosses gets cut.
    if let Interaction::Knife { anchor, cur } = state.interaction {
        stroke_polyline(
            ctx.scene,
            &[anchor, cur],
            KNIFE_W,
            resolve(ColorToken::Danger, theme),
        );
    }
    // The rubber band (left-drag on empty canvas). Translucent fill — it is drawn
    // OVER the very cards it is selecting, and an opaque one would hide them.
    if let Interaction::BoxSelect { anchor, cur, .. } = state.interaction {
        let band = geom::band_rect(anchor, cur);
        fill_rounded_rect(
            ctx.scene,
            band,
            0.0,
            resolve(ColorToken::GraphMarquee, theme),
        );
        stroke_rounded_rect(
            ctx.scene,
            band,
            0.0,
            BAND_W,
            resolve(ColorToken::Accent, theme),
        );
    }
    if let Some(menu) = &state.menu {
        draw_menu(ctx, menu, snap, rect, theme);
    }
}
