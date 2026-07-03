//! **Isolated selection gizmos** rendering (ADR-0103 Am.2 v2). Draws EVERY editable selection shape's gizmo
//! at once (ellipse / polygon / freehand), in the Sprite-gizmo style (theme `Selection` box + `Accent`
//! rounded-square handles + `BorderEmph` outline, a touch darker) via the shared [`super::painter_bridge_gizmo`]
//! helpers — so selection gizmos read IDENTICALLY to the stroke shape gizmos, and NEVER touch the stroke
//! editors. `outline` is the boundary; `square_handles` are resize / anchor squares; `circle_handles` are the
//! rotate / sides cues.

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
    let scene = vector_scene.inner_mut();
    let accents = super::painter_bridge_gizmo::GIZMO_ACCENTS;
    for g in &gizmos {
        // Each gizmo gets a DISTINCT fluorescent accent (Mask-style) so overlapping gizmos never read the
        // same colour.
        let pal = super::painter_bridge_gizmo::palette_accent(
            hero.theme,
            accents[g.accent % accents.len()],
        );
        if g.outline.len() >= 2 {
            let pts: Vec<Point> = g.outline.iter().map(|&p| map(p)).collect();
            super::painter_bridge_gizmo::stroke_open(scene, &pts, &pal);
        }
        if g.frame_box.len() >= 2 {
            let pts: Vec<Point> = g.frame_box.iter().map(|&p| map(p)).collect();
            super::painter_bridge_gizmo::stroke_open(scene, &pts, &pal);
        }
        for &h in &g.square_handles {
            super::painter_bridge_gizmo::square_handle(scene, map(h), &pal);
        }
        for &h in &g.circle_handles {
            super::painter_bridge_gizmo::circle_handle(scene, map(h), &pal);
        }
        for &h in &g.diamond_handles {
            super::painter_bridge_gizmo::diamond_handle(scene, map(h), &pal);
        }
    }
}
