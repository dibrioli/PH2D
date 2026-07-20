//! The brush-cursor **ring** — the brush footprint (flatten+rotate ellipse) scaled to screen at the
//! cursor. Split out of `painter_bridge_overlays` for the HR-18 file-LOC cap. Pure draw. Skipped in
//! Selection mode (which shows a crosshair cursor instead — see `painter_bridge_selection_overlay`).

use ph2d_ecs::SimWorld;
use ph2d_editor::HeroScreen;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tool_painter::{DAB_FLATTEN_MAX, PainterTool};
use ph2d_vector::VectorScene;

/// Segments in the brush-cursor ellipse outline — enough that a flattened ellipse reads smooth.
const BRUSH_RING_SEGS: u32 = 64;

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_brush_ring(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
    cursor: (f32, f32),
) {
    // Selection mode has its own crosshair cursor (drawn by the selection overlay), not the brush ring.
    if painter.is_selection_mode() {
        return;
    }
    // ── Brush cursor ring (UI hint) ──────────────────────────────────
    // The brush radius (image px) scaled to screen at the cursor, while a
    // sprite is selected and the cursor is over the canvas (not a panel).
    // Uses the same sprite affine as the paint delivery, so the ring matches
    // where dabs land. Drawn into the overlay scene (composited over the
    // canvas this frame, like the rubber-band / bgremoval ring).
    if let Some(bits) = hero.gizmo.selection {
        let (cx, cy) = cursor;
        if hero.store.panel_at(cx, cy).is_none() {
            let bs = painter.brush_settings();
            let (iw, ih) = painter.canvas_size();
            let entity = ph2d_ecs::Entity::from_bits(bits);
            if iw > 0
                && let (Some(tr), Some(sprite)) = (
                    sim.world().get::<crate::Transform>(entity),
                    sim.world().get::<ph2d_render::Sprite>(entity),
                )
            {
                // FULL sprite affine, so the ring tracks resize / AR / rotation. The dab is a
                // flatten+rotate ELLIPSE in image space (`BrushSpec::dab_flatten` / `dab_angle_deg` —
                // the same footprint the engine paints); we sweep that ellipse's boundary in image px
                // and push each point through the affine's LINEAR part (the ring is cursor-anchored,
                // so no translation), so the ring matches exactly where the dabs land.
                let affine = super::bgremoval_preview::sprite_image_to_screen_affine(
                    iw,
                    ih,
                    tr,
                    sprite,
                    camera,
                    window_size,
                );
                let c = affine.as_coeffs();
                let scale = (c[0] * c[0] + c[1] * c[1]).sqrt();
                // Deform uses its OWN (round) brush footprint — the deform radius, no flatten/rotation — so
                // the ring shows the deform size, not the paint brush's (Enio 2026-07-04).
                let deform = painter.is_deform_mode();
                let footprint_px = if deform {
                    bs.deform_size_px
                } else {
                    bs.size_px
                };
                // Image-space major radius, floored so the ring stays visible at tiny zoom (the old
                // screen-space `.max(1px)`).
                let r = if scale > 0.0 && f64::from(footprint_px) * scale < 1.0 {
                    1.0 / scale
                } else {
                    f64::from(footprint_px)
                };
                // Minor axis = (1 − flatten) of the major, rotated by the dab's LIVE orientation rotor
                // (`BrushSettings::dab_rotor`): the brush Angle already turned by the stroke-follow
                // rotation, so with Shape Rake / Flow (or Grain Rake) the ring turns WITH THE STROKE in
                // real time, exactly like the tip it represents (Enio 2026-07-19). The rotor comes from
                // the engine's own heading — the ring never re-derives a direction of its own, so it
                // cannot point somewhere the paint does not. Deform is a plain, unrotated disc.
                let m = if deform {
                    1.0
                } else {
                    1.0 - f64::from(bs.dab_flatten.clamp(0.0, DAB_FLATTEN_MAX))
                };
                let rotor = if deform { [1.0, 0.0] } else { bs.dab_rotor };
                let (cos_a, sin_a) = (f64::from(rotor[0]), f64::from(rotor[1]));
                use ph2d_vector::{Affine, BezPath, Brush, Color, Point, Stroke};
                use std::f64::consts::TAU;
                let mut path = BezPath::new();
                for i in 0..BRUSH_RING_SEGS {
                    let (s, co) = (f64::from(i) * TAU / f64::from(BRUSH_RING_SEGS)).sin_cos();
                    // Ellipse boundary `(cosθ, m·sinθ)` rotated by +angle, scaled to image px …
                    let ix = (co * cos_a - m * s * sin_a) * r;
                    let iy = (co * sin_a + m * s * cos_a) * r;
                    // … then the affine's linear 2×2 (cols `[c0,c1]`,`[c2,c3]`) maps image→screen.
                    let p = Point::new(
                        f64::from(cx) + c[0] * ix + c[2] * iy,
                        f64::from(cy) + c[1] * ix + c[3] * iy,
                    );
                    if i == 0 {
                        path.move_to(p);
                    } else {
                        path.line_to(p);
                    }
                }
                path.close_path();
                // Light-grey ring (baked inline, like the rubber-band overlay's
                // colour — a follow-up can swap to a theme token / 2-tone).
                let color = Color::new([0.78, 0.78, 0.78, 0.85]); // LITERAL-COLOR-OK: overlay cursor
                vector_scene.inner_mut().stroke(
                    &Stroke::new(1.5),
                    Affine::IDENTITY,
                    &Brush::Solid(color),
                    None,
                    &path,
                );
            }
        }
    }
}
