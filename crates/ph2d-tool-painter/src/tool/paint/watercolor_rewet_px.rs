//! Watercolor **per-pixel rewet terms** — the wet-on-wet lift / dissolve / pool / backrun-ring
//! evaluation the composite runs per output pixel over the [`RewetFields`] — plus the EDGE-2
//! **union pigment fields** (the wet session-mates' pigment the WATER redisperses: ring, tint and
//! wash-lift read the UNION buffers, a session-stable source, never a refrozen base — Enio
//! 2026-07-09, "área retangular clareia a poça vizinha") and the per-owner `inner` blur set.
//! Split from `watercolor_render.rs` for the workspace LOC cap; pure functions of the sampled
//! fields (the ADR-0109 parallel-composite invariants hold: no cross-pixel state, no RNG).

use super::watercolor_field::{
    BACKRUN_POOL, LIFT_MAX, Luts, NoiseTile, REWET_LIFT, REWET_POOL, RewetFields, SOAK_DISSOLVE,
    SOAK_LIFT, WetStrokeStyle, box_blur, paper_h_px, sample_bilinear,
};
use ph2d_painter_brush::TextureSettings;
use ph2d_painter_brush::texture::{ImageMask, angle_basis, sample_tiled_rot};

/// Fill the canvas-anchored `paper_h` cache for the region's MISSES (`NaN`), serially, so the parallel
/// composite reads it immutably (perf memo; single-substrate only — the caller gates on `!multi()`).
/// A FREE fn, not a method: it borrows only the `substrate` slice, disjoint from the composite's live
/// `paper_img` borrow of `self` (a `&mut self` method would alias it). Split from the composite (LOC).
/// `region = (x0, y0, bw, bh)` in canvas px. Seamless Tiling wraps the built-in noise via `tile` (#2).
#[allow(clippy::too_many_arguments)]
pub(super) fn fill_substrate_cache(
    substrate: &mut [f32],
    paper_active: bool,
    paper_tex: &TextureSettings,
    paper_img: Option<&ImageMask>,
    paper_rot: [f32; 2],
    region: (usize, usize, usize, usize),
    fw: usize,
    tile: NoiseTile,
) {
    let (x0, y0, bw, bh) = region;
    for by in 0..bh {
        let gy = y0 + by;
        for bx in 0..bw {
            let sidx = gy * fw + (x0 + bx);
            if substrate[sidx].is_nan() {
                substrate[sidx] = paper_h_px(
                    paper_active,
                    paper_tex,
                    paper_img,
                    paper_rot,
                    x0 + bx,
                    gy,
                    tile,
                );
            }
        }
    }
}

/// Wet-on-wet **LIFT** applied to the (linear) session base `sb`: rewetting pulls the base's pigment off
/// toward the LOCAL `ground`, density-proportional in log space, so a lifted red on white reads pink and
/// on grey reads grey-pink (both directions converge on the ground, never past it). `lift = 0` ⇒ no-op.
/// Extracted from the composite for the LOC cap; the `lift` the caller passes is already moisture-scaled
/// (#12b: a dried spot doesn't reactivate).
#[inline]
pub(super) fn apply_wet_lift(sb: &mut [f32; 3], ground_lin: &[f32; 3], lift: f32, lut: &Luts) {
    if lift <= 0.0 {
        return;
    }
    for c in 0..3 {
        let g = ground_lin[c].max(1e-4);
        let ratio = sb[c] / g;
        if ratio < 1.0 {
            let mag = lut.absorbance(ratio) * (1.0 - lift);
            sb[c] = g * lut.exp_mag(mag);
        } else if ratio > 1.0 {
            let mag = lut.absorbance((g / sb[c]).clamp(0.0, 1.0)) * (1.0 - lift);
            sb[c] = (g / lut.exp_mag(mag).max(1e-4)).min(1.0);
        }
    }
}

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

/// Per-owner **continuous** wash params (fill/depth/edge_gain/opacity/warp) smoothed ACROSS the owner
/// boundary — the [`build_wet_field`] treatment (`blur(v·m)/blur(m)`) generalised to the params that feed
/// CONTINUOUS terms (#18, Bug #8 lição #4). Read DISCRETELY via [`style_at`], they STEP at the junction:
/// changing Body/Concentration/Edge/Opacity/RaggedEdge and crossing a still-wet neighbour prints a hard
/// edge (and the new stroke's Warp re-warps the OLD wash's boundary). Built ONLY when the owners' params
/// actually differ ([`params_differ`]) — uniform ⇒ the discrete path, byte-identical. Window-local layout.
pub(super) struct StyleField {
    fill: Vec<f32>,
    depth: Vec<f32>,
    edge_gain: Vec<f32>,
    opacity: Vec<f32>,
    warp: Vec<f32>,
    mask: Vec<f32>,
    rw: usize,
    rh: usize,
}

/// Do any two OWNED strokes (the table) differ in a continuous param? If not, the discrete resolution is
/// already seamless — skip the field (byte-identical). The current stroke IS the last table entry.
pub(super) fn params_differ(table: &[WetStrokeStyle]) -> bool {
    let d = |a: &WetStrokeStyle, b: &WetStrokeStyle| {
        a.fill != b.fill
            || a.depth != b.depth
            || a.edge_gain != b.edge_gain
            || a.opacity != b.opacity
            || a.warp != b.warp
    };
    table
        .first()
        .is_some_and(|first| table.iter().any(|s| d(s, first)))
}

pub(super) fn build_style_field(
    style_owner: &[u8],
    table: &[WetStrokeStyle],
    fw: usize,
    (rx0, ry0): (usize, usize),
    (rw, rh): (usize, usize),
) -> StyleField {
    let n = rw * rh;
    let mut fill = vec![0.0f32; n];
    let mut depth = vec![0.0f32; n];
    let mut edge_gain = vec![0.0f32; n];
    let mut opacity = vec![0.0f32; n];
    let mut warp = vec![0.0f32; n];
    let mut mask = vec![0.0f32; n];
    for wy in 0..rh {
        let gy = ry0 + wy;
        for wx in 0..rw {
            // ONLY owned pixels (a real wash) contribute — masked by ownership like `build_wet_field`, so
            // an unowned GAP never leaks the current brush's params into a neighbour that doesn't touch it
            // (the non-contact guard; else two non-overlapping washes would bleed across the gap).
            let o = style_owner[gy * fw + (rx0 + wx)];
            if o == 0 {
                continue;
            }
            let s = &table[(o as usize - 1).min(table.len() - 1)];
            let i = wy * rw + wx;
            fill[i] = s.fill;
            depth[i] = s.depth;
            edge_gain[i] = s.edge_gain;
            opacity[i] = s.opacity;
            warp[i] = s.warp;
            mask[i] = 1.0;
        }
    }
    let r = WET_FIELD_BLUR_PX;
    StyleField {
        fill: box_blur(&fill, rw, rh, r),
        depth: box_blur(&depth, rw, rh, r),
        edge_gain: box_blur(&edge_gain, rw, rh, r),
        opacity: box_blur(&opacity, rw, rh, r),
        warp: box_blur(&warp, rw, rh, r),
        mask: box_blur(&mask, rw, rh, r),
        rw,
        rh,
    }
}

impl StyleField {
    /// Smoothed (fill, depth, edge_gain, opacity) at the WARPED window-local `(sx, sy)` — same sample
    /// point as the wet field + the discrete `st`. `fb` (the discrete values) covers a mass-less miss.
    #[inline]
    pub(super) fn sample(
        &self,
        sx: f32,
        sy: f32,
        fb: (f32, f32, f32, f32),
    ) -> (f32, f32, f32, f32) {
        let m = sample_bilinear(&self.mask, self.rw, self.rh, sx, sy);
        if m > 1e-4 {
            (
                sample_bilinear(&self.fill, self.rw, self.rh, sx, sy) / m,
                sample_bilinear(&self.depth, self.rw, self.rh, sx, sy) / m,
                sample_bilinear(&self.edge_gain, self.rw, self.rh, sx, sy) / m,
                sample_bilinear(&self.opacity, self.rw, self.rh, sx, sy) / m,
            )
        } else {
            fb
        }
    }

    /// Smoothed Warp AMPLITUDE at the PRE-warp window-local `(lx, ly)` (the displacement needs the amp
    /// first, so it reads un-warped — matching the discrete `st_warp`). `fb` covers a mass-less miss.
    #[inline]
    pub(super) fn sample_warp(&self, lx: f32, ly: f32, fb: f32) -> f32 {
        let m = sample_bilinear(&self.mask, self.rw, self.rh, lx, ly);
        if m > 1e-4 {
            sample_bilinear(&self.warp, self.rw, self.rh, lx, ly) / m
        } else {
            fb
        }
    }
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

/// The Paper/Grain **SUBSTRATE** resolver for one composite pass (doc 14 #13, smoke 2026-07-10).
///
/// A **single-substrate** session (every owner shares the paper/grain SETTINGS) resolves to the live
/// brush's globals — byte-identical to the old inline block, and the per-canvas-pixel substrate cache
/// stays live. A **multi-substrate** session (the user changed Paper Kind / Same-as-Paper / Grain
/// mid-session) has each pixel resolve its paper/grain from its OWNER style, so a baked wash keeps ITS
/// substrate instead of being re-textured by the current brush — the "aplica a tudo" + rectangles
/// bug. The loaded paper/grain IMAGES stay session-shared in v1 (only the SETTINGS are per-owner,
/// which covers the reported triggers: procedural Paper Kind · Same as Paper · Grain Amount).
pub(super) struct SubstrateSession<'a> {
    multi: bool,
    // Effective globals (derived from the current style = the live brush).
    paper_active: bool,
    paper: TextureSettings,
    paper_rot: [f32; 2],
    paper_depth: f32,
    gran_own_map: bool,
    gran: TextureSettings,
    gran_rot: [f32; 2],
    gran_use_paper: bool,
    paper_img: Option<ImageMask<'a>>,
    gran_img: Option<ImageMask<'a>>,
    // Per-owner Angle bases (multi only; index 0 = current brush, k = table[k-1]).
    paper_rots: Vec<[f32; 2]>,
    gran_rots: Vec<[f32; 2]>,
    // Seamless Tiling (doc 13 #2): periodic sprite wrap for the built-in paper granulation noise.
    tile: NoiseTile,
}

impl<'a> SubstrateSession<'a> {
    /// Build from the current style (the live brush's substrate) + the session style table.
    pub(super) fn build(
        cur: &WetStrokeStyle,
        table: &[WetStrokeStyle],
        paper_img: Option<ImageMask<'a>>,
        gran_img: Option<ImageMask<'a>>,
        tile: NoiseTile,
    ) -> Self {
        // Multi iff any owner's substrate differs from the first (the current stroke's style is IN
        // the table — pushed at pen-down — so the live brush is covered; unowned pixels are outside
        // the wash / restored, so comparing table entries is sufficient).
        let multi = !table.is_empty() && {
            let f = &table[0];
            table.iter().any(|s| {
                s.paper != f.paper
                    || s.paper_depth != f.paper_depth
                    || s.granulation_use_paper != f.granulation_use_paper
                    || s.texture != f.texture
            })
        };
        let (paper_rots, gran_rots) = if multi {
            let mut pr = Vec::with_capacity(table.len() + 1);
            let mut gr = Vec::with_capacity(table.len() + 1);
            pr.push(angle_basis(cur.paper.angle_deg)); // owner 0 = current brush
            gr.push(angle_basis(cur.texture.angle_deg));
            for s in table {
                pr.push(angle_basis(s.paper.angle_deg));
                gr.push(angle_basis(s.texture.angle_deg));
            }
            (pr, gr)
        } else {
            (Vec::new(), Vec::new())
        };
        Self {
            multi,
            paper_active: cur.paper.is_active(),
            paper: cur.paper,
            paper_rot: angle_basis(cur.paper.angle_deg),
            paper_depth: cur.paper_depth,
            gran_own_map: !cur.granulation_use_paper && cur.texture.is_active(),
            gran: cur.texture,
            gran_rot: angle_basis(cur.texture.angle_deg),
            gran_use_paper: cur.granulation_use_paper,
            paper_img,
            gran_img,
            paper_rots,
            gran_rots,
            tile,
        }
    }

    /// True when owners have differing substrates → the per-canvas-pixel substrate cache is invalid
    /// (the caller must disable it and let [`Self::at`] recompute per owner).
    #[inline]
    pub(super) fn multi(&self) -> bool {
        self.multi
    }

    /// `(paper_h, gran_h, paper_component)` at one pixel. `cached_paper_h` = the memoised substrate
    /// value when the cache is live (single-substrate only); `None` recomputes from the owner's paper.
    pub(super) fn at(
        &self,
        st: &WetStrokeStyle,
        owner: u8,
        cached_paper_h: Option<f32>,
        gx: usize,
        gy: usize,
    ) -> (f32, Option<f32>, f32) {
        let (
            paper_active,
            paper,
            paper_rot,
            paper_depth,
            gran_own_map,
            gran,
            gran_rot,
            gran_use_paper,
        ) = if self.multi {
            let o = owner as usize;
            (
                st.paper.is_active(),
                &st.paper,
                self.paper_rots.get(o).copied().unwrap_or(self.paper_rot),
                st.paper_depth,
                !st.granulation_use_paper && st.texture.is_active(),
                &st.texture,
                self.gran_rots.get(o).copied().unwrap_or(self.gran_rot),
                st.granulation_use_paper,
            )
        } else {
            (
                self.paper_active,
                &self.paper,
                self.paper_rot,
                self.paper_depth,
                self.gran_own_map,
                &self.gran,
                self.gran_rot,
                self.gran_use_paper,
            )
        };
        let paper_h = cached_paper_h.unwrap_or_else(|| {
            paper_h_px(
                paper_active,
                paper,
                self.paper_img.as_ref(),
                paper_rot,
                gx,
                gy,
                self.tile,
            )
        });
        let gran_h = if gran_own_map {
            Some(sample_tiled_rot(
                gran,
                gx as i64,
                gy as i64,
                self.gran_img.as_ref(),
                gran_rot,
            ))
        } else if gran_use_paper {
            Some(paper_h)
        } else {
            None
        };
        let paper_component = if paper_active {
            (paper_h - 0.5) * paper_depth
        } else {
            0.0
        };
        (paper_h, gran_h, paper_component)
    }
}
