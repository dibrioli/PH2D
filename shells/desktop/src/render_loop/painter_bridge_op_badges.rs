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
    use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Point, Stroke};
    let map = |p: [f32; 2]| affine * Point::new(f64::from(p[0]), f64::from(p[1]));
    // Fluorescent yellow accent for the type-square glyph (reads on any canvas); parked frames are dimmer.
    let glyph_col = Color::new([1.0, 0.85, 0.15, 1.0]); // LITERAL-COLOR-OK: op-badge glyph
    let frame_col = Color::new([1.0, 0.85, 0.15, 0.35]); // LITERAL-COLOR-OK: parked-shape frame
    let scene = vector_scene.inner_mut();
    const R: f64 = 6.0; // glyph half-size (screen px)
    for b in &badges {
        // Parked shapes get a faint AABB frame so they read as still-selectable.
        if !b.active {
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
        }
        // The op glyph, drawn as vector geometry (no text) in the centre type-square.
        let c = map(b.center);
        match b.glyph {
            "+" => {
                let mut path = BezPath::new();
                path.move_to(Point::new(c.x - R, c.y));
                path.line_to(Point::new(c.x + R, c.y));
                path.move_to(Point::new(c.x, c.y - R));
                path.line_to(Point::new(c.x, c.y + R));
                scene.stroke(
                    &Stroke::new(2.0),
                    Affine::IDENTITY,
                    &Brush::Solid(glyph_col),
                    None,
                    &path,
                );
            }
            "-" => {
                let mut path = BezPath::new();
                path.move_to(Point::new(c.x - R, c.y));
                path.line_to(Point::new(c.x + R, c.y));
                scene.stroke(
                    &Stroke::new(2.0),
                    Affine::IDENTITY,
                    &Brush::Solid(glyph_col),
                    None,
                    &path,
                );
            }
            _ => {
                // Overlay "o" → a small ring.
                scene.stroke(
                    &Stroke::new(2.0),
                    Affine::IDENTITY,
                    &Brush::Solid(glyph_col),
                    None,
                    &Circle::new(c, R * 0.8),
                );
            }
        }
    }
}
