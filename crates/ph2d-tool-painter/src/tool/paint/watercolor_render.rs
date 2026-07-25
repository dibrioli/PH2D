//! Watercolor **optical render-path** (the wet-media look, `docs/Painter/10_aquarela_render_path_preset_papers.md`).
//!
//! In watercolor mode the normal per-dab deposit is **skipped**; the stroke instead accumulates a
//! coverage mask ([`PaintState::stroke_coverage`](super::PaintState)) and a deposited-colour buffer
//! ([`PaintState::stroke_color`](super::PaintState)), and the whole appearance is **reconstructed
//! optically** each frame over a frozen base — exactly the architecture of
//! `docs/Painter/wet_edges_paint.html` (one optical field, not stacked stamps).
//!
//! The model, per pixel (over the frozen base `B`, all in **linear light**):
//! ```text
//!   cover = smoothstep(SS0, SS1, coverage(warp(x,y)))     // hardened, warped silhouette
//!   inner = blur(coverage)                                 // ~1 inside, →0 at the rim
//!   edge  = clamp(cover·(1 − inner)·edge_gain, 0, 1)       // pigment pooled at the receding front
//!   gran  = 1 + (paperHeight − 0.5)·2·granulation          // paper-tooth granulation (value noise)
//!   D     = (cover·fill + edge)·gran                        // optical density
//!   Tᵢ    = pigmentᵢ^(D·depth)                              // Beer–Lambert transmittance per channel
//!   outᵢ  = l2s( s2l(Bᵢ)·Tᵢ + s2l(pigmentᵢ)·(1 − Tᵢ) )     // base attenuated + pigment scattered
//! ```
//! All per-pixel math is table lookups + sums/mults (HR-5) — LUTs in [`super::watercolor_lut`], the
//! integer-hash value noise in [`super::watercolor_field`]; no transcendental runs in the hot loop.

mod window;

use super::watercolor_field::*;
use super::watercolor_rewet_px::{
    apply_wet_lift, blur_of, build_style_field, build_wet_field, inner_blur_set, params_differ,
    rewet_px, sample_wet_field, style_at,
};
use super::*;
use ph2d_painter_brush::blend::ryb_mix;
use rayon::prelude::*;

/// Hardened-coverage smoothstep edges (wet_edges `SS0`/`SS1`): below `SS0` the wash is transparent,
/// above `SS1` fully covered — a crisp-but-soft silhouette from the feathered coverage discs.
pub(super) const SS0: f32 = 0.12; // LITERAL-PX-OK: coverage-hardening smoothstep low edge (wet_edges)
pub(super) const SS1: f32 = 0.60; // LITERAL-PX-OK: coverage-hardening smoothstep high edge (wet_edges)

impl PainterTool {
    /// Composite the watercolor wash over the frozen base ([`PaintState::watercolor_base`](super::PaintState)).
    ///
    /// Reads the coverage + deposited-colour buffers, reconstructs the optical density `D`, and applies
    /// per-channel Beer–Lambert in linear light — see the module docs. The base is a separate `Arc` (each
    /// frame recomposites from the pristine pre-stroke pixels, no overlay peel); `commit` drops it at the
    /// pen-up bake (inside the undo transaction), the live passes keep it for the next frame.
    ///
    /// **Dirty-rect (wet_edges `renderFrame`/`endStroke`):** live passes recomposite ONLY the frame
    /// dirty rect (dabs since the last composite) padded by the influence radius; the pen-up bake makes
    /// one cumulative pass from a tracked bbox — never a full scan. Returns the region (`None` = no-op).
    pub(super) fn apply_watercolor(&mut self, commit: bool) -> Option<Region> {
        // Window arithmetic (frozen bases + dirty-rect consumption + influence
        // padding + read window) — moved verbatim to [`window::WashWindow`];
        // `None` = nothing to composite, and the commit still drops the base.
        let Some(w) = self.wash_window(commit) else {
            if commit {
                self.paint.watercolor_base = None;
            }
            return None;
        };
        let window::WashWindow {
            fw,
            fh,
            n,
            base_arc,
            backdrop_arc,
            spread,
            warp_amp,
            wet,
            core_r,
            wet_any,
            spread_any,
            soaked,
            watered,
            x0,
            y0,
            y1,
            bw,
            bh,
            region,
            rx0,
            ry0,
            rx1,
            ry1,
            rw,
            rh,
        } = w;

        // Window-local coverage (`[0,1]`) + its blur (reads the true cumulative coverage, so the blur
        // feathers correctly at the real rim, including coverage from earlier frames).
        let mut cov_src = vec![0.0f32; rw * rh];
        for wy in 0..rh {
            let sbase = (ry0 + wy) * fw + rx0;
            let dbase = wy * rw;
            for wx in 0..rw {
                cov_src[dbase + wx] = f32::from(self.paint.stroke_coverage[sbase + wx]) / 255.0;
            }
        }
        // The feather blur (`inner`) must SATURATE to ~1 in a pool's core — `core_r` caps it at
        // ~half the brush (a wider Spread otherwise read the whole pool as "rim" and the edge
        // flooded the centre, Enio 2026-07-07); `spread ≤ radius·½` is a no-op. EDGE-3: blur of
        // the HARDENED coverage (the raw plateau shifted the interior tone with Edge). One blur
        // per DISTINCT per-owner core_r (usually one): a baked wash keeps ITS feather — the live
        // brush's radius re-blurred the neighbour's rim (doc 13 "mudança no brush propaga").
        let hard: Vec<f32> = cov_src.iter().map(|&c| smoothstep(SS0, SS1, c)).collect();
        let blurs = inner_blur_set(&hard, rw, rh, &self.paint.wet_styles.table, core_r);

        let lut = luts();
        let brush = &self.paint.brush;
        let base = &**base_arc;
        let ground = &*backdrop_arc;

        // ── Wet-on-wet rewetting (`wet_rewet`, Enio 2026-07-06): old paint LIFTS toward the
        // ground, DISSOLVES through the wet region (blur radius = spread, 2× where the brush
        // lingered) and POOLS back at the rim. `wet = 0` skips it all (byte-identical, zero cost).
        // The fields read the SESSION base — the dried paint BELOW the whole session. Session-
        // mates are NOT "old paint" (live water, merged by the union re-render); the refrozen
        // per-stroke base poisoned reproducibility — the baked neighbour re-rendered through
        // dissolve/pool/mix as a lightening rectangle (Enio 2026-07-09). The water's redispersion
        // of session-mate pigment comes from the UNION fields below instead.
        let rewet = (wet_any > 0.0 || watered).then(|| {
            build_rewet_fields(
                &base_arc[..],
                ground,
                &self.paint.wet_soak,
                soaked,
                &self.paint.stroke_water,
                watered,
                (fw, fh),
                (rx0, ry0, rx1, ry1),
                spread_any,
            )
        });
        let cur_o = self.paint.wet_styles.current_owner();

        let fill = brush.fill.clamp(0.0, 1.0);
        let depth = brush.depth.max(0.0);
        let edge_gain = brush.edge_gain.max(0.0);
        // "Smooth Edges" (BUGS #16): the AA mode is read ONCE per composite from the live brush —
        // the whole union renders consistently (a per-owner mix would seam at the rim where two
        // strokes' silhouettes meet); committed washes keep the bytes they baked with.
        let smooth_edges = brush.smooth_edges;
        // Interior-thinning multiplier: 1.0 at/below the reference Spread, rising with Spread.
        let spread_thin = (1.0 + (spread as f32 - SPREAD_THIN_REF).max(0.0) / SPREAD_THIN_REF)
            .min(SPREAD_THIN_MAX);
        let granulation = brush.granulation.clamp(0.0, 1.0);
        let pigment_mix = brush.effective_pigment_mix();
        // ── Paper (substrate tooth) + Granulation (mineral settling) — two canvas-anchored
        // slots: the Paper textures the wash subtly; the Granulation settles by Amount into its
        // OWN map or (Same as Paper, default) the paper's tooth. Inactive ⇒ built-in noise.
        // Seamless Tiling (doc 13 #2): the sprite-period context for the canvas-anchored PROCEDURAL
        // noise (RaggedEdge warp, paper granulation, backrun jag — wrapped at the sprite period below)
        // AND the slot-texture snap. Tiling off ⇒ `NoiseTile::NONE` ⇒ byte-identical.
        let noise_tile = if self.paint.tiling[0] || self.paint.tiling[1] {
            NoiseTile::new((fw, fh), self.paint.tiling)
        } else {
            NoiseTile::NONE
        };
        // #2b (doc 13): snap a BITMAP slot's Size (Image or a baked Paper preset) so a whole number of
        // tiles spans the sprite → it repeats seamlessly across the seam (matching the procedural noise).
        // Lattice procedurals aren't snapped (see `snap_slot_size`). No-op off-tiling ⇒ byte-identical.
        let paper_tex = snap_slot_size(brush.paper, noise_tile);
        let paper_active = paper_tex.is_active();
        let paper_img = self.paint.paper_image.as_ref().map(|i| i.as_mask());
        // Precompute each slot's Angle rotation basis ONCE (the per-degree walk is not per-pixel-cheap).
        let paper_rot = ph2d_painter_brush::texture::angle_basis(paper_tex.angle_deg);
        // The Granulation map is the **Grain** slot (`brush.texture`) — used only when "Same as Paper"
        // is off; otherwise the granulation settles into the paper's own tooth. (`gran_own_map` +
        // `gran_rot` moved into `SubstrateSession`, which resolves them per owner — #13.)
        // Match the brush's grain SCALE (Enio 2026-07-11): rescale a ViewPlane Grain so the wash's
        // canvas-anchored sample reproduces the brush's feature scale, then snap for the tiling seam as
        // before (see `grain_view_to_canvas_size` for the why — the ~256/radius× coarsening bug).
        let gran_tex = snap_slot_size(
            grain_view_to_canvas_size(brush.texture, brush.radius_px),
            noise_tile,
        );
        let gran_img = self.paint.texture_image.as_ref().map(|i| i.as_mask());
        let paper_depth = brush.paper_depth.clamp(0.0, 1.0);
        // Fallback pigment when the colour buffer is faint (straight brush colour → sRGB bytes).
        let fallback = [
            (brush.color[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (brush.color[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (brush.color[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        ];
        let has_color = self.paint.stroke_color.len() == n * 4;
        // Manual textured tip (doc 13 #1 round 3): the per-stroke tip-density buffer scales the
        // interior fill (`cw·fill·dens`) — the tip's texture reads as pigment variation INSIDE a
        // normally-wet wash (water fills the tip's silhouette; texture modulates the deposit).
        // Empty (every non-textured path) ⇒ density ≡ 1 → byte-identical.
        let has_dens = self.paint.stroke_density.len() == n;
        let dens_buf = &self.paint.stroke_density;
        // Wet Mix pigment reserve (MIX-1): scales the whole BRUSH density term (fill + edge) AFTER
        // the rim is derived from the intact coverage — Charge depletion fades the pigment while the
        // water footprint (and so the edge anatomy) stays whole. Empty ⇒ factor ≡ 1 → byte-identical.
        let has_depl = self.paint.stroke_deplete.len() == n;
        let depl_buf = &self.paint.stroke_deplete;
        // EDGE-1 per-stroke style: owner map + table ([`WetSessionStyles`]); `cur_style` mirrors
        // the clamped globals above, so unowned pixels and style-less composites resolve to the
        // EXACT same values as before (bit-identical single-style path).
        let style_table = &self.paint.wet_styles.table;
        let style_owner = &self.paint.wet_styles.owner;
        let has_style = style_owner.len() == n && !style_table.is_empty();
        let cur_style = WetStrokeStyle {
            fill,
            depth,
            opacity: brush.opacity.clamp(0.0, 1.0),
            edge_gain,
            wet,
            granulation,
            warp: warp_amp,
            pigment_mix,
            color: fallback,
            spread_thin,
            core_r: core_r as u16,
            spread_px: spread as u16,
            paper: paper_tex,
            paper_depth,
            granulation_use_paper: brush.granulation_use_paper,
            texture: gran_tex,
        };
        // #13 (doc 14): per-owner SUBSTRATE. Single-substrate sessions resolve to the globals
        // (byte-identical + cache live); a multi-substrate session (paper/grain changed mid-session)
        // resolves paper/grain per OWNER so a baked wash keeps ITS substrate (kills "aplica a tudo").
        let substrate_session = watercolor_rewet_px::SubstrateSession::build(
            &cur_style,
            style_table,
            paper_img,
            gran_img,
            noise_tile,
        );
        // Raw per-pixel soak for the granulation settle (GRAN-1) — read-only in the parallel loop.
        let soak_buf = &self.paint.wet_soak;
        let water_buf = &self.paint.stroke_water;
        // Take 10: MOLHADO É CAMPO, NÃO ESTILO ([`build_wet_field`], doc lá) — suaviza a
        // fronteira de dono nos termos wet-driven; sem rewet/água/estilos nem constrói.
        let wet_field = ((wet_any > 0.0 || watered) && has_style && style_owner.len() == n)
            .then(|| build_wet_field(style_owner, style_table, fw, (rx0, ry0), (rw, rh)));
        // #18: continuous wash params smoothed across the owner boundary — only when the owners' params
        // actually differ (else `None` ⇒ discrete `st`, byte-identical). See `build_style_field`.
        let style_field = (has_style && style_owner.len() == n && params_differ(style_table))
            .then(|| build_style_field(style_owner, style_table, fw, (rx0, ry0), (rw, rh)));

        let color_buf = &self.paint.stroke_color;
        // Substrate memoisation (perf, byte-identical): `paper_h` is canvas-anchored, so compute
        // once per canvas pixel ([`paper_h_px`], the loop's exact former expression) and reuse
        // across frames + the bake; pre-pass fills misses serially so the parallel loop reads
        // immutably. Un-sized cache (defensive) ⇒ the loop falls back to the direct call. Multi-
        // substrate ⇒ the cache is invalid (it assumes ONE paper); disable it so the loop resolves
        // paper/grain per owner (see `SubstrateSession`).
        let use_substrate_cache = self.paint.wet_substrate.len() == n && !substrate_session.multi();
        if use_substrate_cache {
            watercolor_rewet_px::fill_substrate_cache(
                &mut self.paint.wet_substrate,
                paper_active,
                &paper_tex,
                paper_img.as_ref(),
                paper_rot,
                (x0, y0, bw, bh),
                fw,
                noise_tile,
            );
        }
        let substrate = &self.paint.wet_substrate;
        // Selection + protection gates (final enforcement): the splats already stop the wash
        // from FORMING on gated-out texels, but warp/dissolve sampling can still REACH them —
        // the keep-LERP on the final bytes below is the exact restore semantics of the canvas
        // gates, a hard guarantee independent of reach. Both `None` ⇒ keep ≡ 1, byte-identical.
        let (gate_sel, gate_prot, gate_alock) = self.wet_splat_gates();
        let gate_on = gate_sel.is_some() || gate_prot.is_some();
        // Alpha-lock (§2.10, doc 13 #8): the splats already gated the wash to the frozen α; here we PIN
        // the composite's α to that same frozen base so a warp-reached transparent texel can't take a
        // deposit and the silhouette never creeps outward. Off ⇒ byte-identical (no pin).
        let alock_on = gate_alock.is_some();
        let gsel: Option<&[u8]> = gate_sel.as_deref().map(Vec::as_slice);
        let gprot: Option<&[u8]> = gate_prot.as_deref().map(Vec::as_slice);
        let out = Arc::make_mut(&mut self.canvas_rgba);
        // PARALLEL composite over OUTPUT rows (ADR-0109 exception): each pixel is a pure
        // function of immutable inputs — no cross-pixel reduction, no shared mutable state, no
        // RNG — so disjoint rows over the pool are BYTE-IDENTICAL to the serial loop (IEEE-754
        // per-op determinism); each task writes only its own row.
        out[y0 * fw * 4..y1 * fw * 4]
            .par_chunks_mut(fw * 4)
            .enumerate()
            .for_each(|(by, row)| {
                let gy = y0 + by;
                let ly = (gy - ry0) as f32;
                for bx in 0..bw {
                    let gx = x0 + bx;
                    let lx = (gx - rx0) as f32;
                    let gi = (gy * fw + gx) * 4;
                    // Warp the sample position (organic boundary). Window-local coords for the read-window
                    // fields; global for the full-canvas colour buffer (same displacement + window origin).
                    // Per-stroke style: warp AMPLITUDE by the pixel's owner (read PRE-warp —
                    // the displacement needs the amp first); owner 0 = current brush, old path.
                    let st_warp =
                        style_at(has_style, style_owner, style_table, cur_style, gy * fw + gx).warp;
                    // #18: smooth the Warp amplitude across the owner boundary (else the new stroke's
                    // RaggedEdge re-warps the old wash's junction — a hard artefact). PRE-warp `(lx, ly)`.
                    let st_warp = style_field
                        .as_ref()
                        .map_or(st_warp, |sf| sf.sample_warp(lx, ly, st_warp));
                    let (sx, sy) = if st_warp > 0.0 {
                        let (wx, wy) = warp_offset(gx as f32, gy as f32, noise_tile);
                        (lx + wx * st_warp, ly + wy * st_warp)
                    } else {
                        (lx, ly)
                    };
                    // Screen-space AA (Enio 2026-07-20, "borda dura pixelada"): on every silhouette
                    // transition `aa_alpha` < 1 carries the texel's fractional coverage and the wash's
                    // APPEARANCE is lerped toward the base by it below (see `aa_coverage` for why the
                    // fraction must be final linear alpha — the optical model saturates it away as
                    // density input). Flat interior/paper ⇒ `(single sample, 1.0)` ⇒ byte-identical.
                    // The subsample positions route through the FULL Ragged-Edge warp (evaluated at
                    // sub-texel OUTPUT offsets — `pos(0,0)` is exactly `(sx, sy)`), so a strong warp's
                    // serrated boundary anti-aliases too instead of jumping the band between texels.
                    // "Smooth Edges" off = the single-sample pre-AA hard edge, byte-for-byte — the
                    // two looks coexist as a brush mode (Enio 2026-07-20, smooth is the default).
                    let (cw, aa_alpha) = if smooth_edges {
                        aa_coverage(
                            &cov_src,
                            rw,
                            rh,
                            |ox, oy| {
                                if st_warp > 0.0 {
                                    let (wx, wy) =
                                        warp_offset(gx as f32 + ox, gy as f32 + oy, noise_tile);
                                    (lx + ox + wx * st_warp, ly + oy + wy * st_warp)
                                } else {
                                    (lx + ox, ly + oy)
                                }
                            },
                            SS0,
                            SS1,
                        )
                    } else {
                        (
                            smoothstep(SS0, SS1, sample_bilinear(&cov_src, rw, rh, sx, sy)),
                            1.0,
                        )
                    };
                    // EDGE-2 (backrun): the WATER channel at a SERRATED coord ([`water_at`]) — a
                    // water pool is live paint-surface even where the PIGMENT coverage is zero
                    // (pure water, Dilution 1), so the early-out only fires where BOTH are dry.
                    let (water, wxg, wyg) = if watered {
                        water_at(water_buf, fw, fh, gx, gy, noise_tile)
                    } else {
                        (0.0, 0.0, 0.0)
                    };
                    if cw <= 0.0 && water <= 0.0 {
                        // Outside the wash: restore the frozen base (peels any previous frame's composite).
                        row[gx * 4] = base[gi];
                        row[gx * 4 + 1] = base[gi + 1];
                        row[gx * 4 + 2] = base[gi + 2];
                        row[gx * 4 + 3] = base[gi + 3];
                        continue;
                    }
                    // Warped canvas-space indices — shared by the tip-density / pigment-reserve /
                    // style-owner reads (nearest, like the colour buffer).
                    let wgx = (rx0 as f32 + sx).clamp(0.0, (fw - 1) as f32) as usize;
                    let wgy = (ry0 as f32 + sy).clamp(0.0, (fh - 1) as f32) as usize;
                    let widx = wgy * fw + wgx;
                    let st = style_at(has_style, style_owner, style_table, cur_style, widx);
                    // #18: fill/depth/edge/opacity SMOOTHED across the owner boundary (else they step = a
                    // hard junction); geometry/colour stay DISCRETE (Bug #8 lição #4). `None` ⇒ discrete.
                    let (st_fill, st_depth, st_edge_gain, st_opacity) = style_field
                        .as_ref()
                        .map_or((st.fill, st.depth, st.edge_gain, st.opacity), |sf| {
                            sf.sample(sx, sy, (st.fill, st.depth, st.edge_gain, st.opacity))
                        });
                    // A pixel owned by a COMMITTED stroke renders SETTLED (its bake's dry state):
                    // the live flag re-rendered baked washes back to the wet settle (rectangle).
                    let owner_px = if has_style { style_owner[widx] } else { 0 };
                    let settled = commit || (owner_px != 0 && owner_px != cur_o);
                    let inner =
                        sample_bilinear(blur_of(&blurs, st.core_r as usize), rw, rh, sx, sy)
                            .min(1.0);
                    // Per-pixel dwell + deposited alpha — read here for the rim modulation
                    // (EDGE-4) and reused by the granulation settle / pigment blocks below.
                    let soak_v = if soaked {
                        f32::from(soak_buf[gy * fw + gx]) / 255.0
                    } else {
                        0.0
                    };
                    let ci = (wgy * fw + wgx) * 4;
                    let col_a = if has_color {
                        f32::from(color_buf[ci + 3]) / 255.0
                    } else {
                        1.0
                    };
                    // EDGE-4: the rim tells the GESTURE's story — stronger where the water pooled
                    // or lingered (soak), fainter where the brush barely deposited (a depleted
                    // trail's dry tail). Zero reads beyond fields that already exist.
                    let gain_px =
                        st_edge_gain * (1.0 + EDGE_SOAK_BOOST * soak_v) * (0.5 + 0.5 * col_a);
                    // EDGE-3 (Curtis §4.3.3, conservação): rim = unsharp ASSINADO — o lobo
                    // negativo (franja, `inner > cw`) EMPALIDECE (o pigmento da borda MIGROU do
                    // interior; era clampado ≥0 = wash uniforme com contorno). Doc 12 §W-C.
                    let mut edge = (gain_px * (cw - inner)).min(1.0);
                    // Paper tooth + Granulation source + paper-depth component — resolved per OWNER
                    // (#13): the cached value when single-substrate (byte-identical), else recomputed
                    // from the owner's paper. See [`SubstrateSession::at`].
                    let cached_ph = use_substrate_cache.then(|| substrate[gy * fw + gx]);
                    let (paper_h, gran_h, paper_component) =
                        substrate_session.at(&st, owner_px, cached_ph, gx, gy);
                    // Take 10: TODO termo wet-driven lê o campo borrado ([`sample_wet_field`]) —
                    // o thinning do interior era o degrau de ~11 bytes/px na fronteira de dono.
                    let st_wet_px =
                        sample_wet_field(wet_field.as_ref(), (rw, rh), (sx, sy), st.wet);
                    // GRAN-1 (Curtis §4.5, Tier-2): valley deposition + the take-3 drying model —
                    // extracted to [`granulation_factor`] (LOC cap); the amount + water follow the
                    // pixel's OWNER stroke (per-stroke style).
                    let gran = granulation_factor(
                        gran_h,
                        paper_component,
                        st.granulation,
                        st_wet_px,
                        soak_v,
                        settled,
                    );
                    // Wet also wets the WASH ITSELF (blank canvas included): more water = the wash's own
                    // pigment redistributes — the interior thins toward the receding front, the edge pools
                    // harder, and the pooling follows the paper tooth (a wetter bloom is ragged, not a
                    // clean ring). This is what makes the Spread read intense + organic under Wet even
                    // before any old paint is involved (Enio 2026-07-06 "mais intenso e menos uniforme").
                    let mut fill_px = st_fill;
                    if st_wet_px > 0.0 {
                        fill_px = st_fill
                            * (1.0 - (WET_THIN * st_wet_px * st.spread_thin * inner).min(0.95));
                        let ragged =
                            (1.0 + (paper_h - 0.5) * 2.0 * WET_RAGGED * st_wet_px).max(0.0);
                        edge = (edge * (1.0 + WET_EDGE_BOOST * st_wet_px) * ragged).min(1.5); // LITERAL-PX-OK: wet edge may overshoot the dry clamp; signed (EDGE-3) keeps the pale lobe
                    }
                    // Tip density at the warped position (nearest, like the colour buffer).
                    let tip_dens = if has_dens {
                        f32::from(dens_buf[wgy * fw + wgx]) / 255.0
                    } else {
                        1.0
                    };
                    let mut density = ((cw * fill_px * tip_dens + edge) * gran).max(0.0);
                    // MIX-1: the brush's local pigment reserve (fresh + carry) fades fill AND edge
                    // together over the intact water footprint — the depleted tail dries toward
                    // plain water. Applied BEFORE the rewet-pool term: pigment dissolved off the
                    // CANVAS is not the brush's reserve and must not fade with it.
                    if has_depl {
                        density *= f32::from(depl_buf[wgy * fw + wgx]) / 255.0;
                    }
                    // Wet-on-wet lift / dissolve / pool / backrun ring — [`rewet_px`] (sibling,
                    // LOC split), verbatim math. `pool` is the density ADDITION from bloom + ring.
                    // O Rewet dos termos é o CAMPO borrado do wet-do-dono (take 10, `st_wet_px`).
                    let rw_px = match &rewet {
                        Some(f) => rewet_px(
                            f,
                            (rx0, ry0),
                            (sx, sy),
                            (wxg, wyg),
                            water,
                            st_wet_px,
                            cw,
                            inner,
                        ),
                        None => Default::default(),
                    };
                    let (lift, dissolve, backrun, bleed, wet_paint) = (
                        rw_px.lift,
                        rw_px.dissolve,
                        rw_px.backrun,
                        rw_px.bleed,
                        rw_px.wet_paint,
                    );
                    density += rw_px.pool;
                    let od = density * st_depth;

                    // Pigment colour: depositado vs brush por lerp PROPORCIONAL (`ca8/255`, a
                    // fração de pigmento do depósito). A janela fixa `COL_LO..COL_HI` (take 6)
                    // cruzava em ~1px na borda do footprint = a LINHA DURA da junção (take 10,
                    // sonda [maps]); depositado == raw (mixer-off) ⇒ byte-idêntico.
                    let mut pig = st.color;
                    if has_color {
                        let ca8 = color_buf[ci + 3];
                        if ca8 == u8::MAX {
                            pig = [color_buf[ci], color_buf[ci + 1], color_buf[ci + 2]];
                        } else if ca8 > 0 {
                            let w = f32::from(ca8) / 255.0;
                            for c in 0..3 {
                                pig[c] = (f32::from(st.color[c])
                                    + (f32::from(color_buf[ci + c]) - f32::from(st.color[c])) * w
                                    + 0.5) as u8;
                            }
                        }
                    }
                    // Wet-on-wet DISSOLVE: the lifted paint's colour (diffused through the wet region)
                    // tints the wash's pigment — the old colour bleeds into and beyond its own footprint.
                    // SUBTRACTIVE mix (absorbance-space geometric mean, via the ln/exp LUTs): paints mix
                    // like pigments, not light — the linear sRGB lerp desaturated the blend toward the
                    // paper's cream ("pálida e amarelada sem Pigment", Enio 2026-07-06). Pigment ON still
                    // adds its full RYB pass on top, unchanged.
                    if dissolve > 0.0 {
                        for c in 0..3 {
                            let a = -lut.lnl[pig[c] as usize];
                            let bi = (bleed[c] + 0.5).clamp(0.0, 255.0) as usize;
                            let b = -lut.lnl[bi];
                            let mag = a + (b - a) * dissolve;
                            pig[c] = lut.l2s_byte(lut.exp_mag(mag));
                        }
                    }
                    // Backrun CONCENTRATION (EDGE-2): the ring's pigment is the pushed paint
                    // CONCENTRATED — Beer–Lambert saturates at the pigment colour, so density
                    // alone can never render darker than the wash the pigment came from; the
                    // "severely darkened edge" needs a darker floor (absorbance × ring).
                    if backrun > 0.0 {
                        for p in &mut pig {
                            let a = -lut.lnl[*p as usize] * (1.0 + BACKRUN_CONC * backrun);
                            *p = lut.l2s_byte(lut.exp_mag(a));
                        }
                    }

                    // Effective base in linear light: the layer's own pixels composited over the REAL
                    // ground (the backdrop under the active layer — so a transparent layer attenuates
                    // what is actually beneath it, not a virtual cream; an opaque base uses only itself).
                    let ab = f32::from(base[gi + 3]) / 255.0;
                    let ground_lin = [
                        lut.s2l[ground[gi] as usize],
                        lut.s2l[ground[gi + 1] as usize],
                        lut.s2l[ground[gi + 2] as usize],
                    ];
                    let mut sb = [
                        lut.s2l[base[gi] as usize] * ab + ground_lin[0] * (1.0 - ab),
                        lut.s2l[base[gi + 1] as usize] * ab + ground_lin[1] * (1.0 - ab),
                        lut.s2l[base[gi + 2] as usize] * ab + ground_lin[2] * (1.0 - ab),
                    ];
                    // Wet-on-wet LIFT ([`apply_wet_lift`]): rewetting walks the base's pigment toward the
                    // LOCAL ground in log space (`lift` already moisture-scaled — a dried spot won't lift).
                    apply_wet_lift(&mut sb, &ground_lin, lift, lut);
                    // BODY / OPACITY (doc 13 #17): pure Beer–Lambert only subtracts, so a light-valued
                    // pigment barely deposits ("azul e amarelo quase não aparecem"). Body lays the pigment's
                    // OWN colour over the transmittance result ([`watercolor_lut::Luts::body_cov`]; `0` ⇒ no-op).
                    let body_cov = lut.body_cov(st_opacity, od);
                    let mut rgb = [0u8; 3];
                    let mut t_lum = 0.0f32;
                    let mut t_min = 1.0f32;
                    // Extra alpha the BODY-shifted appearance needs to stay in gamut (0 when body off, so
                    // `cov_a` is byte-identical). Body lays the pigment's own colour, which can push a
                    // channel PAST the `1 − t_min` floor the un-premultiply assumes — over white the most-
                    // absorbed channel drops below `ground·(1−a)` and `L` clamps (a 22-byte flatten error).
                    let mut a_body = 0.0f32;
                    const LUM: [f32; 3] = [0.2126, 0.7152, 0.0722];
                    for c in 0..3 {
                        let t = lut.transmittance(pig[c], od);
                        let optical = sb[c] * t + lut.s2l[pig[c] as usize] * (1.0 - t);
                        // Hide the substrate under the pigment's own colour by `body_cov` (0 ⇒ optical).
                        let lin = optical + (lut.s2l[pig[c] as usize] - optical) * body_cov;
                        rgb[c] = lut.l2s_byte(lin);
                        if body_cov > 0.0 {
                            // Keep the un-premultiply in gamut: body can push a channel past the `1 − t_min`
                            // floor. Uses the quantised `app` the un-premultiply reads ([`gamut_alpha`]).
                            a_body =
                                a_body.max(gamut_alpha(lut.s2l[rgb[c] as usize], ground_lin[c]));
                        }
                        t_lum += LUM[c] * t;
                        t_min = t_min.min(t);
                    }
                    // Perceptual film opacity — how much pigment sits here; drives the deposited alpha and
                    // the subtractive (RYB) paint-mix amount over the base paint.
                    let film_a = (1.0 - t_lum).clamp(0.0, 1.0);
                    // The subtractive wet-on-wet MIX — the wash's pigment blending with the paint beneath
                    // like real paint. It is "o segredo" of the good wet-on-wet look (Enio 2026-07-06), so
                    // **Wet drives it too**: `max(Pigment's Mix, wet × paint-presence)` — a wet wash mixes
                    // with what it rewets even with Pigment unchecked (inert on blank canvas, like the
                    // lift); the checkbox + Mix slider still set the floor and are the only source when
                    // Wet is 0 (byte-identical default preserved). The blend reads the LIFTED base (`sb`)
                    // where the rewet lightened it — mixing against the raw base would paint the lift
                    // right back over.
                    // Water does NOT drive the RYB paint-mix: a water pool has no pigment of its
                    // own to blend (its tint is the dissolve), and `water = 1` pushed the mix to
                    // full-replace — territory where `ryb_mix` degenerates (OPT-2's documented
                    // defect; the smoke's navy-blue ring). Wet keeps driving it as before.
                    let mix_amt = st.pigment_mix.max(st_wet_px * wet_paint);
                    if mix_amt > 0.0 {
                        // The (possibly lifted) base APPEARANCE over the ground — for an opaque base with
                        // no lift this is the raw base bytes exactly (`l2s(s2l(b)) == b`); for a
                        // transparent one it is the ground showing through (the old raw-bytes read was
                        // black there).
                        let mix_base = [
                            f32::from(lut.l2s_byte(sb[0])) / 255.0,
                            f32::from(lut.l2s_byte(sb[1])) / 255.0,
                            f32::from(lut.l2s_byte(sb[2])) / 255.0,
                        ];
                        let mixed = ryb_mix(
                            mix_base,
                            [
                                f32::from(pig[0]) / 255.0,
                                f32::from(pig[1]) / 255.0,
                                f32::from(pig[2]) / 255.0,
                            ],
                            film_a,
                        );
                        for c in 0..3 {
                            let sub = (mixed[c].clamp(0.0, 1.0) * 255.0 + 0.5).clamp(0.0, 255.0);
                            rgb[c] =
                                (f32::from(rgb[c]) + (sub - f32::from(rgb[c])) * mix_amt) as u8;
                        }
                    }
                    // Screen-space AA alpha (`aa_coverage` — 1.0 everywhere but a thin stroke's steep
                    // rim): composite the wash's target APPEARANCE over the base-over-ground appearance
                    // by the texel's fractional silhouette coverage, in linear light and BEFORE the
                    // un-premultiply — a byte-level lerp on the stored straight-alpha pixels broke the
                    // flatten equality on a transparent layer (the stored L is not an appearance).
                    // Linear on the appearance, so the fringe/Beer–Lambert saturation can't eat it.
                    if aa_alpha < 1.0 {
                        for c in 0..3 {
                            let app = lut.s2l[rgb[c] as usize];
                            let base_app =
                                lut.s2l[base[gi + c] as usize] * ab + ground_lin[c] * (1.0 - ab);
                            rgb[c] = lut.l2s_byte(app * aa_alpha + base_app * (1.0 - aa_alpha));
                        }
                    }
                    // Coverage alpha = the STRONGEST per-channel absorption (`1 − min_c T_c`), not the
                    // luminance film: the un-premultiply below needs `a ≥ 1 − T_c` on EVERY channel or
                    // the solve leaves gamut and clamps (a red wash's G/B absorb far more than the
                    // luminance says — measured 59-byte flatten error with the luminance alpha).
                    // `film_a` stays the perceptual meter for the paint-mix strength above. Body backs the
                    // hidden substrate with the exact alpha its appearance needs (`a_body`, 0 when body
                    // off) so a light opaque pigment un-premultiplies to its own colour, not a clamped
                    // ghost; `a_body = 0` ⇒ `cov_a = 1 − t_min` unchanged (byte-identical).
                    // The AA fraction also scales the wash's ADDED alpha (`aa_alpha` = 1.0 off the rim),
                    // so a transparent layer's silhouette fades with its fractional coverage too.
                    let cov_a = ((1.0 - t_min).max(a_body) * aa_alpha).clamp(0.0, 1.0);
                    let out_a = (ab + (1.0 - ab) * cov_a).clamp(0.0, 1.0);
                    // `rgb` is the target APPEARANCE over the ground. The layer stores straight RGBA
                    // that the compositor will blend over that same ground — so solve the un-premultiply
                    // `L = (appearance − ground·(1−a)) / a` in linear light. Baking the appearance
                    // directly (the old path) baked the ground INTO the pixels: over a white backdrop
                    // the wash carried a permanent cream cast ("puxa para o bege", Enio 2026-07-06).
                    // Opaque base ⇒ a = 1 ⇒ L = appearance, byte-identical to the old path.
                    if out_a <= f32::EPSILON {
                        // No film and no base: the layer stays untouched (appearance == ground).
                        row[gx * 4] = base[gi];
                        row[gx * 4 + 1] = base[gi + 1];
                        row[gx * 4 + 2] = base[gi + 2];
                        row[gx * 4 + 3] = base[gi + 3];
                        continue;
                    }
                    let inv_a = 1.0 / out_a;
                    let mut px = [0u8; 4];
                    for c in 0..3 {
                        let app = lut.s2l[rgb[c] as usize];
                        let lin = (app - ground_lin[c] * (1.0 - out_a)) * inv_a;
                        px[c] = lut.l2s_byte(lin);
                    }
                    px[3] = (out_a * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
                    // Paint gates (selection / protection): keep-lerp the painted bytes toward the
                    // frozen base — the canvas gates' exact restore semantics, warp/diffusion-proof
                    // (see the gate hoist above the loop). Ungated (default) writes paint verbatim.
                    if gate_on {
                        let keep = watercolor_accum::splat_keep(gsel, gprot, None, gy * fw + gx);
                        if keep < 1.0 {
                            for (c, p) in px.iter_mut().enumerate() {
                                let painted = f32::from(*p);
                                let orig = f32::from(base[gi + c]);
                                *p = (painted * keep + orig * (1.0 - keep))
                                    .round()
                                    .clamp(0.0, 255.0) as u8;
                            }
                        }
                    }
                    // Alpha-lock: freeze the layer's α to the frozen base — colour deposits into the
                    // existing paint (the splats already scaled coverage by α), the α itself never moves.
                    if alock_on {
                        px[3] = base[gi + 3];
                    }
                    row[gx * 4] = px[0];
                    row[gx * 4 + 1] = px[1];
                    row[gx * 4 + 2] = px[2];
                    row[gx * 4 + 3] = px[3];
                }
            });
        self.mark_dirty(region);
        if commit {
            self.paint.watercolor_base = None;
        }
        Some(region)
    }
}
