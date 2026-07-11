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
    draw_wetness_overlay(painter, hero, sim, camera, window_size, vector_scene);
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

/// #12a (doc 14): the on-canvas WETNESS sheen — a subtle cool tint over the wet paper (Rebelle
/// "show wetness"), alpha ∝ the local moisture byte. Built over the moisture RECT only (transient,
/// session-scoped) and drawn via the same image→screen affine as the sprite. Read-only.
fn draw_wetness_overlay(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
) {
    // Wetness-preview strength (Wetness card slider): 0 ⇒ no preview.
    let intensity = painter.wet_preview_intensity();
    if intensity <= 0.0 {
        return;
    }
    let Some((wet, cw, ch, [rx0, ry0, rx1, ry1])) = painter.canvas_wet_view() else {
        return;
    };
    let Some(bits) = hero.gizmo.selection else {
        return;
    };
    let entity = ph2d_ecs::Entity::from_bits(bits);
    let (Some(tr), Some(sprite)) = (
        sim.world().get::<crate::Transform>(entity),
        sim.world().get::<ph2d_render::Sprite>(entity),
    ) else {
        return;
    };
    let (rw, rh) = ((rx1 - rx0) as usize, (ry1 - ry0) as usize);
    if rw == 0 || rh == 0 {
        return;
    }
    // Straight-alpha DAMP-PAPER darkening over the wet region (Enio 2026-07-11: "sem tonalidade
    // azulada" — wet paper just darkens, no water-blue sheen). Near-neutral dark tint, a hair warm so it
    // reads as damp paper, not a grey wash; the slider (`intensity`) scales the max veil alpha. Vello
    // premultiplies on draw.
    const TINT: [u8; 3] = [34, 31, 28]; // LITERAL-COLOR-OK: damp-paper darkening (near-neutral, faint warm)
    const BLUR_R: usize = 8; // LITERAL-PX-OK: veil softening radius (canvas px) — wet paper has no hard moisture edges
    let max_alpha = intensity * 0.55; // LITERAL-COLOR-OK: slider 0..1 → veil alpha 0..0.55 at full wetness
    let cwu = cw as usize;
    // Local moisture → veil alpha, then a separable box blur. A stroke re-wetting a RECEDING wash pours
    // a fresh 255 patch against the neighbour's decayed (edges-first) moisture — a sharp rectangular step
    // the raw per-pixel veil printed as the "retângulo na união" (Enio 2026-07-11, confirmado com Preview 0).
    // Blurring the alpha reads it as a soft damp gradient (+ a soft organic fringe past the wet edge),
    // never a hard patch. Cosmetic-only: the moisture map itself is untouched.
    let mut alpha = vec![0.0f32; rw * rh];
    for y in 0..rh {
        let src = (ry0 as usize + y) * cwu + rx0 as usize;
        let dst = y * rw;
        for x in 0..rw {
            alpha[dst + x] = f32::from(wet[src + x]) / 255.0 * max_alpha;
        }
    }
    let alpha = box_blur_f32(&alpha, rw, rh, BLUR_R);
    let mut veil = vec![0u8; rw * rh * 4];
    for (i, &a) in alpha.iter().enumerate() {
        if a > 0.002 {
            let p = i * 4;
            veil[p] = TINT[0];
            veil[p + 1] = TINT[1];
            veil[p + 2] = TINT[2];
            veil[p + 3] = (a * 255.0).clamp(0.0, 255.0) as u8;
        }
    }
    // `base` maps FULL image-px → screen; the sub-image rides it after a translate to the rect origin.
    let base = super::bgremoval_preview::sprite_image_to_screen_affine(
        cw,
        ch,
        tr,
        sprite,
        camera,
        window_size,
    );
    let affine = base * ph2d_vector::Affine::translate((f64::from(rx0), f64::from(ry0)));
    vector_scene.draw_image_rgba_transformed(
        &std::sync::Arc::new(veil),
        rw as u32,
        rh as u32,
        affine,
        ph2d_vector::ImageQuality::Low,
    );
}

/// Separable box blur of a `w×h` f32 map (sliding window, O(w·h) — safe on a full-canvas wet map). Edge
/// pixels normalize by the FULL window, so the field FADES softly at the boundary (the damp fringe we want).
fn box_blur_f32(src: &[f32], w: usize, h: usize, r: usize) -> Vec<f32> {
    if r == 0 || w == 0 || h == 0 {
        return src.to_vec();
    }
    let win = (2 * r + 1) as f32;
    let mut tmp = vec![0.0f32; w * h];
    for y in 0..h {
        let base = y * w;
        let mut acc = 0.0f32;
        for x in 0..(r + 1).min(w) {
            acc += src[base + x];
        }
        for x in 0..w {
            tmp[base + x] = acc / win;
            if x + r + 1 < w {
                acc += src[base + x + r + 1];
            }
            if x >= r {
                acc -= src[base + x - r];
            }
        }
    }
    let mut out = vec![0.0f32; w * h];
    for x in 0..w {
        let mut acc = 0.0f32;
        for y in 0..(r + 1).min(h) {
            acc += tmp[y * w + x];
        }
        for y in 0..h {
            out[y * w + x] = acc / win;
            if y + r + 1 < h {
                acc += tmp[(y + r + 1) * w + x];
            }
            if y >= r {
                acc -= tmp[(y - r) * w + x];
            }
        }
    }
    out
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
            let affine = super::bgremoval_preview::sprite_image_to_screen_affine(
                iw,
                ih,
                tr,
                sprite,
                camera,
                window_size,
            );
            use ph2d_vector::Point;
            let map = |p: [f32; 2]| affine * Point::new(f64::from(p[0]), f64::from(p[1]));
            // Ellipse stroke gizmo = fluorescent YELLOW (distinct stroke-shape accent).
            let pal = super::painter_bridge_gizmo::palette_accent(
                hero.theme,
                super::painter_bridge_gizmo::GIZMO_ACCENTS[0],
            );
            let scene = vector_scene.inner_mut();
            // Outline + handles in the Sprite-gizmo style (theme tokens, a touch darker): the axis + centre
            // handles are rounded squares, the rotate handle is a circle. Matches the selection gizmos.
            if overlay.perimeter.len() >= 2 {
                let pts: Vec<Point> = overlay.perimeter.iter().map(|&p| map(p)).collect();
                super::painter_bridge_gizmo::stroke_box(scene, &pts, &pal);
            }
            let op_glyph = painter.active_op_glyph();
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
            let affine = super::bgremoval_preview::sprite_image_to_screen_affine(
                iw,
                ih,
                tr,
                sprite,
                camera,
                window_size,
            );
            use ph2d_vector::Point;
            let map = |p: [f32; 2]| affine * Point::new(f64::from(p[0]), f64::from(p[1]));
            // Polygon stroke gizmo = fluorescent PINK (distinct stroke-shape accent).
            let pal = super::painter_bridge_gizmo::palette_accent(
                hero.theme,
                super::painter_bridge_gizmo::GIZMO_ACCENTS[1],
            );
            let scene = vector_scene.inner_mut();
            // Sprite-gizmo style (theme tokens, a touch darker): outline box + axis/centre squares + the
            // rotate & sides handles as circles. Matches the selection gizmos.
            if overlay.perimeter.len() >= 2 {
                let pts: Vec<Point> = overlay.perimeter.iter().map(|&p| map(p)).collect();
                super::painter_bridge_gizmo::stroke_box(scene, &pts, &pal);
            }
            let op_glyph = painter.active_op_glyph();
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
