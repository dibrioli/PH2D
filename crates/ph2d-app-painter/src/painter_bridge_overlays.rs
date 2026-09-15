//! ⏸️ **DORMENTE enquanto pintar achatar a arte** (ordem do dono, 2026-09-15): debaixo do pincel
//! não há dobra, logo o mapa deste ficheiro degenera no afim do quad, **ao bit**. Tudo o que o
//! cabeçalho abaixo afirma sobre seguir a arte dobrada está CERTO e hoje **não acontece** — o
//! porquê, o custo de o manter vivo e o instrumento que ata esta nota ao achatamento estão no
//! cabeçalho do [`crate::canvas_map`].
//!
//! Painter on-canvas editing chrome — the brush cursor ring + the Curve / Circle / Polygon /
//! Stencil editor overlays — split from `painter_bridge.rs` for the HR-18 file-LOC cap. Pure draw:
//! reads the active `PainterTool` + selection + camera and writes guide geometry into the overlay
//! `VectorScene`; it mutates no tool or model state. Called once per frame by `painter_bridge::dispatch`
//! while the Painter tool is active (inside the same downcast block that owns `painter`).
use ph2d_ecs::SimWorld;
use ph2d_editor_core::HeroScreen;
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
pub fn draw_overlays(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    // ⭐⭐⭐ **O mundo de APRESENTAÇÃO (read-only)** — o chrome do canvas desenha-se POR CIMA da arte,
    // e desde 2026-09-15 ele tem de saber onde a arte de facto está: a malha posada vive aqui, e o
    // ponteiro que agarra estas alças já a consulta. Ver [`crate::canvas_map`].
    present: &ph2d_ecs::World,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
    text_system: &mut ph2d_text::TextSystem,
    cursor: (f32, f32),
    // `PH2D_PAINT_PERF` split: per-call ms, in call order (`paint_perf::CHROME_LABELS`). Written only
    // when `timing` — otherwise this function does not read the clock at all.
    perf: &mut [f32; crate::paint_perf::CHROME_SUB],
    timing: bool,
    // ⚠️ **O arrasto de cor do balde vem RESOLVIDO** (W2 Fase D): o estado dele é um
    // `thread_local` da camada de entrada da shell, e ir buscá-lo daqui era a única aresta deste
    // grupo de ficheiros para a `shells/desktop`. Quem possui o arrasto é quem responde.
    fill_drag_armed: bool,
) {
    let mut t = std::time::Instant::now();
    let mark = |slot: &mut f32, t: &mut std::time::Instant| {
        if timing {
            *slot = t.elapsed().as_secs_f64() as f32 * 1e3;
            *t = std::time::Instant::now();
        }
    };
    // Wetness sheen FIRST — under the brush ring + editor guides (#12a).
    crate::painter_bridge_wetness::draw_wetness_overlay(
        painter,
        hero,
        sim,
        camera,
        window_size,
        vector_scene,
    );
    mark(&mut perf[0], &mut t);
    crate::painter_bridge_brush_ring::draw_brush_ring(
        painter,
        hero,
        sim,
        camera,
        window_size,
        vector_scene,
        cursor,
    );
    mark(&mut perf[1], &mut t);
    crate::painter_bridge_curve_overlay::draw_curve_overlay(
        painter,
        hero,
        sim,
        present,
        camera,
        window_size,
        vector_scene,
        cursor,
    );
    mark(&mut perf[2], &mut t);
    crate::painter_bridge_shape_overlays::draw_ellipse_overlay(
        painter,
        hero,
        sim,
        present,
        camera,
        window_size,
        vector_scene,
    );
    mark(&mut perf[3], &mut t);
    crate::painter_bridge_line_overlay::draw_line_overlay(
        painter,
        hero,
        sim,
        present,
        camera,
        window_size,
        vector_scene,
        text_system,
        cursor,
    );
    mark(&mut perf[4], &mut t);
    crate::painter_bridge_shape_overlays::draw_polygon_overlay(
        painter,
        hero,
        sim,
        present,
        camera,
        window_size,
        vector_scene,
    );
    mark(&mut perf[5], &mut t);
    // Multi-shape op badges — the `+`/`−`/`○` type-square glyph per shape + a frame per parked shape.
    crate::painter_bridge_op_badges::draw_op_badges(
        painter,
        hero,
        sim,
        present,
        camera,
        window_size,
        vector_scene,
    );
    mark(&mut perf[6], &mut t);
    // Isolated SELECTION gizmos (ADR-0103 Am.2 v2) — every editable selection shape's gizmo at once.
    crate::painter_bridge_selection_gizmos::draw_selection_gizmos(
        painter,
        hero,
        sim,
        present,
        camera,
        window_size,
        vector_scene,
        cursor,
    );
    mark(&mut perf[7], &mut t);
    // Deform Transform gizmo (Wave 2) — the whole-region bounding box, when Transform temperament is active.
    crate::painter_bridge_deform_gizmo::draw_deform_gizmo(
        painter,
        hero,
        sim,
        present,
        camera,
        window_size,
        vector_scene,
        cursor,
    );
    mark(&mut perf[8], &mut t);
    crate::painter_bridge_shape_overlays::draw_stencil_overlay(
        painter,
        hero,
        sim,
        present,
        camera,
        window_size,
        vector_scene,
        cursor,
    );
    mark(&mut perf[9], &mut t);
    draw_symmetry_overlay(
        painter,
        hero,
        sim,
        present,
        camera,
        window_size,
        vector_scene,
    );
    mark(&mut perf[10], &mut t);
    // A rede do Grid Stamp — depois da simetria (as duas são guias de canvas) e antes do cursor de Fill.
    crate::painter_bridge_grid::draw_grid_overlay(
        painter,
        hero,
        sim,
        present,
        camera,
        window_size,
        vector_scene,
    );
    mark(&mut perf[11], &mut t);
    crate::painter_bridge_fill_overlay::draw_fill_cursor(
        painter,
        vector_scene,
        cursor,
        fill_drag_armed,
    );
    mark(&mut perf[12], &mut t);
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
        ph2d_ecs::world_transform(sim.world(), entity),
        sim.world().get::<ph2d_render::Sprite>(entity),
    ) else {
        return;
    };
    // A grelha desta sprite (ADR-0164 F1 passo 6) — ausente = uma célula, e aí o quad do
    // afim é o de sempre, byte-idêntico.
    let sprite_grid = sim.world().get::<ph2d_ecs::SpriteGrid>(entity).copied();
    let affine = ph2d_sprite_screen::sprite_image_to_screen_affine(
        iw,
        ih,
        tr,
        sprite,
        sprite_grid,
        camera,
        window_size,
    );
    painter.set_shape_grab_tol_px(crate::shape_grab::shape_grab_tol_from_affine(&affine));
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
    present: &ph2d_ecs::World,
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
        ph2d_ecs::world_transform(sim.world(), entity),
        sim.world().get::<ph2d_render::Sprite>(entity),
    ) else {
        return;
    };
    // A grelha desta sprite (ADR-0164 F1 passo 6) — ausente = uma célula, e aí o quad do
    // afim é o de sempre, byte-idêntico.
    let sprite_grid = sim.world().get::<ph2d_ecs::SpriteGrid>(entity).copied();
    let affine = ph2d_sprite_screen::sprite_image_to_screen_affine(
        iw,
        ih,
        tr,
        sprite,
        sprite_grid,
        camera,
        window_size,
    );
    use ph2d_vector::{Affine, BezPath, Brush, Color, Stroke};
    // ⭐⭐⭐ As guias de simetria seguem a arte dobrada (item 4 do dono) — e aqui a razão é mais
    // dura que estética: elas dizem ONDE o motor replica os traços, e o motor replica em espaço de
    // IMAGEM. Uma guia recta por cima de arte dobrada aponta para um sítio onde nada é espelhado.
    let mapa =
        crate::canvas_map::CanvasMap::new(present, bits, iw, ih, affine, camera, window_size);
    // ⛔⛔ **UMA GUIA É UM SEGMENTO, NÃO DUAS PONTAS.** Mapear só o início e o fim e ligá-los com
    // um `line_to` deixa a guia RECTA por cima de arte dobrada — o mapa teria sido consultado e a
    // linha sairia igual. A [`crate::canvas_map::CanvasMap::polyline`] parte o segmento até o desvio
    // caber em meio pixel de ecrã, e sobre um quad plano devolve exactamente as duas pontas, ao bit.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "o mapa fala em pixels de imagem (f32); a geometria da guia é derivada em f64"
    )]
    let guia = |a: (f64, f64), b: (f64, f64)| -> BezPath {
        let pts = mapa.polyline(&[[a.0 as f32, a.1 as f32], [b.0 as f32, b.1 as f32]], false);
        let mut path = BezPath::new();
        let Some((primeiro, resto)) = pts.split_first() else {
            return path;
        };
        path.move_to(*primeiro);
        for p in resto {
            path.line_to(*p);
        }
        path
    };
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
            let path = guia((cx, cy), (cx + co * span, cy + s * span));
            scene.stroke(&dash, Affine::IDENTITY, &Brush::Solid(color), None, &path);
        }
    } else {
        // Mirror line through the centre along the axis direction, extended both ways.
        let d = sym.mirror_dir();
        let (dx, dy) = (f64::from(d[0]), f64::from(d[1]));
        let path = guia(
            (cx - dx * span, cy - dy * span),
            (cx + dx * span, cy + dy * span),
        );
        scene.stroke(&dash, Affine::IDENTITY, &Brush::Solid(color), None, &path);
    }
}

/// **Repeat Image**: draw the painted composite repeated in the 8 neighbour positions around the
/// sprite (a 3×3 tile grid), so the artist sees the seamless tiling result. The centre is the real
/// sprite (drawn by the pipeline); we draw only the 8 wraps as overlay images, each abutting at the
/// sprite edges. No-op unless Repeat Image is on and a CPU composite for the selected sprite exists.
/// Must draw BEFORE the editing chrome (`painter_bridge::dispatch` calls it first): the tiles are
/// opaque full-canvas blits, so drawn later they cover any overlay past the sprite border (z-order
/// gate: `repeat_image_tiles_draw_under_the_editing_chrome`).
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_repeat_image(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
    preview: Option<&ph2d_tool_runtime::PreviewCache>,
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
        ph2d_ecs::world_transform(sim.world(), entity),
        sim.world().get::<ph2d_render::Sprite>(entity),
    ) else {
        return;
    };
    // A grelha desta sprite (ADR-0164 F1 passo 6) — ausente = uma célula, e aí o quad do
    // afim é o de sempre, byte-idêntico.
    let sprite_grid = sim.world().get::<ph2d_ecs::SpriteGrid>(entity).copied();
    // image-px → screen for the centre sprite; each neighbour prepends a screen-space translation of
    // the world offset (a pure translation maps through the world→screen scale `k`, Y flipped).
    let base = ph2d_sprite_screen::sprite_image_to_screen_affine(
        preview.width,
        preview.height,
        tr,
        sprite,
        sprite_grid,
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
