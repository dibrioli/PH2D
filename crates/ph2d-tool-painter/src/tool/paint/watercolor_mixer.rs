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
        // [wet-diag v2] TEMP: pickup do dab anterior — o print MIX1 abaixo dispara só no SALTO
        // (travessia de poça), poucas linhas por cruzamento.
        let mut diag_t_prev = (pickup * mix.w).clamp(0.0, 1.0);
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
                // Mistura subtrativa por canal (o MATIZ), depois re-escala de magnitude: o
                // reservatório amostra a APARÊNCIA do filme fino (absorbância baixa — rosa
                // pálido num wash vermelho), e o lerp cru DILUÍA o depósito — cruzar o próprio
                // wash CLAREAVA a junção (mancha pálida + degrau serrilhado na janela COL_LO..HI,
                // smoke 2026-07-09, doc 12 take 9). Pigmento é subtrativo: o pickup muda o matiz,
                // nunca a força — re-escala só pra CIMA (um pool mais escuro que a brocha segue
                // depositando mais escuro; t=0 e mixer-off ficam byte-idênticos: s=1).
                let mut mags = [0.0f32; 3];
                let mut ba_max = 0.0f32;
                let mut mix_max = 0.0f32;
                for (c, m) in mags.iter_mut().enumerate() {
                    let sa = mix.absorb[c] * inv; // unpremultiply → the carried paint's absorbance
                    let byte = (d.color[c].clamp(0.0, 1.0) * 255.0 + 0.5) as usize;
                    let ba = -lut.lnl[byte.min(255)]; // the brush pigment's absorbance
                    *m = ba + (sa - ba) * t; // subtractive mix (log-space lerp)
                    ba_max = ba_max.max(ba);
                    mix_max = mix_max.max(*m);
                }
                if mix_max > 1e-6 && mix_max < ba_max {
                    let s = ba_max / mix_max;
                    for m in &mut mags {
                        *m *= s;
                    }
                }
                for (out_c, &m) in col.iter_mut().zip(mags.iter()) {
                    *out_c = f32::from(lut.l2s_byte(lut.exp_mag(m))) / 255.0;
                }
            }
            // Deposit intensity for the COLOUR alpha (mirror of the coverage map's factors): a
            // depleted brush must not re-TINT a wet pool at full priority — in a wet session the
            // pool re-renders with whatever colour was deposited over it, so the tint has to track
            // the actual pigment on the brush (fresh reserve OR carried pickup), not just the mix
            // weight `t`.
            let depl =
                deplete_fresh(mix.travel, d.radius_px, charge).max(t * reservoir_pigment(&mix));
            // [wet-diag v2] TEMP (Enio 2026-07-09): salto de pickup entre dabs consecutivos =
            // fronteira de poça (MIX-1). Prova/refuta o degrau duro do Charge<1 no app real.
            if (t - diag_t_prev).abs() > (0.3 * pickup).max(0.05) {
                eprintln!(
                    "[wet-diag] MIX1 dab=({:.0},{:.0}) travel={:.0} sw={:.3} w={:.3} \
                     t={:.3}<-{:.3} fresh={:.3} pig={:.3} depl={:.3}",
                    d.center[0],
                    d.center[1],
                    mix.travel,
                    sw,
                    mix.w,
                    t,
                    diag_t_prev,
                    deplete_fresh(mix.travel, d.radius_px, charge),
                    reservoir_pigment(&mix),
                    depl,
                );
            }
            diag_t_prev = t;
            out.push((col, t, depl));
        }
        self.paint.wet_mix = mix;
        out
    }

    /// The per-dab pigment-reserve factors (`fresh ∨ carry`, `0..1`) for the COVERAGE pass's
    /// per-pixel depletion map — REPLAYS the travel chain from the stored `wet_mix` without writing
    /// it back (the colour pass advances it once per batch — the rng replay discipline).
    /// `None` when the mixer is off (no depletion — byte-identical default).
    pub(super) fn wet_mix_depletion(&self, dabs: &[Dab]) -> Option<Vec<f32>> {
        if !self.wet_mixer_active() {
            return None;
        }
        let charge = self.paint.brush.wet_charge.clamp(0.0, 1.0);
        let pickup = (1.0 - charge).clamp(0.0, 1.0);
        let mut travel = self.paint.wet_mix.travel;
        let mut last = self.paint.wet_mix.last_pos;
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
            out.push(deplete_fresh(travel, d.radius_px, charge).max(carry));
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
