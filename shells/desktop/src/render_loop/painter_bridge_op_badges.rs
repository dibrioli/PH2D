//! Multi-shape **op badges** overlay (Enio 2026-07-04): for every stroke shape currently on canvas, draw
//! its Operation type-square in the gizmo centre — the `+` (Add) / `−` (Remove) / `○` (Overlay) glyph — and,
//! for each PARKED (inactive-but-editable) shape, a faint AABB frame so the user sees it is still a shape
//! they can click to re-edit. Pure draw: reads `PainterTool::stroke_op_badges()` + camera, mutates nothing.
//! Split from `painter_bridge_overlays` for the HR-18 file-LOC cap.

use ph2d_ecs::SimWorld;
use ph2d_editor::HeroScreen;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tool_painter::PainterTool;
use ph2d_vector::VectorScene;

pub(super) fn draw_op_badges(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
) {
    let badges = painter.stroke_op_badges();
    if badges.is_empty() {
        return;
    }
    let Some(bits) = hero.gizmo.selection else {
        return;
    };
    let (iw, ih) = painter.canvas_size();
    if iw == 0 || ih == 0 {
        return;
    }
    let entity = ph2d_ecs::Entity::from_bits(bits);
    let (Some(tr), Some(sprite)) = (
        sim.world().get::<crate::Transform>(entity),
        sim.world().get::<ph2d_render::Sprite>(entity),
    ) else {
        return;
    };
    let affine = super::bgremoval_preview::sprite_image_to_screen_affine(
        iw,
        ih,
        tr,
        sprite,
        camera,
        window_size,
    );
    use ph2d_vector::{Affine, BezPath, Brush, Color, Point, Stroke};
    let map = |p: [f32; 2]| affine * Point::new(f64::from(p[0]), f64::from(p[1]));
    // Parked shapes have no live gizmo, so they get a faint AABB frame (so they read as still-selectable)
    // + the SAME doubled centre square + op glyph the active gizmo draws (`center_glyph_handle`).
    let frame_col = Color::new([1.0, 0.85, 0.15, 0.35]); // LITERAL-COLOR-OK: parked-shape frame
    let pal = super::painter_bridge_gizmo::palette_accent(
        hero.theme,
        super::painter_bridge_gizmo::GIZMO_ACCENTS[0],
    );
    let scene = vector_scene.inner_mut();
    for b in &badges {
        let p0 = map([b.bbox[0], b.bbox[1]]);
        let p1 = map([b.bbox[2], b.bbox[1]]);
        let p2 = map([b.bbox[2], b.bbox[3]]);
        let p3 = map([b.bbox[0], b.bbox[3]]);
        let mut frame = BezPath::new();
        frame.move_to(p0);
        frame.line_to(p1);
        frame.line_to(p2);
        frame.line_to(p3);
        frame.close_path();
        scene.stroke(
            &Stroke::new(1.0),
            Affine::IDENTITY,
            &Brush::Solid(frame_col),
            None,
            &frame,
        );
        super::painter_bridge_gizmo::center_glyph_handle(scene, map(b.center), &pal, b.glyph);
    }
}
