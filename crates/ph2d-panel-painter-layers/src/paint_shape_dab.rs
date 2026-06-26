//! The Shape section's **flatten + rotate** gizmo (Procreate Shape panel; Enio 2026-06-26). An
//! immobile template — a gray rim circle + crossed axes — with a black ellipse showing the flattened
//! dab, a dark **flatten** handle on the minor axis (drag toward the centre to flatten, out to the rim
//! for round) and a green **rotation** handle on the rim. Both handles are `CurvePoint`s under
//! [`core_ids::PAINTER_BRUSH_DAB_GIZMO`] (channel `0` = flatten radial, `1` = rotation); the panel
//! decodes the drag in [`crate::event::dab_gizmo`] and forwards the flatten / angle to the tool. The
//! deform is brush-wide — it flattens the falloff, the Shape silhouette and the View-Grain together.

use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{fill_circle, resolve, stroke_polyline};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Spacing};
use ph2d_tool_painter::BrushSettings;

const GIZMO_PX: f32 = 104.0; // LITERAL-PX-OK: the square gizmo extent
const HANDLE_R: f32 = 6.0; // LITERAL-PX-OK: draggable handle radius
const DOT_R: f32 = 3.0; // LITERAL-PX-OK: centre marker radius
const RIM_W: f32 = 2.0; // rim circle stroke (structural)
const AXIS_W: f32 = 1.0; // cross-axis stroke (structural)
const ELLIPSE_W: f32 = 2.0; // black ellipse stroke (structural)
/// Polyline segments approximating the rim + ellipse loops.
const SEGS: usize = 48;

/// Paint the flatten/rotate gizmo (a centred square), registering the two draggable handles. Returns
/// the next `y`.
pub(crate) fn paint_shape_dab_gizmo(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let size = GIZMO_PX.min(content_w.max(0.0));
    let gx = x + (content_w - size) * 0.5;
    let canvas = Rect::new(gx, y, size, size);
    let r = size * 0.5;
    let (cx, cy) = (gx + r, y + r);

    let rim = resolve(ColorToken::Border, theme);
    let axis = resolve(ColorToken::Bg3, theme);
    // Immobile template: the gray rim circle + the crossed axes.
    let circle: Vec<(f32, f32)> = (0..=SEGS)
        .map(|i| {
            let t = i as f32 / SEGS as f32 * std::f32::consts::TAU;
            (cx + r * t.cos(), cy + r * t.sin())
        })
        .collect();
    stroke_polyline(ctx.scene, &circle, RIM_W, rim);
    stroke_polyline(ctx.scene, &[(gx, cy), (gx + size, cy)], AXIS_W, axis);
    stroke_polyline(ctx.scene, &[(cx, y), (cx, y + size)], AXIS_W, axis);

    // The flattened, rotated ellipse: major axis = the rim radius along `angle`, minor = `1 - flatten`.
    let flatten = brush.dab_flatten.clamp(0.0, 1.0);
    let minor = r * (1.0 - flatten);
    let a = f32::from(brush.dab_angle_deg).to_radians();
    let (ca, sa) = (a.cos(), a.sin());
    let ellipse: Vec<(f32, f32)> = (0..=SEGS)
        .map(|i| {
            let t = i as f32 / SEGS as f32 * std::f32::consts::TAU;
            let (lx, ly) = (r * t.cos(), minor * t.sin());
            (cx + lx * ca - ly * sa, cy + lx * sa + ly * ca)
        })
        .collect();
    stroke_polyline(
        ctx.scene,
        &ellipse,
        ELLIPSE_W,
        resolve(ColorToken::Text1, theme),
    );

    // Handle positions: rotation on the rim (major-axis tip), flatten on the minor-axis tip.
    let rot_h = (cx + r * ca, cy + r * sa);
    let flat_h = (cx + minor * sa, cy - minor * ca);

    // Register the two handles as `CurvePoint`s (paint-time, factory ids) + their grab rects.
    {
        let store = ctx.host.store_mut();
        for (ch, _) in [(0u8, flat_h), (1u8, rot_h)] {
            store.register(
                core_ids::painter_brush_dab_handle_id(ch),
                InteractiveState::CurvePoint {
                    parent: core_ids::PAINTER_BRUSH_DAB_GIZMO,
                    channel: ch,
                    index: 0,
                    canvas,
                },
            );
        }
    }
    for (ch, (hx, hy)) in [(0u8, flat_h), (1u8, rot_h)] {
        ctx.host.hit_index_mut().register(
            core_ids::painter_brush_dab_handle_id(ch),
            Rect::new(hx - HANDLE_R, hy - HANDLE_R, HANDLE_R * 2.0, HANDLE_R * 2.0),
        );
    }

    // The handle dots + the red centre marker (painted last, on top).
    fill_circle(
        ctx.scene,
        flat_h.0,
        flat_h.1,
        HANDLE_R,
        resolve(ColorToken::Text2, theme),
    );
    fill_circle(
        ctx.scene,
        rot_h.0,
        rot_h.1,
        HANDLE_R,
        resolve(ColorToken::Success, theme),
    );
    fill_circle(ctx.scene, cx, cy, DOT_R, resolve(ColorToken::Danger, theme));

    y + size + Spacing::Sm.px()
}
