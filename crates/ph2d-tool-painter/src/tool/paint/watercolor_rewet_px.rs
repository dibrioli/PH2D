//! Watercolor **per-pixel rewet terms** — the wet-on-wet lift / dissolve / pool / backrun-ring
//! evaluation the composite runs per output pixel over the [`RewetFields`] — plus the EDGE-2
//! **union pigment fields** (the wet session-mates' pigment the WATER redisperses: ring, tint and
//! wash-lift read the UNION buffers, a session-stable source, never a refrozen base — Enio
//! 2026-07-09, "área retangular clareia a poça vizinha") and the per-owner `inner` blur set.
//! Split from `watercolor_render.rs` for the workspace LOC cap; pure functions of the sampled
//! fields (the ADR-0109 parallel-composite invariants hold: no cross-pixel state, no RNG).

use super::watercolor_field::{
    BACKRUN_POOL, LIFT_MAX, REWET_LIFT, REWET_POOL, RewetFields, SOAK_DISSOLVE, SOAK_LIFT,
    WetStrokeStyle, box_blur, sample_bilinear,
};

/// Take 10 — raio do blur do campo de molhado por-dono ([`build_wet_field`]): a transição da
/// junção soma isto à rampa do próprio depósito (~10 px) ⇒ ~15-25 px, escala do feather.
/// DELIBERADAMENTE menor que o gap do guard de não-contato (A/B a 10 px em
/// `watercolor_session_brush_changes_do_not_touch_baked_washes`): um box kernel não é geodésico —
/// raio ≥ gap vazaria molhado entre washes que nem se tocam.
const WET_FIELD_BLUR_PX: usize = 8; // LITERAL-PX-OK: raio de suavização do campo (vide doc acima)

/// Take 10: MOLHADO É CAMPO, NÃO ESTILO — o `st.wet` do dono entrava BINÁRIO nos termos
/// wet-driven, e com Rewet DIFERENTE entre traços da sessão a fronteira de dono (recency por
/// disco) imprimia um degrau de ~11 bytes em 1 px DENTRO da tinta velha (a linha dura do smoke
/// 2026-07-09; sondas [maps]/[wetmaps], doc 12). O campo é o wet-do-dono com blur MASCARADO por
/// posse (`blur(wet·m)` + `blur(m)`, divididos no sample): o molhado só se espalha entre pixels
/// POSSUÍDOS (tinta real se fundindo na junção) — nunca do brush vivo pra poça assada. Sessão de
/// UM estilo: razão de blurs iguais ⇒ byte-idêntica.
pub(super) fn build_wet_field(
    style_owner: &[u8],
    table: &[WetStrokeStyle],
    fw: usize,
    (rx0, ry0): (usize, usize),
    (rw, rh): (usize, usize),
) -> (Vec<f32>, Vec<f32>) {
    let mut wf = vec![0.0f32; rw * rh];
    let mut mask = vec![0.0f32; rw * rh];
    for wy in 0..rh {
        let gy = ry0 + wy;
        for wx in 0..rw {
            let o = style_owner[gy * fw + (rx0 + wx)];
            if o != 0 {
                wf[wy * rw + wx] = table[(o as usize - 1).min(table.len() - 1)].wet;
                mask[wy * rw + wx] = 1.0;
            }
        }
    }
    (
        box_blur(&wf, rw, rh, WET_FIELD_BLUR_PX),
        box_blur(&mask, rw, rh, WET_FIELD_BLUR_PX),
    )
}

/// O Rewet efetivo no pixel: campo mascarado ÷ massa; sem massa na vizinhança (ou sem campo) ⇒
/// o `wet` escalar do estilo do dono — o caminho antigo, exato.
#[inline]
pub(super) fn sample_wet_field(
    field: Option<&(Vec<f32>, Vec<f32>)>,
    (rw, rh): (usize, usize),
    (sx, sy): (f32, f32),
    st_wet: f32,
) -> f32 {
    field.map_or(st_wet, |(wf, m)| {
        let mass = sample_bilinear(m, rw, rh, sx, sy);
        if mass > 1e-4 {
            sample_bilinear(wf, rw, rh, sx, sy) / mass
        } else {
            st_wet
        }
    })
}

/// Resolve the per-pixel OWNER stroke's style (EDGE-1 per-stroke params — recency ownership: an
/// older wash keeps ITS Concentration/Edge/water on the union re-bake, Enio 2026-07-09). Owner `0`
/// / no style map ⇒ the current brush's style, the exact pre-style path.
#[inline]
pub(super) fn style_at(
    has_style: bool,
    owner: &[u8],
    table: &[WetStrokeStyle],
    cur: WetStrokeStyle,
    idx: usize,
) -> WetStrokeStyle {
    if has_style {
        match owner[idx] {
            0 => cur,
            o => table[(o as usize - 1).min(table.len() - 1)],
        }
    } else {
        cur
    }
}

/// One feather blur per DISTINCT per-owner `core_r` in the session (usually one): a baked wash
/// re-renders with ITS radius, never the live brush's (doc 13 "mudança no brush propaga").
pub(super) fn inner_blur_set(
    hard: &[f32],
    rw: usize,
    rh: usize,
    table: &[WetStrokeStyle],
    cur_core: usize,
) -> Vec<(usize, Vec<f32>)> {
    let mut radii: Vec<usize> = table.iter().map(|s| s.core_r as usize).collect();
    radii.push(cur_core);
    radii.sort_unstable();
    radii.dedup();
    radii
        .into_iter()
        .map(|r| (r, box_blur(hard, rw, rh, r)))
        .collect()
}

/// The blur map for one owner's `core_r` (tiny linear scan; the set is per-session-distinct).
#[inline]
pub(super) fn blur_of(blurs: &[(usize, Vec<f32>)], r: usize) -> &[f32] {
    blurs
        .iter()
        .find(|(br, _)| *br == r)
        .map_or(&blurs[0].1[..], |(_, b)| &b[..])
}

/// The rewet terms at one output pixel: how much base paint LIFTS, how much dissolved pigment
/// tints (`dissolve`, colour in `bleed`), the local raw paint presence (`wet_paint`, gates the
/// wet-driven paint-mix), the backrun ring shell (EDGE-2 — concentrates the pigment floor), how
/// and the optical-density ADDITION from the pool/bloom terms.
#[derive(Default)]
pub(super) struct RewetPx {
    pub(super) lift: f32,
    pub(super) dissolve: f32,
    pub(super) backrun: f32,
    pub(super) wet_paint: f32,
    pub(super) bleed: [f32; 3],
    pub(super) pool: f32,
}

/// Evaluate the rewet terms at the warped sample `(sx, sy)` (window-local) / serrated water read
/// `(wxg, wyg)` (global). `st_wet` = the owner stroke's Rewet; `water` = the carried-water pool
/// (EDGE-2). The dried-base terms are the former inline block verbatim — bit-identical; the
/// water block adds the UNION-sourced ring/tint/wash-lift (zero unless water met foreign wet
/// pigment, so every waterless composite is byte-identical).
#[allow(clippy::too_many_arguments)] // per-pixel kernel seam: each input is a distinct sample
pub(super) fn rewet_px(
    f: &RewetFields,
    (rx0, ry0): (usize, usize),
    (sx, sy): (f32, f32),
    (wxg, wyg): (f32, f32),
    water: f32,
    st_wet: f32,
    cw: f32,
    inner: f32,
) -> RewetPx {
    let mut out = RewetPx::default();
    let lp = f.samp(&f.pres, rx0, ry0, sx, sy);
    out.wet_paint = lp.clamp(0.0, 1.0);
    // Soak (dwell) lerps every dissolve field between the plain and the 2× blur scale and deepens
    // the lift — a lingering brush dissolves farther and digs deeper. Soak = 0 samples the near
    // fields with weight 1 (bit-identical to no-soak).
    let (s, s_raw) = if f.far.is_some() {
        (
            f.samp(&f.soak_halo, rx0, ry0, sx, sy).clamp(0.0, 1.0),
            f.samp(&f.soak_raw, rx0, ry0, sx, sy).clamp(0.0, 1.0),
        )
    } else {
        (0.0, 0.0)
    };
    let fno = f.samp(&f.near[0], rx0, ry0, sx, sy);
    let bp = if let Some(far) = &f.far {
        let ffo = f.samp(&far[0], rx0, ry0, sx, sy);
        (fno + (ffo - fno) * s).clamp(0.0, 1.0)
    } else {
        fno.clamp(0.0, 1.0)
    };
    out.lift =
        (REWET_LIFT * (st_wet * cw).max(water) * lp * (1.0 + SOAK_LIFT * s_raw)).min(LIFT_MAX);
    if bp > 1e-4 {
        let inv = 1.0 / bp;
        for c in 0..3 {
            let nc = f.samp(&f.near[c + 1], rx0, ry0, sx, sy);
            out.bleed[c] = if let Some(far) = &f.far {
                let fc = f.samp(&far[c + 1], rx0, ry0, sx, sy);
                (nc + (fc - nc) * s) * inv
            } else {
                nc * inv
            };
        }
        out.dissolve = (st_wet.max(water) * bp * (1.0 + SOAK_DISSOLVE * s)).clamp(0.0, 1.0);
        // The dissolved pigment re-enters the wash as optical density AT THE RECEDING FRONT (the
        // same rim shape as the edge term, gain-independent): pigment in suspension migrates to
        // the wet boundary — the bloom. A UNIFORM pool flooded the interior and flattened the
        // Spread dynamic (interior must CLEAR as the frontier advances).
        out.pool = REWET_POOL * st_wet * bp * (cw * (1.0 - inner)).clamp(0.0, 1.0);
    }
    // Backrun ring (EDGE-2) — water over the DRIED paint below the session (the base): the
    // redispersed pigment pools along the water's serrated contour (`raw − halo`, Curtis §2.2
    // "severely darkened edges"), scaled pela PRESENÇA da fonte (o gate sozinho flipava o CONC
    // full-strength na cauda da presença — staircase). Companheiro de sessão MOLHADO não é fonte
    // de anel/lift: ele é UM corpo d'água com este traço (fusão da união, EDGE-1) — a fonte
    // union mascarada por dono re-molhava washs assados RETROATIVAMENTE a cada pen-down novo
    // (o retângulo da janela viva com Dilution, Enio 2026-07-09).
    if water > 0.0 && bp > 1e-4 {
        let halo = f
            .samp(&f.water_halo, rx0, ry0, wxg - rx0 as f32, wyg - ry0 as f32)
            .clamp(0.0, 1.0);
        out.backrun = (water - halo).max(0.0) * bp.min(1.0);
        out.pool += BACKRUN_POOL * bp * out.backrun;
    }
    out
}
