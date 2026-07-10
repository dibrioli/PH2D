//! Watercolor **per-pixel rewet terms** — the wet-on-wet lift / dissolve / pool / backrun-ring
//! evaluation the composite runs per output pixel over the [`RewetFields`] — plus the EDGE-2
//! **union pigment fields** (the wet session-mates' pigment the WATER redisperses: ring, tint and
//! wash-lift read the UNION buffers, a session-stable source, never a refrozen base — Enio
//! 2026-07-09, "área retangular clareia a poça vizinha") and the per-owner `inner` blur set.
//! Split from `watercolor_render.rs` for the workspace LOC cap; pure functions of the sampled
//! fields (the ADR-0109 parallel-composite invariants hold: no cross-pixel state, no RNG).

use rayon::prelude::*;

use super::watercolor_field::{
    BACKRUN_POOL, LIFT_MAX, REWET_LIFT, REWET_POOL, RewetFields, SOAK_DISSOLVE, SOAK_LIFT,
    WetStrokeStyle, box_blur, smoothstep,
};
use super::watercolor_render::{SS0, SS1};

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

/// EDGE-2 **union pigment fields** — the wet session-mates' pigment presence + premultiplied
/// colour on the [`RewetFields`] low-res grid, EXCLUDING the live stroke's own deposit (owner
/// mask): the water redisperses the NEIGHBOUR washes' pigment, and a dilution-carrying stroke
/// must not ring against itself (the single-stroke path stays byte-identical).
pub(super) struct UnionFields {
    pres: Vec<f32>,
    near: [Vec<f32>; 4],
    far: Option<[Vec<f32>; 4]>,
}

impl UnionFields {
    /// Sample `(raw presence, blurred presence, colour)` at the warped window-local coord — the
    /// same near→far soak lerp as the dried-base fields.
    fn sample(
        &self,
        f: &RewetFields,
        rx0: usize,
        ry0: usize,
        sx: f32,
        sy: f32,
        s: f32,
    ) -> (f32, f32, [f32; 3]) {
        let lp = f.samp(&self.pres, rx0, ry0, sx, sy).clamp(0.0, 1.0);
        let n0 = f.samp(&self.near[0], rx0, ry0, sx, sy);
        let bp = if let Some(far) = &self.far {
            let f0 = f.samp(&far[0], rx0, ry0, sx, sy);
            (n0 + (f0 - n0) * s).clamp(0.0, 1.0)
        } else {
            n0.clamp(0.0, 1.0)
        };
        let mut bleed = [0.0f32; 3];
        if bp > 1e-4 {
            let inv = 1.0 / bp;
            for c in 0..3 {
                let nc = f.samp(&self.near[c + 1], rx0, ry0, sx, sy);
                bleed[c] = if let Some(far) = &self.far {
                    let fc = f.samp(&far[c + 1], rx0, ry0, sx, sy);
                    (nc + (fc - nc) * s) * inv
                } else {
                    nc * inv
                };
            }
        }
        (lp, bp, bleed)
    }
}

/// Build the [`UnionFields`] on `f`'s grid from the UNION buffers (session-stable): presence =
/// hardened coverage × deposited alpha, colour = the deposited bytes, both masked to owners ≠ the
/// live stroke. Blurs mirror the dried set (near = spread, far = 2× when soaked). Parallel over
/// grid rows (ADR-0109 class: pure per-cell reads, disjoint row slices ⇒ byte-identical).
#[allow(clippy::too_many_arguments)] // field-build seam: every input is a distinct buffer/flag
pub(super) fn build_union_fields(
    f: &RewetFields,
    coverage: &[u8],
    color: &[u8],
    owner: &[u8],
    cur_o: u8,
    soaked: bool,
    (fw, fh): (usize, usize),
    spread: usize,
) -> UnionFields {
    let (ds, lw, lh) = (f.ds, f.lw, f.lh);
    let half = ds / 2;
    let has_col = color.len() == coverage.len() * 4;
    let has_own = owner.len() == coverage.len();
    let mut pres = vec![0.0f32; lw * lh];
    let mut wr = vec![0.0f32; lw * lh];
    let mut wg = vec![0.0f32; lw * lh];
    let mut wb = vec![0.0f32; lw * lh];
    pres.par_chunks_mut(lw)
        .zip(wr.par_chunks_mut(lw))
        .zip(wg.par_chunks_mut(lw))
        .zip(wb.par_chunks_mut(lw))
        .enumerate()
        .for_each(|(lj, (((prow, rrow), grow), brow))| {
            let gy = (((f.loy0 + lj) * ds) + half).min(fh - 1);
            for li in 0..lw {
                let gx = (((f.lox0 + li) * ds) + half).min(fw - 1);
                let gi = gy * fw + gx;
                // Only committed session-mates count (owned, and not by the live stroke).
                if !has_own || owner[gi] == 0 || owner[gi] == cur_o {
                    continue;
                }
                let hard = smoothstep(SS0, SS1, f32::from(coverage[gi]) / 255.0);
                let ca = if has_col {
                    f32::from(color[gi * 4 + 3]) / 255.0
                } else {
                    1.0
                };
                let p = hard * ca;
                prow[li] = p;
                if has_col && p > 0.0 {
                    rrow[li] = f32::from(color[gi * 4]) * p;
                    grow[li] = f32::from(color[gi * 4 + 1]) * p;
                    brow[li] = f32::from(color[gi * 4 + 2]) * p;
                }
            }
        });
    let r1 = (spread / ds).max(1);
    let near = [
        box_blur(&pres, lw, lh, r1),
        box_blur(&wr, lw, lh, r1),
        box_blur(&wg, lw, lh, r1),
        box_blur(&wb, lw, lh, r1),
    ];
    let far = soaked.then(|| {
        let r2 = ((spread * 2) / ds).max(1);
        [
            box_blur(&pres, lw, lh, r2),
            box_blur(&wr, lw, lh, r2),
            box_blur(&wg, lw, lh, r2),
            box_blur(&wb, lw, lh, r2),
        ]
    });
    UnionFields { pres, near, far }
}

/// The rewet terms at one output pixel: how much base paint LIFTS, how much dissolved pigment
/// tints (`dissolve`, colour in `bleed`), the local raw paint presence (`wet_paint`, gates the
/// wet-driven paint-mix), the backrun ring shell (EDGE-2 — concentrates the pigment floor), how
/// much the WATER empties the wet wash's own density (`lift_wash` — the redispersed mass that
/// re-enters as the ring), and the optical-density ADDITION from the pool/bloom terms.
#[derive(Default)]
pub(super) struct RewetPx {
    pub(super) lift: f32,
    pub(super) dissolve: f32,
    pub(super) backrun: f32,
    pub(super) wet_paint: f32,
    pub(super) bleed: [f32; 3],
    pub(super) pool: f32,
    pub(super) lift_wash: f32,
}

/// Evaluate the rewet terms at the warped sample `(sx, sy)` (window-local) / serrated water read
/// `(wxg, wyg)` (global). `st_wet` = the owner stroke's Rewet; `water` = the carried-water pool
/// (EDGE-2). The dried-base terms are the former inline block verbatim — bit-identical; the
/// water block adds the UNION-sourced ring/tint/wash-lift (zero unless water met foreign wet
/// pigment, so every waterless composite is byte-identical).
#[allow(clippy::too_many_arguments)] // per-pixel kernel seam: each input is a distinct sample
pub(super) fn rewet_px(
    f: &RewetFields,
    u: Option<&UnionFields>,
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
    // Backrun (EDGE-2) — water over pigment, DRIED (below the session) or WET (a session-mate,
    // via the union fields): the redispersed pigment pools along the water's serrated contour
    // (`raw − halo`, a shell just inside the jagged boundary — Curtis §2.2 "severely darkened
    // edges"), tints the wash, and EMPTIES the wet wash it came from (`lift_wash`).
    if water > 0.0 {
        let (_lp_u, bp_u, bleed_u) = match u {
            Some(u) => u.sample(f, rx0, ry0, sx, sy, s),
            None => (0.0, 0.0, [0.0; 3]),
        };
        // BLURRED presence (bp_u, já amostrada pro anel — custo zero): a crua (lp_u) degrauza
        // 0→1 em 1 px na linha de cobertura endurecida do wash vizinho — o esvaziamento ligava
        // num pixel e a costura seguia a silhueta original do wash DENTRO da água do traço novo
        // (Enio smoke 2026-07-09, cruz rápida: linha nítida nas junções topo/fundo).
        out.lift_wash = (REWET_LIFT * water * bp_u * (1.0 + SOAK_LIFT * s_raw)).min(LIFT_MAX);
        let bp_ring = bp.max(bp_u);
        if bp_ring > 1e-4 {
            let halo = f
                .samp(&f.water_halo, rx0, ry0, wxg - rx0 as f32, wyg - ry0 as f32)
                .clamp(0.0, 1.0);
            // Escala pela PRESENÇA do pigmento-fonte: o gate `bp_ring > 1e-4` sozinho flipava o
            // deepen do CONC em força total no pixel em que abre (staircase de 38 bytes medido).
            out.backrun = (water - halo).max(0.0) * bp_ring.min(1.0);
            out.pool += BACKRUN_POOL * bp_ring * out.backrun;
            // Union tint: the neighbour's RAW pigment bleeds into the pool (weight-merged with
            // the dried-base tint; zero union presence ⇒ the dried path bit-exact).
            let du = (water * bp_u * (1.0 + SOAK_DISSOLVE * s)).clamp(0.0, 1.0);
            if du > 0.0 {
                let wsum = out.dissolve + du;
                for (b, u) in out.bleed.iter_mut().zip(bleed_u) {
                    *b = (*b * out.dissolve + u * du) / wsum;
                }
                out.dissolve = out.dissolve.max(du);
            }
        }
    }
    out
}
