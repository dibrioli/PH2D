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
//!   inner = min(inner, P(régua, core_r))                   // O ARO VIRA A QUINA (`watercolor_rim`)
//!   edge  = clamp(cover·(1 − inner)·edge_gain, 0, 1)       // pigment pooled at the receding front
//!   gran  = 1 + (paperHeight − 0.5)·2·granulation          // paper-tooth granulation (value noise)
//!   D     = (cover·fill + edge)·gran                        // optical density
//!   Tᵢ    = pigmentᵢ^(D·depth)                              // Beer–Lambert transmittance per channel
//!   outᵢ  = l2s( s2l(Bᵢ)·Tᵢ + s2l(pigmentᵢ)·(1 − Tᵢ) )     // base attenuated + pigment scattered
//! ```
//! All per-pixel math is table lookups + sums/mults (HR-5) — LUTs in [`super::watercolor_lut`], the
//! integer-hash value noise in [`super::watercolor_field`]; no transcendental runs in the hot loop.

mod aparencia; // a óptica de um texel como função do chão (LOC split; o vidro a avalia 2×)
mod diag; // o envelope de diagnóstico do composite (LOC split)
mod pigment; // a COR do pigmento por pixel (LOC split, por assunto)
mod window;
pub(in crate::tool::paint) use window::Alcance;
pub(in crate::tool::paint) use window::ponto_na_janela;

use super::watercolor_field::*;
use super::watercolor_rewet_px::{
    apply_wet_lift, blur_of, build_style_field, build_wet_field, params_differ, rewet_px,
    sample_wet_field, style_at,
};
use super::*;
use rayon::prelude::*;

/// Hardened-coverage smoothstep edges (wet_edges `SS0`/`SS1`): below `SS0` the wash is transparent,
/// above `SS1` fully covered — a crisp-but-soft silhouette from the feathered coverage discs.
pub(super) const SS0: f32 = 0.12; // LITERAL-PX-OK: coverage-hardening smoothstep low edge (wet_edges)
pub(super) const SS1: f32 = 0.60; // LITERAL-PX-OK: coverage-hardening smoothstep high edge (wet_edges)

impl PainterTool {
    /// Um composite sobre a janela do `wet_frame_dirty`; cada linha de saída caminha só o vão das
    /// `linhas` que mudaram ([`window::vao_da_linha`]; `None` = a janela inteira).
    pub(super) fn apply_watercolor_inner(
        &mut self,
        commit: bool,
        linhas: Option<&[(u32, u32, u32)]>,
    ) -> Option<Region> {
        // Window arithmetic (frozen bases + dirty-rect consumption + influence
        // padding + read window) — moved verbatim to [`window::WashWindow`];
        // `None` = nothing to composite, and the commit still drops the base.
        let Some(w) = self.wash_window(commit) else {
            if commit {
                self.paint.watercolor_base = None;
            }
            self.paint.wet_reserve_cache = None; // um sujo consumido sem composite: não confiar
            return None;
        };
        // Counted AFTER the window resolves, so it counts composites that did WORK — a call that
        // bailed on an empty dirty rect is not a composite, and counting it would let the cadence gate
        // pass on a route that never painted anything.
        self.wash.composites = self.wash.composites.saturating_add(1);
        // ...e a ÁREA que ele vai caminhar, pelo mesmo motivo: a contagem é o que ainda diz a verdade
        // com a máquina disputada, e é ela que separa *o quadro caminhou mais* de *o box estava cheio*.
        self.wash.window_px = self
            .wash
            .window_px
            .saturating_add((w.rw as u64) * (w.rh as u64));
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
            pad,
            changed,
            rx0,
            ry0,
            rx1,
            ry1,
            rw,
            rh,
        } = w;

        let cov_src =
            window::cobertura_da_janela(&self.paint.stroke_coverage, fw, (rx0, ry0, rw, rh));
        // OS CAMPOS DO ARO ([`watercolor_rim::rim_fields`]): `hard`, os borrões e a régua do teto
        // nascem juntos e só o aro os lê, então a receita mora ao lado dele.
        let rim = watercolor_rim::rim_fields(
            &cov_src,
            rw,
            rh,
            core_r,
            &self.paint.wet_styles.table,
            &self.paint.brush,
        );

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

        let wash_flow = watercolor_field::wash_flow(brush);
        let fill = brush.fill.clamp(0.0, 1.0) * wash_flow;
        let depth = brush.depth.max(0.0);
        let edge_gain = brush.edge_gain.max(0.0) * wash_flow;
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
        let has_color = self.paint.stroke_color.len() == n * 4;
        // Manual textured tip (doc 13 #1 round 3): the per-stroke tip-density buffer scales the
        // interior fill (`cw·fill·dens`) — the tip's texture reads as pigment variation INSIDE a
        // normally-wet wash (water fills the tip's silhouette; texture modulates the deposit).
        // Empty (every non-textured path) ⇒ density ≡ 1 → byte-identical.
        let has_dens = self.paint.stroke_density.len() == n;
        let dens_buf = &self.paint.stroke_density;
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
            color: watercolor_field::cor_em_bytes(brush.color), // o pigmento quando o buffer é fraco
            spread_thin,
            core_r: core_r as u16,
            spread_px: spread as u16,
            reserve_r: watercolor_reserve::reserve_radius(brush.radius_px, core_r, spread, wet),
            paper: paper_tex,
            paper_depth: brush
                .paper_depth
                .clamp(0.0, ph2d_painter_brush::PAPER_TOOTH_MAX),
            granulation_use_paper: brush.granulation_use_paper,
            texture: gran_tex,
            edge_flow: brush.edge_flow,
            paper_edge: brush.paper_edge.clamp(0.0, 1.0),
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

        // Wet Mix pigment reserve (MIX-1, [`watercolor_reserve`]): scales the BRUSH density term
        // AFTER the rim derives from the intact coverage. Mixer never on ⇒ `None` ⇒ byte-identical.
        let rc = self.paint.wet_reserve_cache.take();
        let reserve = self.reserve_fields((fw, n, (rx0, ry0), (rw, rh)), &cur_style, changed, rc);
        let color_buf = &self.paint.stroke_color;
        // O memo do `paper_h` por texel do canvas — [`watercolor_rewet_px::memo_do_substrato`].
        let use_substrate_cache = watercolor_rewet_px::memo_do_substrato(
            &mut self.paint.wet_substrate,
            n,
            substrate_session.multi(),
            (paper_active, &paper_tex, paper_img.as_ref(), paper_rot),
            (x0, y0, bw, bh),
            fw,
            noise_tile,
        );
        let substrate = &self.paint.wet_substrate;
        // A FORMA DA BORDA (BUGS #31): o fluxo do Ragged Edge e o papel na borda — [`watercolor_flow`].
        let (flow, paper_edge_any) = watercolor_flow::EdgeFlow::build(
            &cur_style,
            style_table,
            has_style.then_some(&style_owner[..]),
            self.imagens_da_borda(),
            fw,
            (x0, y0, bw, bh),
            (rx0, ry0, rw, rh),
            noise_tile,
        );
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
        let out = crate::tool::paint::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            self.source_size.0,
            None,
        );
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
                let (bx0, bx1) = window::vao_da_linha(linhas, gy, pad, (x0, bw));
                for bx in bx0..bx1 {
                    let gx = x0 + bx;
                    let lx = (gx - rx0) as f32;
                    let gi = (gy * fw + gx) * 4;
                    // Warp the sample position (organic boundary). Window-local coords for the read-window
                    // fields; global for the full-canvas colour buffer (same displacement + window origin).
                    // Per-stroke style: warp AMPLITUDE by the pixel's owner (read PRE-warp —
                    // the displacement needs the amp first); owner 0 = current brush, old path.
                    let o_pre = if has_style {
                        style_owner[gy * fw + gx]
                    } else {
                        0
                    };
                    let st_warp =
                        style_at(has_style, style_owner, style_table, cur_style, gy * fw + gx).warp;
                    // #18: smooth the Warp amplitude across the owner boundary (else the new stroke's
                    // RaggedEdge re-warps the old wash's junction — a hard artefact). PRE-warp `(lx, ly)`.
                    let st_warp = style_field
                        .as_ref()
                        .map_or(st_warp, |sf| sf.sample_warp(lx, ly, st_warp));
                    // O ponto deslocado, pela porta [`ponto_na_janela`] (o centro e as subamostras do AA).
                    let (sx, sy) = if st_warp > 0.0 || paper_edge_any {
                        let (dx, dy) = flow.desloca(o_pre, lx, ly, gx as f32, gy as f32, st_warp);
                        (
                            ponto_na_janela(gx, 0.0, dx, rx0),
                            ponto_na_janela(gy, 0.0, dy, ry0),
                        )
                    } else {
                        (lx, ly)
                    };
                    // Screen-space AA (Enio 2026-07-20, "borda dura pixelada"): on every silhouette
                    // transition `aa_alpha` < 1 carries the texel's fractional coverage, and the wash's
                    // APPEARANCE lerps toward the base by it (`aa_coverage`: final linear alpha). Flat
                    // interior/paper ⇒ `(single sample, 1.0)`. The subsamples route through the FULL
                    // Ragged-Edge warp at sub-texel OUTPUT offsets (`pos(0,0)` is `(sx, sy)`), so a
                    // serrated boundary anti-aliases too. "Smooth Edges" off = the pre-AA hard edge.
                    let (cw, aa_alpha) = if smooth_edges {
                        aa_coverage(
                            &cov_src,
                            rw,
                            rh,
                            |ox, oy| {
                                // O centro JÁ é `(sx, sy)` — a mesma conta com `ox = oy = 0`.
                                if ox == 0.0 && oy == 0.0 {
                                    (sx, sy)
                                } else if st_warp > 0.0 || paper_edge_any {
                                    let (x, y) = (gx as f32 + ox, gy as f32 + oy);
                                    let (dx, dy) = flow.desloca(o_pre, lx, ly, x, y, st_warp);
                                    (
                                        ponto_na_janela(gx, ox, dx, rx0),
                                        ponto_na_janela(gy, oy, dy, ry0),
                                    )
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
                        sample_bilinear(blur_of(&rim.blurs, st.core_r as usize), rw, rh, sx, sy)
                            .min(1.0);
                    // O TETO do flanco reto (doc 36). É um `min`, então ele só pode DAR aro, nunca
                    // tirar — e fora / sem borrão a porta devolve `1.0`, logo o `min` é inerte e a
                    // expressão fica incondicional. Ausente (`None`) ⇒ nada a limitar.
                    let inner = rim.sd.as_ref().map_or(inner, |sd| {
                        let d = sample_bilinear(sd, rw, rh, sx, sy);
                        inner.min(watercolor_rim::straight_edge_cap(d, st.core_r as usize))
                    });
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
                            (1.0 + (0.5 - paper_h) * 2.0 * WET_RAGGED * st_wet_px).max(0.0);
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
                    if let Some(rf) = reserve.as_ref() {
                        density *= rf.sample(st.reserve_r, sx, sy);
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

                    // A COR do pigmento (dono suavizado → depósito → água) vive no irmão
                    // [`pigment`]: aqui o laço responde QUANTA densidade há, ali DE QUE COR ela é.
                    let pig = pigment::pigmento_do_pixel(
                        style_field
                            .as_ref()
                            .map_or(st.color, |sf| sf.sample_color(sx, sy, st.color)),
                        has_color.then_some((color_buf.as_slice(), ci)),
                        dissolve,
                        bleed,
                        backrun,
                        lut,
                    );

                    // Effective base: the layer over the REAL ground (a transparent layer attenuates what
                    // is beneath it), joined in TONES OF THE SCREEN like the compositor (ADR-0177); the
                    // optics run on it in linear light ([`aparencia`]).
                    let ab = f32::from(base[gi + 3]) / 255.0;
                    let chao = [ground[gi], ground[gi + 1], ground[gi + 2]];
                    let base_sobre: [f32; 3] = core::array::from_fn(|c| {
                        (f32::from(base[gi + c]) * ab + f32::from(chao[c]) * (1.0 - ab)) / 255.0
                    });
                    // ⚠️ O `Pigment` só mistura com TINTA: sobre papel a presença é 0 e o botão
                    // ligado não desbota a lavagem ([`super::watercolor_mistura`], medido 2026-09-24).
                    let tinta = if st.pigment_mix > 0.0 {
                        super::watercolor_mistura::presenca_de_tinta(base, ground, gi).0
                    } else {
                        0.0
                    };
                    let optica = aparencia::Optica {
                        pig,
                        od,
                        body_cov: lut.body_cov(st_opacity, od),
                        lift,
                        pelo_botao: st.pigment_mix * tinta,
                        pela_agua: st_wet_px * wet_paint,
                        aa_alpha,
                    };
                    let aparencia::Aparencia { rgb, t_min, a_body } =
                        aparencia::aparencia(&optica, base_sobre, chao, lut);
                    let ground_enc = chao.map(|g| f32::from(g) / 255.0);
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
                    // `rgb` is the target APPEARANCE over the ground. The layer stores straight RGBA that
                    // the compositor blends over that same ground — so solve `L = (appearance −
                    // ground·(1−a)) / a` in tones of the screen (ADR-0177). Baking the appearance directly
                    // (the old path) baked the ground INTO the pixels: over a white backdrop the wash
                    // carried a permanent cream cast ("puxa para o bege", Enio 2026-07-06). Opaque base
                    // ⇒ a = 1 ⇒ L = appearance.
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
                        let app = f32::from(rgb[c]) / 255.0;
                        let l = (app - ground_enc[c] * (1.0 - out_a)) * inv_a;
                        px[c] = (l.clamp(0.0, 1.0) * 255.0).round() as u8;
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
        self.paint.wet_reserve_cache = reserve.and_then(watercolor_reserve::ReserveFields::devolve);
        self.mark_dirty(region);
        if commit {
            self.paint.watercolor_base = None;
        }
        Some(region)
    }
}
