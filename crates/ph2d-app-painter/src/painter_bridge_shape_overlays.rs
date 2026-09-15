//! ⏸️ **DORMENTE enquanto pintar achatar a arte** (ordem do dono, 2026-09-15): debaixo do pincel
//! não há dobra, logo o mapa deste ficheiro degenera no afim do quad, **ao bit**. Tudo o que o
//! cabeçalho abaixo afirma sobre seguir a arte dobrada está CERTO e hoje **não acontece** — o
//! porquê, o custo de o manter vivo e o instrumento que ata esta nota ao achatamento estão no
//! cabeçalho do [`crate::canvas_map`].
//!
//! **Os gizmos das FORMAS de traço** — elipse · polígono · stencil —, partidos do
//! `painter_bridge_overlays` pelo teto de LOC por ficheiro (2026-09-15).
//!
//! ⚠️ **O corte é por RESPONSABILIDADE e não por tamanho:** os três desenham a figura de uma forma
//! que o artista está a **editar no canvas** (contorno + alças agarráveis), enquanto o que fica no
//! módulo pai é o DESPACHO, as guias de simetria e os ladrilhos do *Repeat Image* (que é uma
//! IMAGEM, não geometria).
//!
//! ⭐⭐⭐ **Os três pintam ONDE A ARTE DESENHA** ([`crate::canvas_map::CanvasMap`]) — foi esta a wave
//! que os trouxe do afim do quad de repouso (report do dono, 2026-09-15: o contorno da elipse saía
//! recto por cima de arte dobrada, e é exactamente a forma que o roteiro do smoke manda arrastar).
//! ⛔ **E a caixa fecha PELA PORTA** (`polyline(.., true)`): quem a desenha chama `close_path`, que
//! liga o último canto ao primeiro **a direito** — sem isso três arestas seguem a arte e a quarta
//! corta por cima dela, e as três primeiras convencem o olho.
use ph2d_ecs::SimWorld;
use ph2d_editor_core::HeroScreen;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tool_painter::PainterTool;
use ph2d_vector::VectorScene;

use crate::painter_bridge_overlays::overlay_tile_offsets;

#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_ellipse_overlay(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    present: &ph2d_ecs::World,
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
                ph2d_ecs::world_transform(sim.world(), entity),
                sim.world().get::<ph2d_render::Sprite>(entity),
            )
        {
            // image-px → screen via the FULL sprite affine, so the handles ride scale / AR / rotation.
            let base_affine = ph2d_sprite_screen::sprite_image_to_screen_affine(
                iw,
                ih,
                tr,
                sprite,
                sim.world().get::<ph2d_ecs::SpriteGrid>(entity).copied(),
                camera,
                window_size,
            );
            // ⭐⭐⭐ **O CONTORNO É PINTADO ONDE A ARTE DESENHA** (item 4 do dono) — e esta forma
            // era a ÚLTIMA da espécie a sair recta por cima de arte dobrada, apesar de ser
            // exactamente a que o smoke do osso manda arrastar. ⚠️ A metade que a torna obrigatória
            // é a do DEDO: estas alças são agarradas pelo `deliver_canvas_pointer`, que resolve pela
            // malha desde 2026-09-14 — *as duas direcções viajam juntas ou nenhuma viaja*.
            let base_mapa = crate::canvas_map::CanvasMap::new(
                present,
                bits,
                iw,
                ih,
                base_affine,
                camera,
                window_size,
            );
            // Ellipse stroke gizmo = fluorescent YELLOW (distinct stroke-shape accent).
            let pal = crate::painter_bridge_gizmo::palette_accent(
                hero.theme,
                crate::painter_bridge_gizmo::GIZMO_ACCENTS[0],
            );
            let op_glyph = painter.active_op_glyph();
            let scene = vector_scene.inner_mut();
            // Edit-in-tile: draw the gizmo in each visible wrapped tile too (`overlay_tile_offsets`).
            for (ox, oy) in overlay_tile_offsets(painter, iw, ih) {
                // ⚠️ O deslocamento do ladrilho entra nas DUAS metades do mapa (o quad e a malha) —
                // ver o doc do [`crate::canvas_map::CanvasMap::deslocado`].
                let mapa = base_mapa.deslocado(ox, oy);
                let map = |p: [f32; 2]| mapa.point(p);
                // Outline + handles in the Sprite-gizmo style: the axis + centre handles are rounded squares,
                // the rotate handle is a circle. Matches the selection gizmos.
                if overlay.perimeter.len() >= 2 {
                    // ⛔ **FECHADA**: o `stroke_box` fecha o caminho, e sem o troço de fecho a última
                    // aresta sairia RECTA enquanto as outras seguem a arte. *Meia lei aplicada é pior
                    // que nenhuma, porque as primeiras convencem o olho.*
                    let pts = mapa.polyline(&overlay.perimeter, true);
                    crate::painter_bridge_gizmo::stroke_box(scene, &pts, &pal);
                }
                // ⚠️ O contorno aparece nas duas fases; as ALÇAS, só na de edição — no meio do arrasto
                // de criação nenhum Down as alcança (`EllipseOverlay::editing`).
                if !overlay.editing {
                    continue;
                }
                for (i, &h) in overlay.handles.iter().enumerate() {
                    let p = map(h);
                    if i == 4 {
                        crate::painter_bridge_gizmo::circle_handle(scene, p, &pal);
                    } else if i == 5 && op_glyph.is_some() {
                        // Centre-move square (index 5) DOUBLED with the Operation glyph.
                        crate::painter_bridge_gizmo::center_glyph_handle(
                            scene,
                            p,
                            &pal,
                            op_glyph.unwrap(),
                        );
                    } else {
                        crate::painter_bridge_gizmo::square_handle(scene, p, &pal);
                    }
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_polygon_overlay(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    present: &ph2d_ecs::World,
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
                ph2d_ecs::world_transform(sim.world(), entity),
                sim.world().get::<ph2d_render::Sprite>(entity),
            )
        {
            // image-px → screen via the FULL sprite affine, so the handles ride scale / AR / rotation.
            let base_affine = ph2d_sprite_screen::sprite_image_to_screen_affine(
                iw,
                ih,
                tr,
                sprite,
                sim.world().get::<ph2d_ecs::SpriteGrid>(entity).copied(),
                camera,
                window_size,
            );
            // ⭐⭐⭐ O contorno é pintado onde a arte desenha — ver a nota gémea na
            // [`draw_ellipse_overlay`].
            let base_mapa = crate::canvas_map::CanvasMap::new(
                present,
                bits,
                iw,
                ih,
                base_affine,
                camera,
                window_size,
            );
            // Polygon stroke gizmo = fluorescent PINK (distinct stroke-shape accent).
            let pal = crate::painter_bridge_gizmo::palette_accent(
                hero.theme,
                crate::painter_bridge_gizmo::GIZMO_ACCENTS[1],
            );
            let op_glyph = painter.active_op_glyph();
            let scene = vector_scene.inner_mut();
            // Edit-in-tile: draw the gizmo in each visible wrapped tile too (`overlay_tile_offsets`).
            for (ox, oy) in overlay_tile_offsets(painter, iw, ih) {
                let mapa = base_mapa.deslocado(ox, oy);
                let map = |p: [f32; 2]| mapa.point(p);
                // Sprite-gizmo style: outline box + axis/centre squares + the rotate & sides handles as
                // circles. Matches the selection gizmos.
                if overlay.perimeter.len() >= 2 {
                    // ⛔ FECHADA — ver a nota da elipse.
                    let pts = mapa.polyline(&overlay.perimeter, true);
                    crate::painter_bridge_gizmo::stroke_box(scene, &pts, &pal);
                }
                // O contorno nas duas fases, as ALÇAS só na de edição — ver `draw_ellipse_overlay`.
                if !overlay.editing {
                    continue;
                }
                for (i, &h) in overlay.handles.iter().enumerate() {
                    let p = map(h);
                    match i {
                        4 => crate::painter_bridge_gizmo::circle_handle(scene, p, &pal), // rotate
                        5 => crate::painter_bridge_gizmo::diamond_handle(scene, p, &pal), // sides (distinct)
                        6 if op_glyph.is_some() => {
                            // Centre-move square (index 6) DOUBLED with the Operation glyph.
                            crate::painter_bridge_gizmo::center_glyph_handle(
                                scene,
                                p,
                                &pal,
                                op_glyph.unwrap(),
                            );
                        }
                        _ => crate::painter_bridge_gizmo::square_handle(scene, p, &pal),
                    }
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_stencil_overlay(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    present: &ph2d_ecs::World,
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
                ph2d_ecs::world_transform(sim.world(), entity),
                sim.world().get::<ph2d_render::Sprite>(entity),
            )
        {
            // image-px → screen via the FULL sprite affine, so the handles ride scale / AR / rotation.
            let affine = ph2d_sprite_screen::sprite_image_to_screen_affine(
                iw,
                ih,
                tr,
                sprite,
                sim.world().get::<ph2d_ecs::SpriteGrid>(entity).copied(),
                camera,
                window_size,
            );
            use ph2d_vector::{Affine, Point};
            // ⭐⭐⭐ A caixa do stencil é pintada onde a arte desenha — ver a nota gémea na
            // [`draw_ellipse_overlay`].
            let mapa = crate::canvas_map::CanvasMap::new(
                present,
                bits,
                iw,
                ih,
                affine,
                camera,
                window_size,
            );
            let c = affine.as_coeffs();
            let scale = (c[0] * c[0] + c[1] * c[1]).sqrt();
            let map = |p: [f32; 2]| mapa.point(p);
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
            let pal = crate::painter_bridge_gizmo::palette(hero.theme);
            // ⛔ FECHADA: sem o troço de fecho o quarto lado sairia recto — ver a nota da elipse.
            let box_pts = mapa.polyline(&overlay.corners, true);
            crate::painter_bridge_gizmo::stroke_box(scene, &box_pts, &pal);
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
                    crate::painter_bridge_gizmo::circle_handle(scene, sp, &pal);
                } else {
                    crate::painter_bridge_gizmo::square_handle(scene, sp, &pal);
                }
            }
            crate::painter_bridge_gizmo::square_handle(scene, center_sp, &pal);
        }
    }
}
