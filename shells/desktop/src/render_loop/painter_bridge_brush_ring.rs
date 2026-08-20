//! The brush-cursor **ring** — the brush footprint (flatten+rotate ellipse) scaled to screen at the
//! cursor. Split out of `painter_bridge_overlays` for the HR-18 file-LOC cap. Pure draw. Skipped in
//! Selection mode (which shows a crosshair cursor instead — see `painter_bridge_selection_overlay`).

use ph2d_ecs::SimWorld;
use ph2d_editor::HeroScreen;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tool_painter::{DAB_FLATTEN_MAX, PainterTool};
use ph2d_vector::VectorScene;

/// Segments in the brush-cursor ellipse outline — enough that a flattened ellipse reads smooth.
const BRUSH_RING_SEGS: u32 = 64;

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_brush_ring(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
    cursor: (f32, f32),
) {
    // Selection mode has its own crosshair cursor (drawn by the selection overlay), not the brush ring.
    if painter.is_selection_mode() {
        return;
    }
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
                    ph2d_ecs::world_transform(sim.world(), entity),
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
                // ── GRID STAMP: o cursor É a célula, encaixada ──────────────────────────────────
                // Neste método o dab não pousa onde o ponteiro está — ele pousa no CENTRO da célula
                // que o ponteiro cobre, com o tamanho dela. Um anel elíptico preso ao ponteiro
                // desenha, então, um lugar onde nada vai acontecer e um tamanho que não é o do
                // carimbo (Enio, 2026-08-09). A célula sai da porta do carimbo (`grid_cell_rect_at`),
                // nunca de uma rede re-derivada aqui.
                if let Some((centre, half)) = grid_cell_under(painter, affine, cursor) {
                    draw_grid_cell(vector_scene, affine, centre, half);
                    return;
                }
                let scale = (c[0] * c[0] + c[1] * c[1]).sqrt();
                // Deform uses its OWN (round) brush footprint — the deform radius, no flatten/rotation — so
                // the ring shows the deform size, not the paint brush's (Enio 2026-07-04).
                let deform = painter.is_deform_mode();
                let footprint_px = if deform {
                    bs.deform_size_px
                } else {
                    bs.size_px
                };
                // Image-space major radius, floored so the ring stays visible at tiny zoom (the old
                // screen-space `.max(1px)`).
                let r = if scale > 0.0 && f64::from(footprint_px) * scale < 1.0 {
                    1.0 / scale
                } else {
                    f64::from(footprint_px)
                };
                // Minor axis = (1 − flatten) of the major, rotated by the dab's LIVE orientation rotor
                // (`BrushSettings::dab_rotor`): the brush Angle already turned by the stroke-follow
                // rotation, so with Shape Rake / Flow (or Grain Rake) the ring turns WITH THE STROKE in
                // real time, exactly like the tip it represents (Enio 2026-07-19). The rotor comes from
                // the engine's own heading — the ring never re-derives a direction of its own, so it
                // cannot point somewhere the paint does not. Deform is a plain, unrotated disc.
                let m = if deform {
                    1.0
                } else {
                    1.0 - f64::from(bs.dab_flatten.clamp(0.0, DAB_FLATTEN_MAX))
                };
                let rotor = if deform { [1.0, 0.0] } else { bs.dab_rotor };
                let (cos_a, sin_a) = (f64::from(rotor[0]), f64::from(rotor[1]));
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

/// A célula do Grid Stamp sob o cursor, em px de IMAGEM: `(centro, meia-extensão)`. `None` fora do
/// método, ou sob um afim que não se inverte.
///
/// ⚠️ **O afim degenerado é recusado, não aproximado.** Uma sprite com escala zero num eixo (ou um
/// zoom que colapsou) tem determinante zero, e a inversa devolveria `inf`/`NaN` — que `grid_cell_at`
/// converteria em `as i32` SATURADO, desenhando uma célula a um bilhão de pixels de onde o cursor
/// está. Devolver `None` deixa o anel normal aparecer, que é a degradação honesta.
fn grid_cell_under(
    painter: &PainterTool,
    affine: ph2d_vector::Affine,
    cursor: (f32, f32),
) -> Option<([f32; 2], [f32; 2])> {
    // ⚠️ **O Deform tem footprint PRÓPRIO** (o raio dele, sem achatamento nem grade) e não passa pelo
    // carimbo, então com o Grid Stamp armado por baixo o cursor mostraria uma célula onde o gesto de
    // deformar não encaixa em nada. Smear / Blur / Clone NÃO entram nesta recusa: eles carimbam pelo
    // mesmo `stamp_dabs_inner`, logo pousam na célula de verdade.
    if painter.is_deform_mode() {
        return None;
    }
    let c = affine.as_coeffs();
    let det = c[0] * c[3] - c[1] * c[2];
    if !det.is_finite() || det.abs() < f64::EPSILON {
        return None;
    }
    let p = affine.inverse() * ph2d_vector::Point::new(f64::from(cursor.0), f64::from(cursor.1));
    painter.grid_cell_rect_at([p.x as f32, p.y as f32])
}

/// Desenha o retângulo da célula (px de imagem) pelo afim COMPLETO — com translação, ao contrário do
/// anel elíptico, que é ancorado no cursor: uma célula tem um lugar próprio, e é esse lugar que o
/// desenho tem de contar.
fn draw_grid_cell(
    vector_scene: &mut ph2d_vector::VectorScene,
    affine: ph2d_vector::Affine,
    centre: [f32; 2],
    half: [f32; 2],
) {
    use ph2d_vector::{Affine, BezPath, Brush, Color, Point, Stroke};
    let (cx, cy) = (f64::from(centre[0]), f64::from(centre[1]));
    let (hx, hy) = (f64::from(half[0]), f64::from(half[1]));
    let mut path = BezPath::new();
    for (i, (sx, sy)) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
        .into_iter()
        .enumerate()
    {
        let p = affine * Point::new(cx + sx * hx, cy + sy * hy);
        if i == 0 {
            path.move_to(p);
        } else {
            path.line_to(p);
        }
    }
    path.close_path();
    let color = Color::new([0.78, 0.78, 0.78, 0.85]); // LITERAL-COLOR-OK: overlay cursor
    vector_scene.inner_mut().stroke(
        &Stroke::new(1.5),
        Affine::IDENTITY,
        &Brush::Solid(color),
        None,
        &path,
    );
}
