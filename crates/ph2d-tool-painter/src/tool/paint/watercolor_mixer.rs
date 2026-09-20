//! Watercolor **Wet Mix** mixer-brush (Procreate Wet Mix / MyPaint-Krita "Dulling", `docs/Painter/07`
//! §4) — the per-stroke colour-pickup + carry that makes rediluição DIRECTIONAL (the pigment a wet
//! brush crosses is dragged along the gesture), on top of the per-pixel `wet_rewet` diffusion.
//!
//! Per dab, in stroke order, the brush:
//! 1. resamples the FROZEN pre-stroke surface under its disc — the base composited over the real
//!    [`ground`](super::watercolor_backdrop), weighted by paint **presence** (how far it departs from
//!    the local ground, the same reference the rewet uses) — into a running-average **reservoir**;
//! 2. **asymmetric** running-average: it LOADS fast toward more paint (a strong pickup the moment it
//!    enters a pool) and UNLOADS slowly toward bare ground, so the picked-up colour LINGERS a few
//!    dabs past the pool — the EXIT bleed mirrors the ENTRY instead of hard-cutting. **Pull** raises
//!    the unload retention toward a long smudge (the carried colour dragged far downstream);
//! 3. deposits `lerp(brush, reservoir, (1 − charge)·w)` — a fully **Charged** brush (default `1`)
//!    deposits pure fresh colour (the mixer is skipped entirely → byte-identical); a depleted brush
//!    smears what it picked up.
//!
//! **Subtractive colour (W-A, doc 12 OPT-1):** the reservoir and the deposit blend in per-channel
//! ABSORBANCE (the `lnl`/`exp_mag` LUTs — the dissolve's ratified pattern), so the carried mix
//! behaves like PIGMENT: blue across a yellow pool deposits green, never the sRGB-lerp grey.
//!
//! **No self-feeding** (the retired reservoir's failure mode, [[project-wash-undo…]] lesson): the
//! pickup reads `watercolor_base` (frozen at pen-down) over the frozen backdrop, NEVER the live
//! canvas. **No cadence-binding**: the resample clock is Pull/distance-driven (per dab), not per
//! frame. The visible signature is the CARRY (Pull) + the pickup fraction (Charge) — not a weak
//! colour tint. Deterministic (HR-5): only sums/mults over integer-indexed samples.

use super::*;

/// The mixer's per-stroke reservoir: the picked-up colour as a premultiplied per-channel
/// **ABSORBANCE** triple (`A = −ln(linear)`, `0` = white / no pigment — W-A, doc 12 OPT-1: paints
/// mix like PIGMENTS, in log-transmittance space, never by sRGB lerp, which turned blue over a
/// yellow pool into grey) + a presence-weighted confidence `w ∈ [0, 1]` (how much real paint it
/// holds). `w == 0` (fresh / over bare ground) ⇒ the deposit is pure brush colour. Kept in FLOAT
/// end-to-end (the audit's drift warning applies to iterative byte re-quantisation only).
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct WetMix {
    absorb: [f32; 3],
    w: f32,
    /// Stroke travel in px (MIX-1, doc 12 — Procreate Charge depletion: *"the longer you drag your
    /// stroke out... the trail of color it leaves will become fainter"*). Advanced once per batch by
    /// the colour pass; [`PainterTool::wet_mix_depletion`] REPLAYS the chain from the stored value
    /// (the rng discipline) to feed the coverage pass's per-pixel pigment-reserve map.
    travel: f32,
    /// The previous dab centre of the travel chain (`None` = stroke start).
    last_pos: Option<[f32; 2]>,
}

/// Reservoir **load** retention (entering paint): low ⇒ the reservoir tracks the surface FAST, so the
/// brush picks up a pool strongly the moment it enters. Kept separate from the unload retention so
/// the pickup is strong AND the exit bleed is long (a single rate couples the two: fast enough to
/// pick up hard is too fast to carry — Enio 2026-07-07).
const RETAIN_LOAD: f32 = 0.2;
/// Reservoir **unload** retention floor (leaving paint) at Pull 0 — a wet brush does not forget a
/// picked-up colour the instant it leaves the pool, so the EXIT bleed persists a few dabs and mirrors
/// the ENTRY (the reported hard-cut exit). Pull raises it toward a long smudge.
const RETAIN_UNLOAD_MIN: f32 = 0.88;
/// MIX-1 — Charge depletion span: the fresh-paint reserve lasts `SPAN · radius · charge/(1−charge)`
/// px of stroke travel. Charge controls how LONG the reserve lasts, NEVER the head intensity — the
/// reserve always starts FULL (`fresh = 1` at travel 0), because any head factor < 1 scaled the
/// coverage below saturation and flooded the interior with residual edge term (the flat opaque slab,
/// Enio smoke 2026-07-08). The `charge/(1−charge)` factor approaches ∞ as Charge → 1, meeting the
/// `wet_charge < 1` gate with no seam. Charge 0.5 on a 16-px brush ⇒ ~1900 px of paint; 0.25 ⇒
/// ~640 px; 0.1 ⇒ ~210 px. Calibration knob (Enio smoke 2026-07-08): 40 died off too fast on
/// canvas — "o esmaecimento lento estava melhor" → 120.
const MIX_DEPLETE_SPAN: f32 = 120.0;
/// MIX-1 — the reservoir absorbance (max channel, unpremultiplied) that counts as "fully loaded
/// pigment" for the carry: `A = 2` ⇒ transmittance `e⁻² ≈ 0.135`, a rich dark pool. The carry
/// intensity is `t × (A_max / this).min(1)`, so a barely-tinted pool (low absorbance) sustains only
/// a faint smudge — crossing a pale wash must NOT re-ink a depleted brush at full strength (Enio
/// smoke 2026-07-08: "explode em muito pigmento" ignoring both the brush's and the pool's intensity).
const MIX_CARRY_FULL_ABSORB: f32 = 2.0;

impl PainterTool {
    /// Whether the Wet Mix mixer drives this stroke's deposited colour: watercolor render-path on and
    /// `wet_charge < 1` (some pickup). Off ⇒ the deposit is the plain per-dab brush colour (byte-
    /// identical), and the whole pickup path is skipped.
    pub(super) fn wet_mixer_active(&self) -> bool {
        self.watercolor_render_active() && self.paint.brush.wet_charge < 1.0
    }

    /// Reset the mixer reservoir for a beginning stroke (fresh brush, no pickup yet).
    pub(super) fn reset_wet_mix(&mut self) {
        self.paint.wet_mix = WetMix::default();
    }

    /// Compute the per-dab **deposited** colour + **deposit priority** for `dabs`, advancing the mixer
    /// reservoir in stroke order. The priority (`pickup × load`, `0..1`) scales the colour's deposit
    /// alpha at splat time ([`super::watercolor_accum`]): a HIGH-pickup dab (in a pool) dominates, a
    /// LOW-pickup one (leaving the pool, back over bare ground) barely writes — so the picked-up colour
    /// is NOT overwritten by the following bare-ground dabs (source-over recency). That is what makes
    /// the pool's EXIT edge as coloured as its ENTRY edge: without it, the last dab over a pixel won,
    /// and on the exit side that was a bare-ground dab, so the crossing read asymmetric (Enio
    /// 2026-07-07 — misattributed to Dilution, which is symmetric; the mixer is the source, and
    /// Dilution only made it more visible by thinning the wash). Priority `1.0` for every dab when the
    /// mixer is off ⇒ the plain source-over path (byte-identical). Reads the frozen base + backdrop
    /// (cloned `Arc`s, so no borrow clash with the caller's `stroke_color` mutation).
    pub(super) fn wet_mix_dab_colors(&mut self, dabs: &[Dab]) -> Vec<([f32; 3], f32, f32)> {
        if !self.wet_mixer_active() {
            return dabs.iter().map(|d| (d.color, 1.0, 1.0)).collect();
        }
        let (fw, fh) = self.source_size;
        let (fw, fh) = (fw as usize, fh as usize);
        let base = self.paint.watercolor_base.as_ref().map(Arc::clone);
        let backdrop = self.paint.wet_backdrop.as_ref().map(Arc::clone);
        let (Some(base), Some(backdrop)) = (base, backdrop) else {
            return dabs.iter().map(|d| (d.color, 1.0, 1.0)).collect();
        };
        if base.len() != fw * fh * 4 || backdrop.len() != fw * fh * 4 || fw == 0 || fh == 0 {
            return dabs.iter().map(|d| (d.color, 1.0, 1.0)).collect();
        }
        let pickup = (1.0 - self.paint.brush.wet_charge).clamp(0.0, 1.0);
        let p = self.paint.brush.wet_pull.clamp(0.0, 1.0);
        // Unload retention rises with Pull (`p·(2−p)` concave, transcendental-free, HR-5 safe): Pull 0
        // = a short baseline carry (symmetric exit bleed), Pull → 1 = a long smudge. The LOAD rate is
        // fixed + fast (RETAIN_LOAD) so the pickup is always strong.
        let unload =
            (RETAIN_UNLOAD_MIN + (0.98 - RETAIN_UNLOAD_MIN) * p * (2.0 - p)).clamp(0.0, 0.98);
        let lut = super::watercolor_field::luts();
        let charge = self.paint.brush.wet_charge.clamp(0.0, 1.0);
        let mut out = Vec::with_capacity(dabs.len());
        let mut mix = self.paint.wet_mix;
        for d in dabs {
            // MIX-1 — advance the travel chain (one advance per batch, here; the depletion itself is
            // computed by `wet_mix_depletion`'s replay and lands in the per-pixel pigment map).
            if let Some(prev) = mix.last_pos {
                let (dx, dy) = (d.center[0] - prev[0], d.center[1] - prev[1]);
                mix.travel += (dx * dx + dy * dy).sqrt();
            }
            mix.last_pos = Some(d.center);
            // ── 1. Resample the frozen surface under the disc every dab (star), then ASYMMETRIC
            //    running-average in PREMULTIPLIED space: LOAD fast toward more paint (strong pickup
            //    on entry), UNLOAD slow toward bare ground (the carried colour lingers past the pool
            //    → the EXIT bleed mirrors the ENTRY). Bare ground (`sw = 0`) only DEPLETES the load
            //    (× update); it never pulls the carried hue toward the ground (premul), so `rgb / w`
            //    stays the picked-up colour. ──
            let (srgb, sw) = sample_surface(&base, &backdrop, fw, fh, d.center, d.radius_px);
            let update = if sw >= mix.w { RETAIN_LOAD } else { unload };
            // Convert the sampled appearance to ABSORBANCE at the mixer boundary (W-A): the running
            // average then blends log-transmittances — a geometric mean of paints, not a light mix.
            for (channel, &s) in mix.absorb.iter_mut().zip(srgb.iter()) {
                let byte = (s.clamp(0.0, 1.0) * 255.0 + 0.5) as usize;
                let a = -lut.lnl[byte.min(255)]; // absorbance ≥ 0; white = 0 (no pigment)
                *channel = update * *channel + (1.0 - update) * sw * a;
            }
            mix.w = update * mix.w + (1.0 - update) * sw;
            // ── 2. Deposit: blend the brush colour toward the (unpremultiplied) reservoir colour by
            //    the priority `t = pickup × load`, in ABSORBANCE space (W-A, doc 12 OPT-1): the
            //    subtractive lerp is what makes blue over a yellow pool deposit GREEN — the sRGB
            //    lerp it replaces deposited grey ("blue and yellow make gray", Mixbox's flagship
            //    defect; probe 2026-07-07: deposit (128,128,115) R≈G). Load `w` decays downstream ⇒
            //    the carried colour fades with distance, AND `t` decays with it — so a fading exit
            //    dab can't overwrite a stronger in-pool deposit (symmetric crossing). ──
            let t = (pickup * mix.w).clamp(0.0, 1.0);
            let mut col = d.color;
            if mix.w > 1e-4 {
                let inv = 1.0 / mix.w;
                // O lerp em absorbância APARENTE deposita mais claro que a brocha ao cruzar o
                // próprio wash (filme fino = absorbância baixa) — o CLAREAMENTO da junção. Uma
                // re-escala de magnitude que o anulava foi implementada e REVERTIDA (Enio
                // 2026-07-09, take 9→10: "perdeu o efeito de clareamento" — o clareamento é o
                // look desejado; o defeito era só a FRONTEIRA dura dele, atacada na rampa
                // espacial do load acima).
                for ((out_c, &m), &brush_c) in
                    col.iter_mut().zip(mix.absorb.iter()).zip(d.color.iter())
                {
                    let sa = m * inv; // unpremultiply → the carried paint's absorbance
                    let byte = (brush_c.clamp(0.0, 1.0) * 255.0 + 0.5) as usize;
                    let ba = -lut.lnl[byte.min(255)]; // the brush pigment's absorbance
                    let mag = ba + (sa - ba) * t; // subtractive mix (log-space lerp)
                    *out_c = f32::from(lut.l2s_byte(lut.exp_mag(mag))) / 255.0;
                }
            }
            // Deposit intensity for the COLOUR alpha (mirror of the coverage map's factors): a
            // depleted brush must not re-TINT a wet pool at full priority — in a wet session the
            // pool re-renders with whatever colour was deposited over it, so the tint has to track
            // the actual pigment on the brush (fresh reserve OR carried pickup), not just the mix
            // weight `t`.
            let depl =
                deplete_fresh(mix.travel, d.radius_px, charge).max(t * reservoir_pigment(&mix));
            out.push((col, t, depl));
        }
        self.paint.wet_mix = mix;
        out
    }

    /// The per-dab pigment-reserve factors (`fresh ∨ carry`, `0..1`) for the COVERAGE pass's
    /// per-pixel depletion map — REPLAYS the travel chain from the stored `wet_mix` without writing
    /// it back (the colour pass advances it once per batch — the rng replay discipline).
    /// `None` when the mixer is off (no depletion — byte-identical default).
    /// ⚠️ Devolve `(reserva, percurso)` por dab: o percurso ja' e' calculado aqui, e o plano do
    /// ARCO precisa exactamente dele — deriva-lo outra vez no passe de cobertura seria a segunda
    /// resposta a' mesma pergunta.
    pub(super) fn wet_mix_depletion(&self, dabs: &[Dab]) -> Option<Vec<(f32, f32)>> {
        if !self.wet_mixer_active() {
            return None;
        }
        let charge = self.paint.brush.wet_charge.clamp(0.0, 1.0);
        let pickup = (1.0 - charge).clamp(0.0, 1.0);
        let mut travel = self.paint.wet_mix.travel;
        let mut last = self.paint.wet_mix.last_pos;
        // Self Pickup (doc 40 §S2-C) — a vista do depósito VIVO. ⭐ Ela vive AQUI, no replay do
        // passe de COBERTURA, e não no avanço do passe de cor: quem escreve o mapa de pigmento
        // (`stroke_deplete`, o que o composite lê) é a cobertura, e ela corre PRIMEIRO. Ligado ao
        // avanço, o termo alimentava só o alfa da cor — que em papel virgem é saltado
        // (`if a <= 0 { continue }`, doc 40 §9.2) ⇒ media-se `live_pig = 0,58` e a tela não mexia.
        //
        // ⚠️ E ficar aqui torna a recolha **independente do corte dos lotes** de graça: os planos
        // ainda não têm os dabs DESTE lote, e os do lote anterior estão a menos de uma fracção de
        // diâmetro ⇒ a cerca de idade fecha sobre eles de qualquer maneira.
        let (fw, fh) = self.source_size;
        let (fw, fh) = (fw as usize, fh as usize);
        let n_tex = fw * fh;
        let gain = self.paint.brush.wet_self_pickup.clamp(0.0, 1.0);
        let live_planes = (gain > 0.0
            && self.paint.stroke_arc.len() == n_tex
            && self.paint.stroke_coverage.len() == n_tex
            && self.paint.stroke_deplete.len() == n_tex
            && self.paint.stroke_color.len() == n_tex * 4)
            .then_some((
                &self.paint.stroke_coverage[..],
                &self.paint.stroke_deplete[..],
                &self.paint.stroke_arc[..],
                &self.paint.stroke_color[..],
            ));
        // The carry factor needs the reservoir's evolution too — approximate with the CURRENT stored
        // state (exact per-dab values need the full pickup resample; the coverage fade tolerance is
        // perceptual, and the colour pass applies the exact factor to the deposit itself).
        let carry = (pickup * self.paint.wet_mix.w).clamp(0.0, 1.0)
            * reservoir_pigment(&self.paint.wet_mix);
        let mut out = Vec::with_capacity(dabs.len());
        for d in dabs {
            if let Some(prev) = last {
                let (dx, dy) = (d.center[0] - prev[0], d.center[1] - prev[1]);
                travel += (dx * dx + dy * dy).sqrt();
            }
            last = Some(d.center);
            // ⭐ O item 4 em uma linha: a reserva recolhida do próprio traço é mais um PISO do
            // pigmento que o pincel leva, ao lado do fresco e do carry. Com o knob em `0` o
            // `live_planes` é `None` ⇒ o termo é `0` e a expressão volta à de sempre, ao bit.
            let live = live_planes.map_or(0.0, |(cov, lvl, arc, col)| {
                let l = LivePickup {
                    cov,
                    lvl,
                    arc,
                    col,
                    now: arc_stamp(travel),
                    min_age: ((PICKUP_AGE_DIAMETERS * 2.0 * d.radius_px) / ARC_UNIT_PX).max(1.0)
                        as u16,
                };
                live_reserve(&l, fw, fh, d.center, d.radius_px)
            });
            // ⭐⭐ O knob INTERPOLA, nunca disputa — e a diferença é o que o torna um mostrador.
            // Escrito como `base.max(live * gain)` (a 1.ª redacção, o idioma `fresco ∨ carry` da
            // casa) ele vira um DEGRAU: a recolha só morde quando `live·gain` passa o fresco, e
            // abaixo disso ela morre ao primeiro dab, porque o próprio dab dilui o nível que o
            // seguinte vai amostrar. MEDIDO no nível da perna de volta: `67 → 69 → 137` para
            // `0 / 0,5 / 1` — `0,5` do curso comprava `3 %` do efeito. Interpolando, o meio curso
            // é o meio do efeito, e os dois extremos ficam onde estavam (`gain = 0` ⇒ `base`, ao
            // bit; `gain = 1` ⇒ o mesmo `max` de antes, porque só se interpola para CIMA).
            let base = deplete_fresh(travel, d.radius_px, charge).max(carry);
            out.push((base + gain * (live - base).max(0.0), travel));
        }
        Some(out)
    }
}

/// MIX-1 fresh reserve at `travel` px into the stroke: starts FULL (1.0 — the head always carries
/// the complete watercolor anatomy) and runs out over a span that grows with Charge, unbounded as
/// Charge → 1 (seamless against the `wet_charge < 1` mixer gate).
///
/// The decay is QUADRATIC in density so it reads LINEAR to the eye (Enio smoke 2026-07-08: the
/// linear-in-density take "está caindo mais rapidamente no fim"): the composite maps density
/// through Beer–Lambert (`T = e^{−pig·od}`), which is COMPRESSIVE at the dark end — near the
/// head a density drop barely shows, near the tail the same drop plummets. `(1−u)²` front-loads
/// the density loss where vision can't see it and lands gently where it can — the perceived fade
/// rate stays constant along the whole trail.
fn deplete_fresh(travel: f32, radius_px: f32, charge: f32) -> f32 {
    let span = (MIX_DEPLETE_SPAN * radius_px * charge / (1.0 - charge).max(1e-3)).max(1.0);
    let left = (1.0 - travel / span).max(0.0);
    left * left
}

/// How much PIGMENT the reservoir actually holds (`0..1`): its strongest unpremultiplied absorbance
/// channel against [`MIX_CARRY_FULL_ABSORB`]. A pale pickup (near-white pool) scores near 0 — the
/// carry it can sustain is proportionally faint.
fn reservoir_pigment(mix: &WetMix) -> f32 {
    if mix.w <= 1e-4 {
        return 0.0;
    }
    let a_max = mix.absorb.iter().fold(0.0f32, |m, &a| m.max(a)) / mix.w;
    (a_max / MIX_CARRY_FULL_ABSORB).min(1.0)
}

/// Average the frozen **surface appearance** (base over ground) + its paint **presence** under the
/// disc at `center` (radius `r`), via a cheap 5-tap star (centre + 4 mid-radius points). Presence =
/// the max per-channel departure from the local ground, dead-zoned like the rewet (`14→50` bytes), so
/// bare ground contributes `w = 0` (no pickup there). Returns `(straight sRGB 0..1, presence 0..1)`.
/// A unidade em que o arco da primeira cobertura é guardado (px de percurso por passo do `u16`).
///
/// ⚠️ **O recurso é o alcance, não a precisão:** a `4 px` o `u16` cobre `262 143 px` de percurso
/// num traço, e o que se perde é a resolução da idade — que é comparada contra um limiar de
/// *diâmetros*, ou seja dezenas a centenas de px. Um traço mais longo que isso satura, e a
/// saturação é o lado CONSERVADOR (idade lida como `0` ⇒ não se recolhe).
pub(super) const ARC_UNIT_PX: f32 = 4.0;

/// A idade mínima, em DIÂMETROS de percurso, para um texel do próprio traço ser recolhível.
///
/// ⭐ **Este número é DERIVADO, não escolhido.** Num traço recto o tap mais atrasado do amostrador
/// fica a `r/2` atrás do centro, e um texel ali foi coberto pela primeira vez quando o centro
/// estava a `r/2 + r = 1,5 r` — ou seja **`0,75` diâmetros** de idade. Qualquer limiar acima disso
/// fecha a cerca num traço recto; `2,5` deixa **`3,3×`** de margem. Do outro lado, a perna de ida de
/// um U de raio `32` é cruzada com `~5,8` diâmetros de idade ⇒ a cerca abre. O gate
/// `a_straight_stroke_never_feeds_on_its_own_trail` mede o lado fechado e o
/// `the_return_leg_picks_up_the_pigment_it_crosses` o lado aberto.
pub(super) const PICKUP_AGE_DIAMETERS: f32 = 2.5;

/// O carimbo de arco de um percurso: `0` fica reservado para «nunca coberto».
#[inline]
pub(super) fn arc_stamp(travel: f32) -> u16 {
    let q = (travel / ARC_UNIT_PX).max(0.0);
    1 + (q.min(f32::from(u16::MAX - 1))) as u16
}

/// A vista do depósito VIVO que o Self Pickup lê (doc 40 §S2-C) — `None` no caminho de fábrica.
pub(super) struct LivePickup<'a> {
    /// Cobertura do traço (`stroke_coverage`).
    pub cov: &'a [u8],
    /// NÍVEL da reserva depositada (`stroke_deplete`) — o pigmento que de facto está lá.
    pub lvl: &'a [u8],
    /// O arco da PRIMEIRA cobertura (`stroke_arc`), `0` = nunca coberto.
    pub arc: &'a [u16],
    /// A cor depositada (`stroke_color`, RGBA) — só se recolhe COR onde o alfa dela é > 0.
    pub col: &'a [u8],
    /// O carimbo de arco AGORA.
    pub now: u16,
    /// A idade mínima, já em unidades de arco.
    pub min_age: u16,
}

impl LivePickup<'_> {
    /// O que este texel oferece: `(presença, Some(rgb) se ele tem cor própria)`.
    ///
    /// ⚠️ A cor vem `None` em papel virgem **de propósito**: deixá-la acender tira os dabs da volta
    /// do caminho *«pula»* do passe de cor e põe-nos no *«deposita»*, medido a `2,6×` o carimbo por
    /// dab (doc 40 §9.4) — e é trabalho inútil, porque num traço de uma cor só a cor recolhida **é**
    /// a do pincel, que o composite já usa como recurso.
    #[inline]
    fn at(&self, i: usize) -> (f32, Option<[f32; 3]>) {
        let a = self.arc[i];
        if a == 0 || self.now.saturating_sub(a) < self.min_age {
            return (0.0, None);
        }
        let pig = (f32::from(self.cov[i]) / 255.0) * (f32::from(self.lvl[i]) / 255.0);
        if pig <= 0.0 {
            return (0.0, None);
        }
        let ci = i * 4;
        let rgb = (self.col.get(ci + 3).copied().unwrap_or(0) > 0).then(|| {
            [
                f32::from(self.col[ci]) / 255.0,
                f32::from(self.col[ci + 1]) / 255.0,
                f32::from(self.col[ci + 2]) / 255.0,
            ]
        });
        (pig.clamp(0.0, 1.0), rgb)
    }
}

/// A **reserva** que o pincel recolhe do PRÓPRIO traço sob o disco (doc 40 §S2-C) — o máximo sobre
/// os MESMOS 5 taps do [`sample_surface`].
///
/// ⛔ **Cinco taps e não a integral do disco:** ela parece «mais correcta» e custa uma segunda
/// caminhada do disco por dab — **`+58 %`** do passe de cobertura nos dois raios medidos
/// (doc 40 §9.4). Recusa medida.
///
/// ⚠️ Este canal **não toca na cor**: em papel virgem com `Charge < 1` o passe de cor toma o
/// caminho «pula» e o `stroke_color` nunca é escrito (doc 40 §8), logo somar presença ao
/// reservatório de cor puxá-lo-ia para o BRANCO e a volta sairia mais CLARA — o oposto do item 4.
fn live_reserve(l: &LivePickup<'_>, fw: usize, fh: usize, center: [f32; 2], r: f32) -> f32 {
    let rr = (r * 0.5).max(0.0);
    let taps = [
        (center[0], center[1]),
        (center[0] - rr, center[1]),
        (center[0] + rr, center[1]),
        (center[0], center[1] - rr),
        (center[0], center[1] + rr),
    ];
    let mut best = 0.0f32;
    for (tx, ty) in taps {
        if tx < 0.0 || ty < 0.0 {
            continue;
        }
        let (x, y) = (tx as usize, ty as usize);
        if x >= fw || y >= fh {
            continue;
        }
        best = best.max(l.at(y * fw + x).0);
    }
    best
}

fn sample_surface(
    base: &[u8],
    ground: &[u8],
    fw: usize,
    fh: usize,
    center: [f32; 2],
    r: f32,
) -> ([f32; 3], f32) {
    // Tap ring at HALF the dab radius — FIXED. A configurable pickup radius (Procreate's Wet Mix
    // Blur) was exposed and REVERTED (Enio 2026-07-07: "funcionava melhor quando ele não era
    // configurável") — don't re-expose without a smoke that says otherwise.
    let rr = (r * 0.5).max(0.0);
    let taps = [
        (center[0], center[1]),
        (center[0] - rr, center[1]),
        (center[0] + rr, center[1]),
        (center[0], center[1] - rr),
        (center[0], center[1] + rr),
    ];
    let mut acc = [0.0f32; 3];
    let mut pres = 0.0f32;
    let mut psum = 0.0f32; // sum of tap presences — the colour normaliser
    let mut n = 0.0f32;
    for (tx, ty) in taps {
        if tx < 0.0 || ty < 0.0 {
            continue;
        }
        let (x, y) = (tx as usize, ty as usize);
        if x >= fw || y >= fh {
            continue;
        }
        let bi = (y * fw + x) * 4;
        let ab = f32::from(base[bi + 3]) / 255.0;
        let (gr, gg, gb) = (
            f32::from(ground[bi]),
            f32::from(ground[bi + 1]),
            f32::from(ground[bi + 2]),
        );
        let rgb = [
            f32::from(base[bi]) * ab + gr * (1.0 - ab),
            f32::from(base[bi + 1]) * ab + gg * (1.0 - ab),
            f32::from(base[bi + 2]) * ab + gb * (1.0 - ab),
        ];
        let dd = (gr - rgb[0])
            .abs()
            .max((gg - rgb[1]).abs())
            .max((gb - rgb[2]).abs());
        let pt = smooth_pres(dd);
        // PRESENCE-WEIGHT the colour: a bare-ground tap (`pt ≈ 0`) contributes almost nothing to the
        // hue, so a disc half over a red pool picks up SATURATED red (weight 0.5), not a pink average
        // of red + white. Averaging the raw colour instead leaked the ground into the reservoir, so
        // the carried mix read watery / bleached toward white rather than a rich mix (Enio 2026-07-07).
        for c in 0..3 {
            acc[c] += pt * rgb[c] / 255.0;
        }
        pres += pt;
        psum += pt;
        n += 1.0;
    }
    if n <= 0.0 || psum <= 1e-4 {
        return ([0.0; 3], 0.0);
    }
    let inv_c = 1.0 / psum;
    (
        [acc[0] * inv_c, acc[1] * inv_c, acc[2] * inv_c],
        (pres / n).clamp(0.0, 1.0),
    )
}

/// Dead-zone presence ramp (bytes `14 → 50`), matching the rewet's `PAINT_LO`/`PAINT_HI` so the
/// mixer and the per-pixel rewet agree on "what is liftable paint".
#[inline]
fn smooth_pres(d: f32) -> f32 {
    // Cubic smoothstep over [14, 50] (transcendental-free).
    let t = ((d - 14.0) / (50.0 - 14.0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
