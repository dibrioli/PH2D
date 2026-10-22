//! **A LANE DE CADA DAB** — o casamento geométrico de um dab com a lane da sua cópia, a tinta na língua do motor e as
//! contas por dab da rota (filho de [`super`], LOC cap, a partir de `dab_route.rs`).

use super::*;

/// The engine speaks sRGB **0..255** (its boot colour is `[50, 140, 210]` and the render
/// writes plane values straight through `clamp_u8`); the dab stores sRGB 0..1. Passing the
/// normalized value painted BLACK — Enio's W1 smoke, "a cor está preta".
pub(super) fn tinta(c: [f32; 3]) -> [f64; 3] {
    [
        f64::from(c[0]) * 255.0,
        f64::from(c[1]) * 255.0,
        f64::from(c[2]) * 255.0,
    ]
}

/// ⭐ O pincel de cerdas FINO esmaece (`Knob::ThinStrokeFade`, J4 óleo): `(D ÷ ref)^γ` abaixo do diâmetro de
/// referência — `γ = 0` = 1, a lei de sempre ao bit.
pub(super) fn esmaece_fino(engine: &ph2d_wet_paint::painter::Engine, brush: &BrushSpec) -> f64 {
    let g = engine
        .tuning
        .get(ph2d_wet_paint::tuning::Knob::ThinStrokeFade);
    if g > 0.0 {
        let r = engine
            .tuning
            .get(ph2d_wet_paint::tuning::Knob::ThinStrokeRef)
            .max(1.0);
        (2.0 * f64::from(brush.clamped_radius()) / r)
            .clamp(0.0, 1.0)
            .powf(g)
    } else {
        1.0
    }
}

/// LANE matching, geometric: the dab belongs to the lane whose
/// last position is within its own radius (consecutive dabs of one
/// copy are a spacing apart; other copies are far). A dab with no
/// lane in reach BEGINS one — a symmetry copy at stroke start, or
/// a Tiling wrap born mid-stroke at the sprite edge. Near a radial
/// centre the copies converge and may swap lanes; there their
/// positions coincide, so a swap deposits the same paint.
/// The ERASER skips the lanes entirely (it never calls this) — every copy just erases
/// where it lands. The direct wet tools keep the geometric lane
/// MATCHING (it is what carries each symmetry/tiling copy's own
/// previous dab centre — the smear/blow displacement source) but
/// never touch the engine's lane trails.
///
/// Devolve `(lane, nasceu)`.
pub(super) fn casa_a_lane(
    sess: &mut WetSession,
    d: &Dab,
    brush: &BrushSpec,
    copia: u32,
    wet_tool: WetTool,
) -> (usize, bool) {
    let ratio = sess.ratio;
    let [x, y] = d.center;
    let thr = brush.limiar_da_lane(d.radius_px);
    let mut best = thr * thr;
    let mut lane = None;
    for (i, l) in sess.lanes.iter().enumerate() {
        let (ddx, ddy) = (x - l.pos[0], y - l.pos[1]);
        let d2 = ddx * ddx + ddy * ddy;
        if l.copia == copia && d2 <= best {
            best = d2;
            lane = Some(i);
        }
    }
    match lane {
        Some(i) => {
            if wet_tool.uses_lanes() {
                // A janela do rasto anda pela CORDA entre centros: pelo sub-passo antes de arredondar, o
                // dab do resto pesava um sub-passo inteiro — a escada do Oil Bristle a 103 px (doc 51 §4).
                let chord = grid_map::px_len_to_cell(f64::from(best.sqrt()), ratio);
                let arco = (d.arc_len - sess.lanes[i].arco).max(0.0);
                let caminho = grid_map::px_len_to_cell(f64::from(arco), ratio);
                sess.engine.direct_segment_no_caminho(i, chord, caminho);
            }
            (i, false)
        }
        None => {
            let i = sess.lanes.len();
            if wet_tool.uses_lanes() {
                // The DAB's colour, not the brush's: Randomize
                // is already resolved per dab by the stroke
                // engine (W2.2).
                sess.engine.color = tinta(d.color);
                sess.engine.begin_direct_stroke(
                    i,
                    grid_map::px_to_cell(f64::from(x), ratio),
                    grid_map::px_to_cell(f64::from(y), ratio),
                );
            }
            sess.lanes.push(Lane {
                pos: d.center,
                ink: d.color,
                arco: d.arc_len,
                copia,
            });
            (i, true)
        }
    }
}

/// O `b` do motor e a opacidade do que pousa, por dab. ⭐ A pressão → OPACIDADE (prompt 45): com o pincel que pinta,
/// o `b` é o do toque inteiro e a fracção da pressão vai para a opacidade. ⭐ O FALL-OFF (L5) no Wet é a carga do
/// PIGMENTO (o que pousa), não a pressão: a cobertura do dab já o leva (`Taper::fall`), e no `b` da pressão ele
/// desfaz-se para pousar na opacidade. Neutro = `1`.
pub(super) fn b_e_opacidade(
    d: &Dab,
    brush: &BrushSpec,
    strength: f32,
    opacidade_da_pressao: Option<f32>,
    wet_tool: WetTool,
) -> (f64, f64) {
    let (b, opacidade) = match opacidade_da_pressao {
        Some(ov) if wet_tool == WetTool::Paint => (
            f64::from(ov) * 10.0,
            (f64::from(d.coverage) / (f64::from(strength) * f64::from(ov)).max(1e-9))
                .clamp(0.0, 1.0),
        ),
        _ => (((d.coverage / strength).clamp(0.0, 1.0) as f64) * 10.0, 1.0),
    };
    let carga = f64::from(brush.taper.fall(d.arc_len, 2.0 * brush.clamped_radius()));
    if carga < 1.0 && !(opacidade_da_pressao.is_some() && wet_tool == WetTool::Paint) {
        ((b / carga.max(1e-6)).min(10.0), opacidade * carga)
    } else {
        (b, opacidade)
    }
}

/// O estado do fluxo do sub-dab `k` da cápsula, tirado do do dab (o splitmix64).
pub(super) fn sorteio_do_sub_dab(rng: u64, k: u64) -> u64 {
    let mut z = (rng ^ k.wrapping_mul(0xD1B5_4A32_D192_ED03)).wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
