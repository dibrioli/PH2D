//! Painter on-canvas editing chrome — the brush cursor ring + the Curve / Circle / Polygon /
//! Stencil editor overlays — split from `painter_bridge.rs` for the HR-18 file-LOC cap. Pure draw:
//! reads the active `PainterTool` + selection + camera and writes guide geometry into the overlay
//! `VectorScene`; it mutates no tool or model state. Called once per frame by `painter_bridge::dispatch`
//! while the Painter tool is active (inside the same downcast block that owns `painter`).

use ph2d_ecs::SimWorld;
use ph2d_editor::HeroScreen;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tool_painter::{DAB_FLATTEN_MAX, PainterTool};
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
    cursor: (f32, f32),
) {
    draw_brush_ring(
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
    draw_line_overlay(painter, hero, sim, camera, window_size, vector_scene);
    draw_polygon_overlay(painter, hero, sim, camera, window_size, vector_scene);
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

/// Segments in the brush-cursor ellipse outline — enough that a flattened ellipse reads smooth.
const BRUSH_RING_SEGS: u32 = 64;

#[allow(clippy::too_many_arguments)]
fn draw_brush_ring(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
    cursor: (f32, f32),
) {
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
                // Image-space major radius, floored so the ring stays visible at tiny zoom (the old
                // screen-space `.max(1px)`).
                let r = if scale > 0.0 && f64::from(bs.size_px) * scale < 1.0 {
                    1.0 / scale
                } else {
                    f64::from(bs.size_px)
                };
                // Minor axis = (1 − flatten) of the major; rotated by the dab angle (engine
                // convention — `rotate_by_degrees` builds `[cos(deg), sin(deg)]`).
                let m = 1.0 - f64::from(bs.dab_flatten.clamp(0.0, DAB_FLATTEN_MAX));
                let (sin_a, cos_a) = f64::from(bs.dab_angle_deg).to_radians().sin_cos();
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
            use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Point, Stroke};
            let map = |p: [f32; 2]| affine * Point::new(f64::from(p[0]), f64::from(p[1]));
            let scene = vector_scene.inner_mut();
            let guide = Color::new([0.55, 0.72, 1.0, 0.85]); // LITERAL-COLOR-OK: ellipse guide
            // Outline.
            if overlay.perimeter.len() >= 2 {
                let mut path = BezPath::new();
                path.move_to(map(overlay.perimeter[0]));
                for &p in &overlay.perimeter[1..] {
                    path.line_to(map(p));
                }
                path.close_path();
                scene.stroke(
                    &Stroke::new(1.5),
                    Affine::IDENTITY,
                    &Brush::Solid(guide),
                    None,
                    &path,
                );
            }
            // Connector from the centre to the rotation handle.
            let mut stem = BezPath::new();
            stem.move_to(map(overlay.handles[5]));
            stem.line_to(map(overlay.handles[4]));
            scene.stroke(
                &Stroke::new(1.0),
                Affine::IDENTITY,
                &Brush::Solid(guide),
                None,
                &stem,
            );
            // Handles: axis (white), rotate (green), centre (grey), grabbed (orange).
            let axis = Color::new([0.95, 0.95, 0.97, 0.95]); // LITERAL-COLOR-OK: axis handle
            let rotate = Color::new([0.45, 0.85, 0.50, 1.0]); // LITERAL-COLOR-OK: rotation handle
            let center = Color::new([0.75, 0.78, 0.82, 0.95]); // LITERAL-COLOR-OK: centre handle
            let grab = Color::new([1.0, 0.62, 0.20, 1.0]); // LITERAL-COLOR-OK: grabbed handle
            for (i, &h) in overlay.handles.iter().enumerate() {
                let grabbed = overlay.grabbed == Some(i as u8);
                let base = match i {
                    4 => rotate,
                    5 => center,
                    _ => axis,
                };
                let c = if grabbed { grab } else { base };
                let r = if grabbed { 6.0 } else { 4.0 };
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(c),
                    None,
                    &Circle::new(map(h), r),
                );
            }
        }
    }
}

/// The **Line** polyline editor overlay: the segments through the committed corner points (+ the live
/// rubber-band to the draft while drawing, closing the loop when closed), a white dot at each corner, and
/// a fainter ghost dot at the draft point. (Per-corner Fillet/Chamfer gizmos + the finish transform gizmo
/// are layered on in later increments.)
fn draw_line_overlay(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
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
            use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Point, Stroke};
            let map = |p: [f32; 2]| affine * Point::new(f64::from(p[0]), f64::from(p[1]));
            let scene = vector_scene.inner_mut();
            let guide = Color::new([0.55, 0.72, 1.0, 0.85]); // LITERAL-COLOR-OK: line guide
            // Segments through the committed points + the rubber-band to the draft while drawing.
            let mut pts = overlay.points.clone();
            if let Some(d) = overlay.draft {
                pts.push(d);
            }
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
            // A white dot at each committed corner; a fainter ghost at the live draft point.
            let dot = Color::new([0.95, 0.95, 0.97, 0.95]); // LITERAL-COLOR-OK: corner dot
            for &p in &overlay.points {
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(dot),
                    None,
                    &Circle::new(map(p), 4.0),
                );
            }
            if let Some(d) = overlay.draft {
                let ghost = Color::new([0.95, 0.95, 0.97, 0.5]); // LITERAL-COLOR-OK: draft ghost dot
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(ghost),
                    None,
                    &Circle::new(map(d), 3.0),
                );
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
            use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Point, Stroke};
            let map = |p: [f32; 2]| affine * Point::new(f64::from(p[0]), f64::from(p[1]));
            let scene = vector_scene.inner_mut();
            let guide = Color::new([0.55, 0.72, 1.0, 0.85]); // LITERAL-COLOR-OK: polygon guide
            // Closed outline through the vertices.
            if overlay.perimeter.len() >= 2 {
                let mut path = BezPath::new();
                path.move_to(map(overlay.perimeter[0]));
                for &p in &overlay.perimeter[1..] {
                    path.line_to(map(p));
                }
                path.close_path();
                scene.stroke(
                    &Stroke::new(1.5),
                    Affine::IDENTITY,
                    &Brush::Solid(guide),
                    None,
                    &path,
                );
            }
            // Connectors from the centre to the rotation + sides handles.
            for h in [overlay.handles[4], overlay.handles[5]] {
                let mut stem = BezPath::new();
                stem.move_to(map(overlay.handles[6]));
                stem.line_to(map(h));
                scene.stroke(
                    &Stroke::new(1.0),
                    Affine::IDENTITY,
                    &Brush::Solid(guide),
                    None,
                    &stem,
                );
            }
            let axis = Color::new([0.95, 0.95, 0.97, 0.95]); // LITERAL-COLOR-OK: axis handle
            let rotate = Color::new([0.45, 0.85, 0.50, 1.0]); // LITERAL-COLOR-OK: rotation handle
            let sides = Color::new([0.40, 0.78, 0.95, 1.0]); // LITERAL-COLOR-OK: sides handle
            let center = Color::new([0.75, 0.78, 0.82, 0.95]); // LITERAL-COLOR-OK: centre handle
            let grab = Color::new([1.0, 0.62, 0.20, 1.0]); // LITERAL-COLOR-OK: grabbed handle
            for (i, &h) in overlay.handles.iter().enumerate() {
                let grabbed = overlay.grabbed == Some(i as u8);
                let base = match i {
                    4 => rotate,
                    5 => sides,
                    6 => center,
                    _ => axis,
                };
                let c = if grabbed { grab } else { base };
                let r = if grabbed { 6.0 } else { 4.0 };
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(c),
                    None,
                    &Circle::new(map(h), r),
                );
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
