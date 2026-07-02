//! The Painter **Line** polyline editor overlay — the segments through the committed corner points, a dot
//! at each corner (the selected one emphasised), the whole-line transform gizmo, and the per-corner CAD
//! gizmos (a CIRCLE = Fillet, a SQUARE = Chamfer). Split from `painter_bridge_overlays` for the HR-18
//! file-LOC cap, mirroring `painter_bridge_curve_overlay`. Pure draw: reads the active `PainterTool`
//! snapshot + camera and writes guide geometry into the overlay `VectorScene`; mutates nothing.

use ph2d_ecs::SimWorld;
use ph2d_editor::HeroScreen;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tool_painter::PainterTool;
use ph2d_vector::VectorScene;

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_line_overlay(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
    cursor: (f32, f32),
) {
    if let Some(bits) = hero.gizmo.selection
        && let Some(overlay) = painter.line_overlay()
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
            let affine = super::bgremoval_preview::sprite_image_to_screen_affine(
                iw,
                ih,
                tr,
                sprite,
                camera,
                window_size,
            );
            use ph2d_vector::{
                Affine, BezPath, Brush, Circle, Color, Fill, Point, RoundedRect, Stroke,
            };
            let map = |p: [f32; 2]| affine * Point::new(f64::from(p[0]), f64::from(p[1]));
            let scene = vector_scene.inner_mut();
            // Transform gizmo (editing phase) — drawn FIRST (under the segments + dots) so the editing
            // geometry stays visually dominant. Identical to the Curve gizmo (shared helper).
            if let Some(gz) = overlay.transform_gizmo.as_ref() {
                super::painter_bridge_gizmo::draw_transform_gizmo(
                    scene, gz, affine, hero.theme, cursor,
                );
            }
            let guide = Color::new([0.55, 0.72, 1.0, 0.85]); // LITERAL-COLOR-OK: line guide
            // Segments through the committed corner points.
            let pts = &overlay.points;
            if pts.len() >= 2 {
                let mut path = BezPath::new();
                path.move_to(map(pts[0]));
                for &p in &pts[1..] {
                    path.line_to(map(p));
                }
                if overlay.closed && overlay.points.len() >= 3 {
                    path.close_path();
                }
                scene.stroke(
                    &Stroke::new(1.5),
                    Affine::IDENTITY,
                    &Brush::Solid(guide),
                    None,
                    &path,
                );
            }
            // A white dot at each committed corner; the SELECTED corner emphasised (purple, larger).
            let dot = Color::new([0.95, 0.95, 0.97, 0.95]); // LITERAL-COLOR-OK: corner dot
            let sel = Color::new([0.72, 0.45, 0.95, 1.0]); // LITERAL-COLOR-OK: selected corner
            for (i, &p) in overlay.points.iter().enumerate() {
                let selected = overlay.selected == Some(i);
                let (c, r) = if selected { (sel, 5.5) } else { (dot, 4.0) };
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(c),
                    None,
                    &Circle::new(map(p), r),
                );
            }
            // Per-corner CAD gizmos: a CIRCLE (Fillet) + a SQUARE (Chamfer) handle at each real corner;
            // the active mod is accented (orange). Shapes carry the meaning — round = round the corner,
            // square = straight bevel.
            let handle = Color::new([0.80, 0.84, 0.92, 0.92]); // LITERAL-COLOR-OK: corner handle
            let hot = Color::new([1.0, 0.62, 0.20, 1.0]); // LITERAL-COLOR-OK: active corner mod
            let edge = Color::new([0.12, 0.13, 0.16, 0.9]); // LITERAL-COLOR-OK: handle outline
            // Same visual size as the transform gizmo's square handles (half-extent 6 px → 12 px).
            for g in &overlay.corner_gizmos {
                let fc = if g.active == 1 { hot } else { handle };
                let fillet = Circle::new(map(g.fillet_handle), 6.0);
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(fc),
                    None,
                    &fillet,
                );
                scene.stroke(
                    &Stroke::new(1.0),
                    Affine::IDENTITY,
                    &Brush::Solid(edge),
                    None,
                    &fillet,
                );
                let s = map(g.chamfer_handle);
                let cc = if g.active == 2 { hot } else { handle };
                let sq = RoundedRect::new(s.x - 6.0, s.y - 6.0, s.x + 6.0, s.y + 6.0, 2.0);
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(cc),
                    None,
                    &sq,
                );
                scene.stroke(
                    &Stroke::new(1.0),
                    Affine::IDENTITY,
                    &Brush::Solid(edge),
                    None,
                    &sq,
                );
            }
        }
    }
}
