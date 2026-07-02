//! The Painter **Curve / Free Hand** editor overlay — the auto-smoothed spine, the draggable control
//! dots, and (for the selected anchor) its Bézier **tangent handles** (dots on stems off the point).
//! Split from `painter_bridge_overlays` for the HR-18 file-LOC cap. Pure draw: reads the active
//! `PainterTool` snapshot + camera and writes guide geometry into the overlay `VectorScene`; mutates
//! nothing. Called once per frame by `painter_bridge_overlays::draw_overlays` while the Painter is active.

use ph2d_ecs::SimWorld;
use ph2d_editor::HeroScreen;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tool_painter::PainterTool;
use ph2d_vector::VectorScene;

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_curve_overlay(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
    cursor: (f32, f32),
) {
    // ── Curve editor overlay (control dots + the auto-smoothed spine + the selected anchor's tangents) ──
    // Drawn while a Curve session is being EDITED, regardless of the cursor / panels — it's the editing
    // chrome, not a hover hint. Maps image px → screen via the SAME sprite affine as the paint delivery, so
    // the dots sit exactly on the painted curve under any transform.
    if let Some(bits) = hero.gizmo.selection
        && let Some(overlay) = painter.curve_overlay()
    {
        let (iw, ih) = painter.canvas_size();
        let entity = ph2d_ecs::Entity::from_bits(bits);
        if iw > 0
            && ih > 0
            && let (Some(tr), Some(sprite)) = (
                sim.world().get::<crate::Transform>(entity),
                sim.world().get::<ph2d_render::Sprite>(entity),
            )
        {
            // image-px → screen via the FULL sprite affine, so the handles ride scale / AR / rotation.
            let affine = super::bgremoval_preview::sprite_image_to_screen_affine(
                iw,
                ih,
                tr,
                sprite,
                camera,
                window_size,
            );
            use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Point, Stroke};
            let map = |p: [f32; 2]| affine * Point::new(f64::from(p[0]), f64::from(p[1]));
            let scene = vector_scene.inner_mut();
            // Transform gizmo — the Sprite-gizmo box + handles (theme tokens, a touch darker). Drawn FIRST
            // (under the spine + dots) so the editing geometry stays visually dominant. Corners (0..4) flip
            // to circles while rotating; edges (4..8) + the centre move handle (8) stay rounded squares.
            if let Some(gz) = overlay.transform_gizmo.as_ref() {
                super::painter_bridge_gizmo::draw_transform_gizmo(
                    scene, gz, affine, hero.theme, cursor,
                );
            }
            // Spine guide — the auto-smoothed curve through the control points.
            if overlay.spine.len() >= 2 {
                let mut path = BezPath::new();
                path.move_to(map(overlay.spine[0]));
                for &p in &overlay.spine[1..] {
                    path.line_to(map(p));
                }
                let guide = Color::new([0.55, 0.72, 1.0, 0.85]); // LITERAL-COLOR-OK: curve guide
                scene.stroke(
                    &Stroke::new(1.5),
                    Affine::IDENTITY,
                    &Brush::Solid(guide),
                    None,
                    &path,
                );
            }
            // Tangent handles of the selected anchor — thin teal stems with grabbable dots (orange when
            // dragged). Drawn UNDER the control dots so the anchor stays the visually dominant grab.
            if let Some(t) = &overlay.tangents {
                let stem = Color::new([0.40, 0.85, 0.85, 0.85]); // LITERAL-COLOR-OK: tangent stem
                let grab = Color::new([1.0, 0.62, 0.20, 1.0]); // LITERAL-COLOR-OK: grabbed tangent
                let a = map(t.anchor);
                for (handle, is_out) in [(t.in_handle, false), (t.out_handle, true)] {
                    let Some(h) = handle else { continue };
                    let hp = map(h);
                    let mut line = BezPath::new();
                    line.move_to(a);
                    line.line_to(hp);
                    scene.stroke(
                        &Stroke::new(1.0),
                        Affine::IDENTITY,
                        &Brush::Solid(stem),
                        None,
                        &line,
                    );
                    let grabbed = t.grabbed_out == Some(is_out);
                    let c = if grabbed { grab } else { stem };
                    let r = if grabbed { 5.0 } else { 3.5 };
                    scene.fill(
                        Fill::NonZero,
                        Affine::IDENTITY,
                        &Brush::Solid(c),
                        None,
                        &Circle::new(hp, r),
                    );
                }
            }
            // Control dots — the selected one larger + accented.
            let dot = Color::new([0.95, 0.95, 0.97, 0.95]); // LITERAL-COLOR-OK: curve point
            let sel = Color::new([1.0, 0.62, 0.20, 1.0]); // LITERAL-COLOR-OK: selected curve point
            for (i, &p) in overlay.points.iter().enumerate() {
                let is_sel = overlay.selected == Some(i);
                let r = if is_sel { 6.0 } else { 4.0 };
                let c = if is_sel { sel } else { dot };
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(c),
                    None,
                    &Circle::new(map(p), r),
                );
            }
        }
    }
}
