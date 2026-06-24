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
    draw_curve_overlay(painter, hero, sim, camera, window_size, vector_scene);
    draw_circle_overlay(painter, hero, sim, camera, window_size, vector_scene);
    draw_polygon_overlay(painter, hero, sim, camera, window_size, vector_scene);
    draw_stencil_overlay(painter, hero, sim, camera, window_size, vector_scene);
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
    let k = f64::from(window_size.height) / f64::from(camera.height_world).max(1e-6);
    let off_w = f64::from(sprite.size[0]);
    let off_h = f64::from(sprite.size[1]);
    for dy in [-1i32, 0, 1] {
        for dx in [-1i32, 0, 1] {
            if dx == 0 && dy == 0 {
                continue; // the real sprite occupies the centre
            }
            let screen_off = ph2d_vector::Affine::translate((
                f64::from(dx) * off_w * k,
                -f64::from(dy) * off_h * k,
            ));
            vector_scene.draw_image_rgba_transformed(
                &preview.rgba,
                preview.width,
                preview.height,
                screen_off * base,
                ph2d_vector::ImageQuality::Low,
            );
        }
    }
}

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
    // Uses the same footprint-AABB mapping as the paint delivery, so the
    // ring matches where dabs land. Drawn into the overlay scene (composited
    // over the canvas this frame, like the rubber-band / bgremoval ring).
    if let Some(bits) = hero.gizmo.selection {
        let (cx, cy) = cursor;
        if hero.store.panel_at(cx, cy).is_none() {
            let size_px = painter.brush_settings().size_px;
            let (iw, _ih) = painter.canvas_size();
            let entity = ph2d_ecs::Entity::from_bits(bits);
            if iw > 0
                && let (Some(tr), Some(sprite)) = (
                    sim.world().get::<crate::Transform>(entity),
                    sim.world().get::<ph2d_render::Sprite>(entity),
                )
            {
                let (tx, ty) = (tr.translation.x, tr.translation.y);
                let (sw, sh) = (sprite.size[0], sprite.size[1]);
                let (x0, _) = camera.world_to_screen([tx - sw * 0.5, ty + sh * 0.5], window_size);
                let (x1, _) = camera.world_to_screen([tx + sw * 0.5, ty - sh * 0.5], window_size);
                let scale = (x1 - x0).abs() / iw as f32;
                let r_screen = (size_px * scale).max(1.0);
                use ph2d_vector::{Affine, Brush, Circle, Color, Stroke};
                // Light-grey ring (baked inline, like the rubber-band overlay's
                // colour — a follow-up can swap to a theme token / 2-tone).
                let color = Color::new([0.78, 0.78, 0.78, 0.85]); // LITERAL-COLOR-OK: overlay cursor
                vector_scene.inner_mut().stroke(
                    &Stroke::new(1.5),
                    Affine::IDENTITY,
                    &Brush::Solid(color),
                    None,
                    &Circle::new((f64::from(cx), f64::from(cy)), f64::from(r_screen)),
                );
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_curve_overlay(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
) {
    // ── Curve editor overlay (control dots + the auto-smoothed spine) ──────
    // Drawn while a Curve session is being EDITED, regardless of the cursor /
    // panels — it's the editing chrome, not a hover hint. Maps image px →
    // screen via the SAME sprite-footprint AABB as the paint delivery, so the
    // dots sit exactly on the painted curve.
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
            let (tx, ty) = (tr.translation.x, tr.translation.y);
            let (sw, sh) = (sprite.size[0], sprite.size[1]);
            let (sx0, sy0) = camera.world_to_screen([tx - sw * 0.5, ty + sh * 0.5], window_size);
            let (sx1, sy1) = camera.world_to_screen([tx + sw * 0.5, ty - sh * 0.5], window_size);
            use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Point, Stroke};
            let map = |p: [f32; 2]| {
                Point::new(
                    f64::from(sx0 + p[0] / iw as f32 * (sx1 - sx0)),
                    f64::from(sy0 + p[1] / ih as f32 * (sy1 - sy0)),
                )
            };
            let scene = vector_scene.inner_mut();
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

#[allow(clippy::too_many_arguments)]
fn draw_circle_overlay(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
) {
    // ── Circle editor overlay (ellipse outline + 4 axis handles + rotate + centre) ──
    // Same footprint mapping as the curve overlay; the handle indices match `CircleOverlay`:
    // 0 right, 1 top, 2 left, 3 bottom, 4 rotate, 5 centre.
    if let Some(bits) = hero.gizmo.selection
        && let Some(overlay) = painter.circle_overlay()
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
            let (tx, ty) = (tr.translation.x, tr.translation.y);
            let (sw, sh) = (sprite.size[0], sprite.size[1]);
            let (sx0, sy0) = camera.world_to_screen([tx - sw * 0.5, ty + sh * 0.5], window_size);
            let (sx1, sy1) = camera.world_to_screen([tx + sw * 0.5, ty - sh * 0.5], window_size);
            use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Point, Stroke};
            let map = |p: [f32; 2]| {
                Point::new(
                    f64::from(sx0 + p[0] / iw as f32 * (sx1 - sx0)),
                    f64::from(sy0 + p[1] / ih as f32 * (sy1 - sy0)),
                )
            };
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
            let (tx, ty) = (tr.translation.x, tr.translation.y);
            let (sw, sh) = (sprite.size[0], sprite.size[1]);
            let (sx0, sy0) = camera.world_to_screen([tx - sw * 0.5, ty + sh * 0.5], window_size);
            let (sx1, sy1) = camera.world_to_screen([tx + sw * 0.5, ty - sh * 0.5], window_size);
            use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Point, Stroke};
            let map = |p: [f32; 2]| {
                Point::new(
                    f64::from(sx0 + p[0] / iw as f32 * (sx1 - sx0)),
                    f64::from(sy0 + p[1] / ih as f32 * (sy1 - sy0)),
                )
            };
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
) {
    // ── Stencil texture overlay (rect outline + drag handles of the image-space mask) ──
    // The stencil is positioned/sized via its handles (corners = resize, centre = move) or the
    // Texture section's Offset/Size sliders; Angle rotates it. The outline shows where the mask
    // lets paint through.
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
            let (tx, ty) = (tr.translation.x, tr.translation.y);
            let (sw, sh) = (sprite.size[0], sprite.size[1]);
            let (sx0, sy0) = camera.world_to_screen([tx - sw * 0.5, ty + sh * 0.5], window_size);
            let (sx1, sy1) = camera.world_to_screen([tx + sw * 0.5, ty - sh * 0.5], window_size);
            use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Point, Stroke};
            let map = |p: [f32; 2]| {
                Point::new(
                    f64::from(sx0 + p[0] / iw as f32 * (sx1 - sx0)),
                    f64::from(sy0 + p[1] / ih as f32 * (sy1 - sy0)),
                )
            };
            let scene = vector_scene.inner_mut();
            let guide = Color::new([1.0, 0.62, 0.20, 0.9]); // LITERAL-COLOR-OK: stencil outline
            let mut path = BezPath::new();
            path.move_to(map(overlay.corners[0]));
            for &p in &overlay.corners[1..] {
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
            // Handles: 4 corners (resize) + centre (move); the grabbed one is larger + orange.
            let handle = Color::new([0.95, 0.95, 0.97, 0.95]); // LITERAL-COLOR-OK: stencil handle
            let grab = Color::new([1.0, 0.62, 0.20, 1.0]); // LITERAL-COLOR-OK: grabbed handle
            for (i, &p) in overlay
                .corners
                .iter()
                .enumerate()
                .chain(std::iter::once((4usize, &overlay.center)))
            {
                let grabbed = overlay.grabbed == Some(i as u8);
                let c = if grabbed { grab } else { handle };
                let r = if grabbed { 6.0 } else { 4.0 };
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
