//! Painter on-canvas editing chrome — the brush cursor ring + the Curve / Circle / Polygon /
//! Stencil editor overlays — split from `painter_bridge.rs` for the HR-18 file-LOC cap. Pure draw:
//! reads the active `PainterTool` + selection + camera and writes guide geometry into the overlay
//! `VectorScene`; it mutates no tool or model state. Called once per frame by `painter_bridge::dispatch`
//! while the Painter tool is active (inside the same downcast block that owns `painter`).

use ph2d_ecs::SimWorld;
use ph2d_editor::HeroScreen;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tool_painter::PainterTool;
use ph2d_vector::VectorScene;

/// The image-space offsets (px) to draw a shape-editor overlay at. Currently just the geometry itself —
/// **a single continuous overlay** (Enio 2026-07-11): a VECTOR overlay can't wrap toroidally the way the
/// raster wash does (Repeat Image tiles the *wrapped* sprite), so per-tile copies "split" a shape crossing
/// the seam. One continuous overlay reads cleanly — it stays VISIBLE beyond the sprite (un-clipped) and is
/// still editable from any tile via the tool's pointer-wrap (`route_shape_pointer_multi`). This is the ONE
/// switch: a future toroidal-wrap overlay would re-populate the 3×3 offsets here. Shared by every
/// stroke-shape overlay (ellipse / polygon / line / op-badges); the curve overlay mirrors it inline.
pub(super) fn overlay_tile_offsets(_painter: &PainterTool, _iw: u32, _ih: u32) -> Vec<(f64, f64)> {
    vec![(0.0, 0.0)]
}

/// Draw every Painter editing overlay for the active tool into `vector_scene`.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_overlays(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
    text_system: &mut ph2d_text::TextSystem,
    cursor: (f32, f32),
) {
    // Wetness sheen FIRST — under the brush ring + editor guides (#12a).
    super::painter_bridge_wetness::draw_wetness_overlay(
        painter,
        hero,
        sim,
        camera,
        window_size,
        vector_scene,
    );
    super::painter_bridge_brush_ring::draw_brush_ring(
        painter,
        hero,
        sim,
        camera,
        window_size,
        vector_scene,
        cursor,
    );
    super::painter_bridge_curve_overlay::draw_curve_overlay(
        painter,
        hero,
        sim,
        camera,
        window_size,
        vector_scene,
        cursor,
    );
    draw_ellipse_overlay(painter, hero, sim, camera, window_size, vector_scene);
    super::painter_bridge_line_overlay::draw_line_overlay(
        painter,
        hero,
        sim,
        camera,
        window_size,
        vector_scene,
        text_system,
        cursor,
    );
    draw_polygon_overlay(painter, hero, sim, camera, window_size, vector_scene);
    // Multi-shape op badges — the `+`/`−`/`○` type-square glyph per shape + a frame per parked shape.
    super::painter_bridge_op_badges::draw_op_badges(
        painter,
        hero,
        sim,
        camera,
        window_size,
        vector_scene,
    );
    // Isolated SELECTION gizmos (ADR-0103 Am.2 v2) — every editable selection shape's gizmo at once.
    super::painter_bridge_selection_gizmos::draw_selection_gizmos(
        painter,
        hero,
        sim,
        camera,
        window_size,
        vector_scene,
        cursor,
    );
    // Deform Transform gizmo (Wave 2) — the whole-region bounding box, when Transform temperament is active.
    super::painter_bridge_deform_gizmo::draw_deform_gizmo(
        painter,
        hero,
        sim,
        camera,
        window_size,
        vector_scene,
        cursor,
    );
    draw_stencil_overlay(
        painter,
        hero,
        sim,
        camera,
        window_size,
        vector_scene,
        cursor,
    );
    draw_symmetry_overlay(painter, hero, sim, camera, window_size, vector_scene);
    super::painter_bridge_fill_overlay::draw_fill_cursor(painter, vector_scene, cursor);
}

/// Sync the painter's shape-editor grab tolerance to the LIVE camera, once per frame BEFORE the overlays
/// are generated. `shape_grab_tol_px` is otherwise refreshed only on a painter Down/Move/Up (never on a
/// zoom or a plain hover), so after zooming a finished shape the overlay draws its on-canvas handles
/// (Line Fillet/Chamfer, Curve, Stencil…) at the stale scale and the first grab snaps them to the new
/// one. Keeping it current every frame removes that snap. No-op without a selected sprite; the value
/// matches what the pointer path computes, so it never fights the on-Down refresh.
pub(super) fn refresh_shape_grab_tol(
    painter: &mut PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
) {
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
    painter.set_shape_grab_tol_px(
        crate::input_dispatch::painter_canvas_input::shape_grab_tol_from_affine(&affine),
    );
}

/// Discrete **symmetry** guides: a dashed mirror line (X / Y / custom) or N dashed radial spokes from
/// the centre, so the artist sees where strokes will be replicated. No-op unless symmetry is enabled
/// and a sprite is selected. Pure draw, like the rest of this module; mirrors the brush-ring affine so
/// the guides ride the sprite's scale / aspect / rotation exactly where the engine mirrors the dabs.
#[allow(clippy::too_many_arguments)]
fn draw_symmetry_overlay(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
) {
    let sym = painter.symmetry();
    if !sym.enabled {
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
    let map = |x: f64, y: f64| affine * Point::new(x, y);
    let scene = vector_scene.inner_mut();
    // Subtle light guide; dashed in SCREEN px (the path is already mapped, stroked under IDENTITY), so
    // the dash reads the same at any zoom.
    let color = Color::new([0.85, 0.85, 0.92, 0.5]); // LITERAL-COLOR-OK: subtle symmetry guide overlay
    let dash = Stroke::new(1.0).with_dashes(0.0, [5.0, 4.0]); // LITERAL-PX-OK: screen-px dash on/off run
    let cx = f64::from(sym.center[0]);
    let cy = f64::from(sym.center[1]);
    // Extend lines by the canvas diagonal so they always cross the whole sprite, whatever the centre.
    let span = (f64::from(iw) * f64::from(iw) + f64::from(ih) * f64::from(ih)).sqrt();
    if sym.circular {
        // N rotational sectors → N dashed spokes from the centre, `360/n` apart.
        use std::f64::consts::TAU;
        let n = sym.segments();
        for k in 0..n {
            let (s, co) = (f64::from(k) * TAU / f64::from(n)).sin_cos();
            let mut path = BezPath::new();
            path.move_to(map(cx, cy));
            path.line_to(map(cx + co * span, cy + s * span));
            scene.stroke(&dash, Affine::IDENTITY, &Brush::Solid(color), None, &path);
        }
    } else {
        // Mirror line through the centre along the axis direction, extended both ways.
        let d = sym.mirror_dir();
        let (dx, dy) = (f64::from(d[0]), f64::from(d[1]));
        let mut path = BezPath::new();
        path.move_to(map(cx - dx * span, cy - dy * span));
        path.line_to(map(cx + dx * span, cy + dy * span));
        scene.stroke(&dash, Affine::IDENTITY, &Brush::Solid(color), None, &path);
    }
}

/// **Repeat Image**: draw the painted composite repeated in the 8 neighbour positions around the
/// sprite (a 3×3 tile grid), so the artist sees the seamless tiling result. The centre is the real
/// sprite (drawn by the pipeline); we draw only the 8 wraps as overlay images, each abutting at the
/// sprite edges. No-op unless Repeat Image is on and a CPU composite for the selected sprite exists.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_repeat_image(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
    preview: Option<&crate::app_state::PainterPreview>,
) {
    if !painter.repeat_image() {
        return;
    }
    let Some(bits) = hero.gizmo.selection else {
        return;
    };
    // Need the CPU composite for THIS sprite (the GPU-only path leaves it `None`).
    let Some(preview) = preview.filter(|p| p.entity_bits == bits) else {
        return;
    };
    let entity = ph2d_ecs::Entity::from_bits(bits);
    let (Some(tr), Some(sprite)) = (
        sim.world().get::<crate::Transform>(entity),
        sim.world().get::<ph2d_render::Sprite>(entity),
    ) else {
        return;
    };
    // image-px → screen for the centre sprite; each neighbour prepends a screen-space translation of
    // the world offset (a pure translation maps through the world→screen scale `k`, Y flipped).
    let base = super::bgremoval_preview::sprite_image_to_screen_affine(
        preview.width,
        preview.height,
        tr,
        sprite,
        camera,
        window_size,
    );
    // Each neighbour is the same image translated by ±one image dimension in IMAGE-px space, so the
    // tile rides through `base`'s full transform (scale · rotation · anchor) — a screen-space offset
    // would shear off a rotated/scaled sprite. The central image (`base`) already includes everything.
    let (iw, ih) = (f64::from(preview.width), f64::from(preview.height));
    let (win_w, win_h) = (f64::from(window_size.width), f64::from(window_size.height));
    for dy in [-1i32, 0, 1] {
        for dx in [-1i32, 0, 1] {
            if dx == 0 && dy == 0 {
                continue; // the real sprite occupies the centre
            }
            let tile =
                base * ph2d_vector::Affine::translate((f64::from(dx) * iw, f64::from(dy) * ih));
            // Viewport-cull: each tile is a FULL-canvas blit, so 8/frame ≈ halves FPS when zoomed in
            // (the neighbours sit off-screen). Skip a tile whose screen-space bbox misses the window —
            // zero cost when the sprite fills the view (Enio 2026-06-26).
            let bb = tile.transform_rect_bbox(ph2d_vector::Rect::new(0.0, 0.0, iw, ih));
            if bb.x1 < 0.0 || bb.y1 < 0.0 || bb.x0 > win_w || bb.y0 > win_h {
                continue;
            }
            vector_scene.draw_image_rgba_transformed(
                &preview.rgba,
                preview.width,
                preview.height,
                tile,
                ph2d_vector::ImageQuality::Low,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_ellipse_overlay(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
) {
    // ── Circle editor overlay (ellipse outline + 4 axis handles + rotate + centre) ──
    // Same footprint mapping as the curve overlay; the handle indices match `EllipseOverlay`:
    // 0 right, 1 top, 2 left, 3 bottom, 4 rotate, 5 centre.
    if let Some(bits) = hero.gizmo.selection
        && let Some(overlay) = painter.ellipse_overlay()
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
            let base_affine = super::bgremoval_preview::sprite_image_to_screen_affine(
                iw,
                ih,
                tr,
                sprite,
                camera,
                window_size,
            );
            use ph2d_vector::{Affine, Point};
            // Ellipse stroke gizmo = fluorescent YELLOW (distinct stroke-shape accent).
            let pal = super::painter_bridge_gizmo::palette_accent(
                hero.theme,
                super::painter_bridge_gizmo::GIZMO_ACCENTS[0],
            );
            let op_glyph = painter.active_op_glyph();
            let scene = vector_scene.inner_mut();
            // Edit-in-tile: draw the gizmo in each visible wrapped tile too (`overlay_tile_offsets`).
            for (ox, oy) in overlay_tile_offsets(painter, iw, ih) {
                let affine = base_affine * Affine::translate((ox, oy));
                let map = |p: [f32; 2]| affine * Point::new(f64::from(p[0]), f64::from(p[1]));
                // Outline + handles in the Sprite-gizmo style: the axis + centre handles are rounded squares,
                // the rotate handle is a circle. Matches the selection gizmos.
                if overlay.perimeter.len() >= 2 {
                    let pts: Vec<Point> = overlay.perimeter.iter().map(|&p| map(p)).collect();
                    super::painter_bridge_gizmo::stroke_box(scene, &pts, &pal);
                }
                for (i, &h) in overlay.handles.iter().enumerate() {
                    let p = map(h);
                    if i == 4 {
                        super::painter_bridge_gizmo::circle_handle(scene, p, &pal);
                    } else if i == 5 && op_glyph.is_some() {
                        // Centre-move square (index 5) DOUBLED with the Operation glyph.
                        super::painter_bridge_gizmo::center_glyph_handle(
                            scene,
                            p,
                            &pal,
                            op_glyph.unwrap(),
                        );
                    } else {
                        super::painter_bridge_gizmo::square_handle(scene, p, &pal);
                    }
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_polygon_overlay(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
) {
    // ── Polygon editor overlay (N-gon outline + 4 axis + rotate + sides + centre) ──
    // Handle indices match `PolygonOverlay`: 0 right, 1 top, 2 left, 3 bottom, 4 rotate,
    // 5 sides (changes the side count), 6 centre.
    if let Some(bits) = hero.gizmo.selection
        && let Some(overlay) = painter.polygon_overlay()
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
            let base_affine = super::bgremoval_preview::sprite_image_to_screen_affine(
                iw,
                ih,
                tr,
                sprite,
                camera,
                window_size,
            );
            use ph2d_vector::{Affine, Point};
            // Polygon stroke gizmo = fluorescent PINK (distinct stroke-shape accent).
            let pal = super::painter_bridge_gizmo::palette_accent(
                hero.theme,
                super::painter_bridge_gizmo::GIZMO_ACCENTS[1],
            );
            let op_glyph = painter.active_op_glyph();
            let scene = vector_scene.inner_mut();
            // Edit-in-tile: draw the gizmo in each visible wrapped tile too (`overlay_tile_offsets`).
            for (ox, oy) in overlay_tile_offsets(painter, iw, ih) {
                let affine = base_affine * Affine::translate((ox, oy));
                let map = |p: [f32; 2]| affine * Point::new(f64::from(p[0]), f64::from(p[1]));
                // Sprite-gizmo style: outline box + axis/centre squares + the rotate & sides handles as
                // circles. Matches the selection gizmos.
                if overlay.perimeter.len() >= 2 {
                    let pts: Vec<Point> = overlay.perimeter.iter().map(|&p| map(p)).collect();
                    super::painter_bridge_gizmo::stroke_box(scene, &pts, &pal);
                }
                for (i, &h) in overlay.handles.iter().enumerate() {
                    let p = map(h);
                    match i {
                        4 => super::painter_bridge_gizmo::circle_handle(scene, p, &pal), // rotate
                        5 => super::painter_bridge_gizmo::diamond_handle(scene, p, &pal), // sides (distinct)
                        6 if op_glyph.is_some() => {
                            // Centre-move square (index 6) DOUBLED with the Operation glyph.
                            super::painter_bridge_gizmo::center_glyph_handle(
                                scene,
                                p,
                                &pal,
                                op_glyph.unwrap(),
                            );
                        }
                        _ => super::painter_bridge_gizmo::square_handle(scene, p, &pal),
                    }
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_stencil_overlay(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
    cursor: (f32, f32),
) {
    // ── Stencil texture overlay (rect outline + drag handles of the image-space mask) ──
    // The stencil is positioned/sized/rotated via its handles (corners = resize; the ring just outside
    // a corner = rotate, à la the sprite gizmo; centre = move) or the Texture / Stencil-card number
    // boxes. The outline shows where the mask lets paint through; while the user transforms the gizmo
    // or its params, the live Grain preview tiles inside it.
    if let Some(bits) = hero.gizmo.selection
        && let Some(overlay) = painter.stencil_overlay()
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
            use ph2d_vector::{Affine, Point};
            let c = affine.as_coeffs();
            let scale = (c[0] * c[0] + c[1] * c[1]).sqrt();
            let map = |p: [f32; 2]| affine * Point::new(f64::from(p[0]), f64::from(p[1]));
            // Live Grain preview INSIDE the rect (under the outline + handles). Rendered in the rect's
            // LOCAL frame; map buffer-px → image-px (centre ± half along the rect axes `u`/`v`) → screen.
            if let Some(prev) = painter.stencil_preview() {
                let u = prev.u;
                let v = [-u[1], u[0]];
                let (hx, hy) = (f64::from(prev.half[0]), f64::from(prev.half[1]));
                let (ax, ay) = (2.0 * hx / f64::from(prev.w), 2.0 * hx / f64::from(prev.w));
                let (bx, by) = (2.0 * hy / f64::from(prev.h), 2.0 * hy / f64::from(prev.h));
                let buf_to_img = Affine::new([
                    ax * f64::from(u[0]),
                    ay * f64::from(u[1]),
                    bx * f64::from(v[0]),
                    by * f64::from(v[1]),
                    f64::from(prev.center[0]) - hx * f64::from(u[0]) - hy * f64::from(v[0]),
                    f64::from(prev.center[1]) - hx * f64::from(u[1]) - hy * f64::from(v[1]),
                ]);
                vector_scene.draw_image_rgba_transformed(
                    &prev.rgba,
                    prev.w,
                    prev.h,
                    affine * buf_to_img,
                    ph2d_vector::ImageQuality::Low,
                );
            }
            let scene = vector_scene.inner_mut();
            // The Sprite-gizmo box + handles (theme tokens, a touch darker), so the Stencil rect reads like
            // the Sprite transform gizmo. Corners flip to circles as the rotate cue; the centre is a square.
            let pal = super::painter_bridge_gizmo::palette(hero.theme);
            let box_pts: Vec<Point> = overlay.corners.iter().map(|&p| map(p)).collect();
            super::painter_bridge_gizmo::stroke_box(scene, &box_pts, &pal);
            let inner = f64::from(overlay.scale_tol_px) * scale;
            let outer = f64::from(overlay.rotate_tol_px) * scale;
            let cur = Point::new(f64::from(cursor.0), f64::from(cursor.1));
            let center_sp = map(overlay.center);
            // The rotate cue matches the tool's hit-test: in the band just OUTSIDE a corner (farther from
            // the centre than the corner), so it doesn't light up for points inside the rect.
            let over_rotate = overlay.corners.iter().any(|&p| {
                let sp = map(p);
                let d = sp.distance(cur);
                d > inner && d <= outer && cur.distance(center_sp) > sp.distance(center_sp)
            });
            let draw_circle = overlay.rotating || over_rotate;
            for &p in &overlay.corners {
                let sp = map(p);
                if draw_circle {
                    super::painter_bridge_gizmo::circle_handle(scene, sp, &pal);
                } else {
                    super::painter_bridge_gizmo::square_handle(scene, sp, &pal);
                }
            }
            super::painter_bridge_gizmo::square_handle(scene, center_sp, &pal);
        }
    }
}
