//! **Isolated selection gizmos** rendering (ADR-0103 Am.2 v2). Draws EVERY editable selection shape's gizmo
//! at once, all STANDARDISED to the Sprite transform gizmo: the shape outline (thin) inside an oriented
//! bounding box with 8 scale squares (corners + edge mids), a centre-move square, and — only for a Polygon —
//! the sides diamond. A square reads as a **circle** when the cursor is in its rotate ring (the rotate cue).
//! Each shape gets a DISTINCT fluorescent accent. Reuses the shared [`super::painter_bridge_gizmo`] helpers.

use ph2d_ecs::SimWorld;
use ph2d_editor::HeroScreen;
use ph2d_host::WindowSize;
use ph2d_render::{Camera2d, Sprite};
use ph2d_tool_painter::PainterTool;
use ph2d_vector::{Point, VectorScene};

use crate::Transform;

pub(super) fn draw_selection_gizmos(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
    cursor: (f32, f32),
) {
    let gizmos = painter.selection_gizmos();
    if gizmos.is_empty() {
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
        sim.world().get::<Transform>(entity),
        sim.world().get::<Sprite>(entity),
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
    let map = |p: [f32; 2]| affine * Point::new(f64::from(p[0]), f64::from(p[1]));
    // Image-px → screen per-pixel scale (for the rotate-ring hover distances).
    let scale = {
        let c = affine.as_coeffs();
        (c[0] * c[0] + c[1] * c[1]).sqrt()
    };
    let cur = Point::new(f64::from(cursor.0), f64::from(cursor.1));
    let accents = super::painter_bridge_gizmo::GIZMO_ACCENTS;
    let scene = vector_scene.inner_mut();
    for g in &gizmos {
        // Each gizmo gets a DISTINCT fluorescent accent so overlapping gizmos never read the same colour.
        let pal = super::painter_bridge_gizmo::palette_accent(
            hero.theme,
            accents[g.accent % accents.len()],
        );
        // Thin shape outline (ellipse / polygon / freehand) inside the box.
        if g.outline.len() >= 2 {
            let pts: Vec<Point> = g.outline.iter().map(|&p| map(p)).collect();
            super::painter_bridge_gizmo::stroke_open(scene, &pts, &pal);
        }
        // Oriented bounding box (closed) through the four corners.
        let box_pts: Vec<Point> = g.box_corners.iter().map(|&p| map(p)).collect();
        super::painter_bridge_gizmo::stroke_box(scene, &box_pts, &pal);
        // 8 scale squares — each reads as a CIRCLE when the cursor is in its rotate ring (band just outside).
        let center_sp = map(g.center);
        let inner = f64::from(g.scale_tol) * scale;
        let outer = f64::from(g.rotate_tol) * scale;
        for &h in &g.scale_handles {
            let sp = map(h);
            let d = sp.distance(cur);
            let in_rotate_ring =
                d > inner && d <= outer && cur.distance(center_sp) > sp.distance(center_sp);
            if in_rotate_ring {
                super::painter_bridge_gizmo::circle_handle(scene, sp, &pal);
            } else {
                super::painter_bridge_gizmo::square_handle(scene, sp, &pal);
            }
        }
        // Centre move square + polygon sides diamond.
        super::painter_bridge_gizmo::square_handle(scene, center_sp, &pal);
        if let Some(d) = g.diamond {
            super::painter_bridge_gizmo::diamond_handle(scene, map(d), &pal);
        }
    }
}
