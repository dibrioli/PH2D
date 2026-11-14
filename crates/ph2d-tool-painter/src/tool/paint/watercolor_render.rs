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
#[cfg(test)]
pub(super) use aparencia::APARENCIA_PELA_CONTA;
mod diag; // o envelope de diagnóstico do composite (LOC split)
mod dono; // o dono do halo: o texel sem dono pelo dono mais próximo no alcance (B16.2)
mod pigment; // a COR do pigmento por pixel (LOC split, por assunto)
mod texel; // o corpo do laço por texel (B52.R21, LOC split)
mod window;
pub(in crate::tool::paint) use window::Alcance;
#[cfg(test)]
pub(in crate::tool::paint) use window::COMMIT_PELA_UNIAO;
pub(in crate::tool::paint) use window::ponto_na_janela;

use super::watercolor_field::*;
use super::watercolor_rewet_px::{build_style_field, build_wet_field, params_differ};
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
        // B16.7: um quadro sem sujo (Move abaixo do espaçamento) não consome nada ⇒ os campos guardados
        // continuam certos; só um sujo consumido sem composite os larga.
        let consome = commit || self.wash.recompoe_tudo || self.paint.wet_frame_dirty.is_some();
        let Some(w) = self.wash_window(commit) else {
            if commit {
                self.paint.watercolor_base = None;
            }
            if consome {
                self.paint.wet_reserve_cache = None; // um sujo consumido sem composite: não confiar
                self.paint.wet_rewet_cache = None; // B16.5: idem
            }
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

        // O VIDRO (doc 48): o plano da camada sai do mapa ANTES dos empréstimos do laço.
        let vidro = self.vidro_do_quadro(&base_arc, &backdrop_arc, Some(region));
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
        // B16.5: os campos lêem-se da grelha da tela guardada entre quadros (só o sujo ⊕ R refaz).
        let rewet_cache = self.paint.wet_rewet_cache.take();
        #[cfg(test)]
        let rewet_cache = rewet_cache.filter(|_| !REWET_SEM_CACHE.get());
        let rewet = (wet_any > 0.0 || watered).then(|| {
            #[cfg(test)]
            if REWET_SEM_CACHE.get() {
                return build_rewet_fields(
                    &base_arc[..],
                    ground,
                    &self.paint.wet_soak,
                    soaked,
                    &self.paint.stroke_water,
                    watered,
                    (fw, fh),
                    (rx0, ry0, rx1, ry1),
                    spread_any,
                );
            }
            let e = RewetEntradas {
                base: &base_arc[..],
                ground,
                wet_soak: &self.paint.wet_soak,
                soaked,
                stroke_water: &self.paint.stroke_water,
                watered,
                fw,
                fh,
            };
            let (f, c) = build_rewet_fields_cached(
                e,
                (rx0, ry0, rx1, ry1),
                spread_any,
                changed,
                rewet_cache,
            );
            self.paint.wet_rewet_cache = Some(c);
            f
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
        let paper_tex = snap_slot_size(self.papel_na_escala(brush.paper), noise_tile);
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
            deposito_dono: false,
            cor_no_papel: false,
            ponta_texturada: false,
        }
        .com_marcas(self.marcas_da_aguada());
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
        let halo_do_wet = self.paint_media() == super::media::PaintMedia::WetPaint;
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
        window::nota_o_substrato(
            &mut self.paint.wet_substrate_caixa,
            use_substrate_cache,
            region,
            fw,
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
        // O laço só escreve `region` (o `declare_wrote` abaixo): o desfazer guarda os tiles dela, não a tela inteira.
        let caixa_da_escrita = crate::tool::paint::plane_fork::CAPTURA_PELA_CAIXA
            .get()
            .then_some(region);
        let out = crate::tool::paint::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            self.source_size.0,
            caixa_da_escrita,
        );
        // PARALLEL composite over OUTPUT rows (ADR-0109 exception): each pixel is a pure
        // function of immutable inputs — no cross-pixel reduction, no shared mutable state, no
        // RNG — so disjoint rows over the pool are BYTE-IDENTICAL to the serial loop (IEEE-754
        // per-op determinism); each task writes only its own row.
        let (vcamada, mut vplano, vbase, vchao) = match vidro {
            Some(v) => (Some(v.camada), Some(v.plano), Some(v.base), v.chao_preto),
            None => (None, None, None, None),
        };
        let vbase = vbase.as_ref();
        let vchao: Option<&[u8]> = vchao.as_deref().map(Vec::as_slice);
        // B16.2 cont3 (o dono mais próximo do halo) só no Wet Paint (BUGS_painter #115): fora dele o texel sem dono
        // resolve-se pelo pincel vivo, os bytes do main.
        let halo = if has_style && halo_do_wet {
            dono::DonoDoHalo::build(style_owner, (fw, fh), (rx0, ry0, rw, rh), pad)
        } else {
            dono::DonoDoHalo::vazio()
        };
        let q = texel::Quadro {
            fw,
            fh,
            rx0,
            ry0,
            rw,
            rh,
            commit,
            cur_o,
            cov_src: &cov_src,
            rim: &rim,
            rewet: &rewet,
            lut,
            base,
            ground,
            smooth_edges,
            noise_tile,
            has_color,
            color_buf,
            has_dens,
            dens_buf,
            style_table,
            style_owner,
            halo: &halo,
            has_style,
            cur_style,
            substrate_session: &substrate_session,
            soaked,
            soak_buf,
            watered,
            water_buf,
            wet_field: &wet_field,
            style_field: &style_field,
            reserve: &reserve,
            use_substrate_cache,
            substrate,
            flow: &flow,
            paper_edge_any,
            gate_on,
            gsel,
            gprot,
            alock_on,
            vbase,
            vchao,
        };
        let vlinhas: Vec<Option<&mut [crate::compositor::vidro::Vidro]>> = match vplano.as_mut() {
            Some(p) => Arc::get_mut(p).expect("o plano é só deste quadro")[y0 * fw..y1 * fw]
                .chunks_mut(fw)
                .map(Some)
                .collect(),
            None => (y0..y1).map(|_| None).collect(),
        };
        out[y0 * fw * 4..y1 * fw * 4]
            .par_chunks_mut(fw * 4)
            .zip(vlinhas.into_par_iter())
            .enumerate()
            .for_each(|(by, (row, mut vrow))| {
                let gy = y0 + by;
                let (bx0, bx1) = window::vao_da_linha(linhas, gy, pad, (x0, bw));
                for bx in bx0..bx1 {
                    q.texel(x0 + bx, gy, row, vrow.as_deref_mut());
                }
            });
        if let (Some(c), Some(p)) = (vcamada, vplano) {
            self.devolve_o_vidro(c, p);
        }
        self.paint.wet_reserve_cache = reserve.and_then(watercolor_reserve::ReserveFields::devolve);
        self.declare_wrote(Some(region)); // o laço só escreve `y0..y1 × x0..x0+bw` (B16.1)
        self.mark_dirty(region);
        if commit {
            self.paint.watercolor_base = None;
        }
        Some(region)
    }
}
