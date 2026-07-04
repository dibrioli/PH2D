//! **Deform Transform gizmo** rendering (Deform Wave 2). When the Painter is in Deform mode with the
//! Transform temperament active, draws the whole-region bounding box: an oriented box with 8 scale squares
//! (corners + edge mids) + a centre-move square. A square reads as a **circle** when the cursor is in its
//! rotate ring (the rotate cue), exactly like the Sprite / selection gizmos. Reuses the shared
//! [`super::painter_bridge_gizmo`] helpers + the sprite→screen affine.

use ph2d_ecs::SimWorld;
use ph2d_editor::HeroScreen;
use ph2d_host::WindowSize;
use ph2d_render::{Camera2d, Sprite};
use ph2d_tool_painter::PainterTool;
use ph2d_vector::{Point, VectorScene};

use crate::Transform;

pub(super) fn draw_deform_gizmo(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
    cursor: (f32, f32),
) {
    let Some(g) = painter.deform_gizmo() else {
        return;
    };
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
    // A single fluorescent accent (the first) — there's only ever one Transform gizmo.
    let accents = super::painter_bridge_gizmo::GIZMO_ACCENTS;
    let pal = super::painter_bridge_gizmo::palette_accent(hero.theme, accents[0]);
    let scene = vector_scene.inner_mut();
    // Warp mesh: draw the grid lines connecting adjacent control points + a handle at each point.
    if let Some((cols, rows, pts)) = &g.mesh {
        let idx = |r: u32, c: u32| (r * (cols + 1) + c) as usize;
        for r in 0..=*rows {
            for c in 0..=*cols {
                let p = map(pts[idx(r, c)]);
                if c < *cols {
                    super::painter_bridge_gizmo::stroke_open(
                        scene,
                        &[p, map(pts[idx(r, c + 1)])],
                        &pal,
                    );
                }
                if r < *rows {
                    super::painter_bridge_gizmo::stroke_open(
                        scene,
                        &[p, map(pts[idx(r + 1, c)])],
                        &pal,
                    );
                }
            }
        }
        for &p in pts {
            super::painter_bridge_gizmo::square_handle(scene, map(p), &pal);
        }
        return;
    }
    // Oriented transform box / distort quad (closed).
    let box_pts: Vec<Point> = g.box_corners.iter().map(|&p| map(p)).collect();
    super::painter_bridge_gizmo::stroke_box(scene, &box_pts, &pal);
    if g.corner_only {
        // Distort: only the 4 corners are draggable (perspective) — no edges / rotate / centre.
        for &c in &g.box_corners {
            super::painter_bridge_gizmo::square_handle(scene, map(c), &pal);
        }
        return;
    }
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
    // Centre-move square.
    super::painter_bridge_gizmo::square_handle(scene, center_sp, &pal);
}
