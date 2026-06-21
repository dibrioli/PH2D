//! The Brush Falloff curve graph — the visual representation of the active
//! falloff profile, and the editor for the `Custom` preset.
//!
//! Mirrors Blender's Falloff panel: a curve graph is always shown (so picking
//! Smooth / Sphere / Sharp / … visibly reshapes it — the "visual representation
//! of the falloff curves"), and when the `Custom` preset is selected the graph
//! becomes editable — draggable control points + add/remove buttons, exactly the
//! affordance the adjustment-layer Curves editor uses ([`crate::paint_adjust`]).
//!
//! For presets the polyline samples the engine's formula
//! ([`ph2d_tool_painter::brush_falloff_weight_at`]); for `Custom` it samples the
//! same editable curve the dab evaluates, so the graph always matches the brush.
//! The control points reuse the foundational `CurvePoint` 2-D drag dispatch
//! (`interaction/dispatch/curve.rs`): each handle registers an
//! `InteractiveState::CurvePoint` carrying the plotting canvas; the dispatch
//! stashes the normalized `(distance, strength)` on the store; `event.rs` drains
//! it and forwards `PAINTER_BRUSH_FALLOFF_EDIT` to the tool.

use crate::paint::register_button;
use ph2d_editor_core::ids::{self as core_ids, painter_brush_falloff_point_id};
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, paint_text_centered, resolve, stroke_polyline,
    stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, TypeToken};
use ph2d_tool_painter::{BrushSettings, Falloff, brush_falloff_weight_at};

const CANVAS_H: f32 = 96.0; // LITERAL-PX-OK: falloff curve-graph canvas height
const HANDLE_R: f32 = 4.0; // LITERAL-PX-OK: control-point handle radius
const GRAB_R: f32 = 9.0; // LITERAL-PX-OK: half-size of a handle's pointer grab box
const STROKE_W: f32 = 1.5; // LITERAL-PX-OK: plotted-curve stroke width
const GRID_W: f32 = 1.0; // LITERAL-PX-OK: grid stroke width
const BTN_W: f32 = 22.0; // LITERAL-PX-OK: +/− point-button width
const RING_W: f32 = 1.5; // LITERAL-PX-OK: handle ring outline width
const MAX_POINTS: usize = 8; // contract cap (matches ph2d_painter_brush::MAX_FALLOFF_POINTS)
const GRID_DIVS: usize = 4; // LITERAL-PX-OK: quarter grid divisions

/// Paint the Falloff curve graph below the preset dropdown. Always draws the
/// active profile's shape; when the `Custom` preset is selected it also draws the
/// +/− buttons and the draggable control points. Returns the next `y`.
pub(crate) fn paint_falloff_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let gap = Spacing::Xs.px();
    let is_custom = brush.falloff == Falloff::Custom.to_u8();
    let mut y = y;

    // ── (Custom only) +/− point buttons, right-aligned above the canvas ──
    if is_custom {
        let add_rect = Rect::new(x + content_w - BTN_W * 2.0 - gap, y, BTN_W, ROW_H_PX);
        let rem_rect = Rect::new(x + content_w - BTN_W, y, BTN_W, ROW_H_PX);
        for (brect, label, id) in [
            (add_rect, "+", core_ids::PAINTER_BRUSH_FALLOFF_ADD),
            (rem_rect, "−", core_ids::PAINTER_BRUSH_FALLOFF_REMOVE),
        ] {
            fill_rounded_rect(
                ctx.scene,
                brect,
                Radius::Sm.px(),
                resolve(ColorToken::Bg2, theme),
            );
            paint_text_centered(
                ctx.text_system,
                ctx.scene,
                label,
                brect,
                TypeToken::Base.px(),
                resolve(ColorToken::Text1, theme),
            );
            register_button(ctx.host.store_mut(), id);
            ctx.host.hit_index_mut().register(id, brect);
        }
        y += ROW_H_PX + gap;
    }

    // ── Canvas: background + border + quarter grid ──
    let canvas = Rect::new(x, y, content_w.max(1.0), CANVAS_H);
    fill_rounded_rect(
        ctx.scene,
        canvas,
        Radius::Sm.px(),
        resolve(ColorToken::Bg2, theme),
    );
    stroke_rounded_rect(
        ctx.scene,
        canvas,
        Radius::Sm.px(),
        GRID_W,
        resolve(ColorToken::TextDisabled, theme),
    );
    let grid = resolve(ColorToken::GridLine, theme);
    for i in 1..GRID_DIVS {
        let f = i as f32 / GRID_DIVS as f32;
        let gx = canvas.x + f * canvas.w;
        let gy = canvas.y + f * canvas.h;
        stroke_polyline(
            ctx.scene,
            &[(gx, canvas.y), (gx, canvas.y + canvas.h)],
            GRID_W,
            grid,
        );
        stroke_polyline(
            ctx.scene,
            &[(canvas.x, gy), (canvas.x + canvas.w, gy)],
            GRID_W,
            grid,
        );
    }

    // ── Plot the active falloff profile (x = distance 0→1, y = strength 1→0) ──
    let samples = (canvas.w * 0.5).clamp(48.0, 256.0) as usize; // LITERAL-PX-OK: sample-count clamp
    let mut poly: Vec<(f32, f32)> = Vec::with_capacity(samples + 1);
    for k in 0..=samples {
        let t = k as f32 / samples as f32;
        let w = brush_falloff_weight_at(&brush, t).clamp(0.0, 1.0);
        poly.push((canvas.x + t * canvas.w, canvas.y + (1.0 - w) * canvas.h));
    }
    // Accent while editable (Custom), muted for a read-only preset preview.
    let curve_color = resolve(
        if is_custom {
            ColorToken::Accent
        } else {
            ColorToken::Text2
        },
        theme,
    );
    stroke_polyline(ctx.scene, &poly, STROKE_W, curve_color);

    // ── (Custom only) draggable control points over the same canvas ──
    if is_custom {
        let ring = resolve(ColorToken::Accent, theme);
        let fill = resolve(ColorToken::Text1, theme);
        let n = (brush.falloff_len as usize).min(MAX_POINTS);
        for index in 0..n {
            let [px, py] = brush.falloff_points[index];
            let id = painter_brush_falloff_point_id(index as u8);
            let cx = canvas.x + px.clamp(0.0, 1.0) * canvas.w;
            let cy = canvas.y + (1.0 - py.clamp(0.0, 1.0)) * canvas.h;
            // Overwrite each frame so the carried `canvas` tracks panel resizes
            // (CurvePoint has no per-frame drag state — the result lands in the
            // store's `curve_point_drag` slot).
            ctx.host.store_mut().register(
                id,
                InteractiveState::CurvePoint {
                    parent: core_ids::PAINTER_BRUSH_FALLOFF_EDIT,
                    channel: 0,
                    index: index as u8,
                    canvas,
                },
            );
            let grab = Rect::new(cx - GRAB_R, cy - GRAB_R, GRAB_R * 2.0, GRAB_R * 2.0);
            ctx.host.hit_index_mut().register(id, grab);
            fill_circle(ctx.scene, cx, cy, HANDLE_R + RING_W, ring);
            fill_circle(ctx.scene, cx, cy, HANDLE_R, fill);
        }
    }

    y + CANVAS_H + Spacing::Sm.px()
}
