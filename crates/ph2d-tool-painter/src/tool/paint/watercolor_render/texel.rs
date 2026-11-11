//! **UM TEXEL DA AGUADA** — o corpo do laço do composite ([`super`]), cortado pelo tecto de LOC
//! (B52.R21): [`Quadro`] guarda as entradas imutáveis do quadro e [`Quadro::texel`] escreve um texel
//! de saída (e o seu vidro). A aritmética é a de antes, verbatim — o mesmo bit.

use super::super::watercolor_field::*;
use super::super::watercolor_rewet_px::{
    StyleField, SubstrateSession, blur_of, rewet_px, sample_wet_field, style_at,
};
use super::super::*;
use super::{SS0, SS1, aparencia, pigment, ponto_na_janela};
use crate::compositor::vidro::Vidro;

/// As entradas de um composite que o laço por texel LÊ (todas imutáveis durante o laço).
#[derive(Clone, Copy)]
pub(super) struct Quadro<'a> {
    pub(super) fw: usize,
    pub(super) fh: usize,
    pub(super) rx0: usize,
    pub(super) ry0: usize,
    pub(super) rw: usize,
    pub(super) rh: usize,
    pub(super) commit: bool,
    pub(super) cur_o: watercolor_field::Dono,
    pub(super) cov_src: &'a [f32],
    pub(super) rim: &'a watercolor_rim::RimFields,
    pub(super) rewet: &'a Option<watercolor_field::RewetFields>,
    pub(super) lut: &'a watercolor_lut::Luts,
    pub(super) base: &'a [u8],
    pub(super) ground: &'a [u8],
    pub(super) smooth_edges: bool,
    pub(super) noise_tile: NoiseTile,
    pub(super) has_color: bool,
    pub(super) color_buf: &'a Vec<u8>,
    pub(super) has_dens: bool,
    pub(super) dens_buf: &'a Vec<u8>,
    pub(super) style_table: &'a [WetStrokeStyle],
    pub(super) style_owner: &'a [watercolor_field::Dono],
    /// O dono resolvido do halo (B16.2, [`super::dono`]).
    pub(super) halo: &'a super::dono::DonoDoHalo,
    pub(super) has_style: bool,
    pub(super) cur_style: WetStrokeStyle,
    pub(super) substrate_session: &'a SubstrateSession<'a>,
    pub(super) soaked: bool,
    pub(super) soak_buf: &'a [u8],
    pub(super) watered: bool,
    pub(super) water_buf: &'a [u8],
    pub(super) wet_field: &'a Option<(Vec<f32>, Vec<f32>)>,
    pub(super) style_field: &'a Option<StyleField>,
    pub(super) reserve: &'a Option<watercolor_reserve::ReserveFields>,
    pub(super) use_substrate_cache: bool,
    pub(super) substrate: &'a [f32],
    pub(super) flow: &'a watercolor_flow::EdgeFlow,
    pub(super) paper_edge_any: bool,
    pub(super) gate_on: bool,
    pub(super) gsel: Option<&'a [u8]>,
    pub(super) gprot: Option<&'a [u8]>,
    pub(super) alock_on: bool,
    pub(super) vbase: Option<&'a [Vidro]>,
    pub(super) vchao: Option<&'a [u8]>,
}

impl Quadro<'_> {
    /// Compõe o texel `(gx, gy)` na `row` de saída (a linha `gy`) e, havendo plano, o seu vidro.
    #[inline]
    pub(super) fn texel(
        &self,
        gx: usize,
        gy: usize,
        row: &mut [u8],
        mut vrow: Option<&mut [Vidro]>,
    ) {
        let Quadro {
            fw,
            fh,
            rx0,
            ry0,
            rw,
            rh,
            commit,
            cur_o,
            cov_src,
            rim,
            rewet,
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
            halo,
            has_style,
            ref cur_style,
            substrate_session,
            soaked,
            soak_buf,
            watered,
            water_buf,
            wet_field,
            style_field,
            reserve,
            use_substrate_cache,
            substrate,
            flow,
            paper_edge_any,
            gate_on,
            gsel,
            gprot,
            alock_on,
            vbase,
            vchao,
        } = *self;
        let ly = (gy - ry0) as f32;
        let lx = (gx - rx0) as f32;
        let gi = (gy * fw + gx) * 4;
        // Warp the sample position (organic boundary). Window-local coords for the read-window
        // fields; global for the full-canvas colour buffer (same displacement + window origin).
        // Per-stroke style: warp AMPLITUDE by the pixel's owner (read PRE-warp —
        // the displacement needs the amp first); owner 0 = current brush, old path.
        let o_pre = if has_style {
            halo.em(gx, gy, style_owner[gy * fw + gx])
        } else {
            0
        };
        let st_warp = style_at(has_style, o_pre, style_table, cur_style).warp;
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
                cov_src,
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
                smoothstep(SS0, SS1, sample_bilinear(cov_src, rw, rh, sx, sy)),
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
            if let (Some(v), Some(b)) = (vrow.as_deref_mut(), vbase) {
                v[gx] = b[gy * fw + gx]; // e o vidro da base com ela
            }
            return;
        }
        // Warped canvas-space indices — shared by the tip-density / pigment-reserve /
        // style-owner reads (nearest, like the colour buffer).
        let wgx = (rx0 as f32 + sx).clamp(0.0, (fw - 1) as f32) as usize;
        let wgy = (ry0 as f32 + sy).clamp(0.0, (fh - 1) as f32) as usize;
        let widx = wgy * fw + wgx;
        let dono_px = if has_style {
            halo.em(wgx, wgy, style_owner[widx])
        } else {
            0
        };
        let st = style_at(has_style, dono_px, style_table, cur_style);
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
            sample_bilinear(blur_of(&rim.blurs, st.core_r as usize), rw, rh, sx, sy).min(1.0);
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
        let didx = pigment::texel_do_deposito(
            st.cor_no_papel,
            has_color.then_some(color_buf.as_slice()),
            gy * fw + gx,
            widx,
        );
        let ci = didx * 4;
        let col_a = if has_color {
            f32::from(color_buf[ci + 3]) / 255.0
        } else {
            1.0
        };
        // EDGE-4: the rim tells the GESTURE's story — stronger where the water pooled
        // or lingered (soak), fainter where the brush barely deposited (a depleted
        // trail's dry tail). Zero reads beyond fields that already exist.
        let gain_px = st_edge_gain * (1.0 + EDGE_SOAK_BOOST * soak_v) * (0.5 + 0.5 * col_a);
        // EDGE-3 (Curtis §4.3.3, conservação): rim = unsharp ASSINADO — o lobo
        // negativo (franja, `inner > cw`) EMPALIDECE (o pigmento da borda MIGROU do
        // interior; era clampado ≥0 = wash uniforme com contorno). Doc 12 §W-C.
        let mut edge = watercolor_rim::densidade_do_aro(gain_px, cw, inner);
        // Paper tooth + Granulation source + paper-depth component — resolved per OWNER
        // (#13): the cached value when single-substrate (byte-identical), else recomputed
        // from the owner's paper. See [`SubstrateSession::at`].
        let cached_ph = use_substrate_cache.then(|| substrate[gy * fw + gx]);
        let (paper_h, gran_h, paper_component) =
            substrate_session.at(st, dono_px, cached_ph, gx, gy);
        // Take 10: TODO termo wet-driven lê o campo borrado ([`sample_wet_field`]) —
        // o thinning do interior era o degrau de ~11 bytes/px na fronteira de dono.
        let st_wet_px = sample_wet_field(wet_field.as_ref(), (rw, rh), (sx, sy), st.wet);
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
            fill_px = st_fill * (1.0 - (WET_THIN * st_wet_px * st.spread_thin * inner).min(0.95));
            let ragged = (1.0 + (0.5 - paper_h) * 2.0 * WET_RAGGED * st_wet_px).max(0.0);
            edge = (edge * (1.0 + WET_EDGE_BOOST * st_wet_px) * ragged).min(1.5); // LITERAL-PX-OK: wet edge may overshoot the dry clamp; signed (EDGE-3) keeps the pale lobe
        }
        // Tip density at the deposit texel — the colour buffer's (`didx`).
        let tip_dens = if has_dens && st.ponta_texturada {
            f32::from(dens_buf[didx]) / 255.0
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
            st.deposito_dono,
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
        // ligado não desbota a lavagem ([`super::super::watercolor_mistura`], medido 2026-09-24).
        let tinta = if st.pigment_mix > 0.0 {
            super::super::watercolor_mistura::presenca_de_tinta(base, ground, gi).0
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
        // O VIDRO deste texel (doc 48): a mesma óptica sobre o chão PRETO, com a base posta
        // sobre ele pelo vidro DELA; o que muda de um chão ao outro, canal a canal, é o que
        // o chão atravessa ([`super::super::watercolor_vidro::alfas_do_vidro`]).
        let vidro_do = |px: [u8; 4], b: &[crate::compositor::vidro::Vidro]| {
            let i = gy * fw + gx;
            let cb = vchao.map_or([0u8; 3], |g| [g[gi], g[gi + 1], g[gi + 2]]);
            let ab_c = crate::compositor::vidro::alfas(&base[gi..gi + 4], &b[i]);
            let base_preto: [f32; 3] = core::array::from_fn(|c| {
                base_sobre[c]
                    - (1.0 - f32::from(ab_c[c]) / 255.0) * (f32::from(chao[c]) - f32::from(cb[c]))
                        / 255.0
            });
            let preto = aparencia::aparencia(&optica, base_preto, cb, lut).rgb;
            crate::compositor::vidro::sela(
                px,
                super::super::watercolor_vidro::alfas_do_vidro(rgb, preto, chao, cb, px[3]),
            )
        };
        if out_a <= f32::EPSILON {
            // No film and no base: the layer stays untouched (appearance == ground) — mas a
            // orla que sobre o branco se anula pode ver-se sobre o papel de cor: o vidro sela-a.
            row[gx * 4] = base[gi];
            row[gx * 4 + 1] = base[gi + 1];
            row[gx * 4 + 2] = base[gi + 2];
            row[gx * 4 + 3] = base[gi + 3];
            if let (Some(v), Some(b)) = (vrow.as_deref_mut(), vbase) {
                v[gx] = vidro_do([base[gi], base[gi + 1], base[gi + 2], base[gi + 3]], b);
            }
            return;
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
        let mut mantem = 1.0f32;
        if gate_on {
            let keep = watercolor_accum::splat_keep(gsel, gprot, None, gy * fw + gx);
            if keep < 1.0 {
                mantem = keep;
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
        // A seleção e a proteção misturam o texel com a base por `keep`: os alfas misturam-se
        // na mesma proporção. Com o alpha-lock a cobertura não muda: os alfas são os da base.
        if let (Some(v), Some(b)) = (vrow, vbase) {
            let da_base = crate::compositor::vidro::alfas(&base[gi..gi + 4], &b[gy * fw + gx]);
            let alfas = if alock_on {
                da_base
            } else {
                let novo = vidro_do(px, b);
                core::array::from_fn(|c| {
                    let (a0, a1) = (f32::from(da_base[c]), f32::from(novo[4 + c]));
                    (a0 + (a1 - a0) * mantem).round() as u8
                })
            };
            v[gx] = crate::compositor::vidro::sela(px, alfas);
        }
    }
}
