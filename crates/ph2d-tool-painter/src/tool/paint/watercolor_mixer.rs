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
//! **No self-feeding** (the retired reservoir's failure mode, [[project-wash-undo…]] lesson): the
//! pickup reads `watercolor_base` (frozen at pen-down) over the frozen backdrop, NEVER the live
//! canvas. **No cadence-binding**: the resample clock is Pull/distance-driven (per dab), not per
//! frame. The visible signature is the CARRY (Pull) + the pickup fraction (Charge) — not a weak
//! colour tint. Deterministic (HR-5): only sums/mults over integer-indexed samples.

use super::*;

/// The mixer's per-stroke reservoir: unpremultiplied picked-up colour (straight sRGB `0..1`), a
/// presence-weighted confidence `w ∈ [0, 1]` (how much real paint it holds), and the Pull resample
/// `w == 0` (fresh / over bare ground) ⇒ the deposit is pure brush colour.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct WetMix {
    rgb: [f32; 3],
    w: f32,
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
    pub(super) fn wet_mix_dab_colors(&mut self, dabs: &[Dab]) -> Vec<([f32; 3], f32)> {
        if !self.wet_mixer_active() {
            return dabs.iter().map(|d| (d.color, 1.0)).collect();
        }
        let (fw, fh) = self.source_size;
        let (fw, fh) = (fw as usize, fh as usize);
        let base = self.paint.watercolor_base.as_ref().map(Arc::clone);
        let backdrop = self.paint.wet_backdrop.as_ref().map(Arc::clone);
        let (Some(base), Some(backdrop)) = (base, backdrop) else {
            return dabs.iter().map(|d| (d.color, 1.0)).collect();
        };
        if base.len() != fw * fh * 4 || backdrop.len() != fw * fh * 4 || fw == 0 || fh == 0 {
            return dabs.iter().map(|d| (d.color, 1.0)).collect();
        }
        let pickup = (1.0 - self.paint.brush.wet_charge).clamp(0.0, 1.0);
        let p = self.paint.brush.wet_pull.clamp(0.0, 1.0);
        // Unload retention rises with Pull (`p·(2−p)` concave, transcendental-free, HR-5 safe): Pull 0
        // = a short baseline carry (symmetric exit bleed), Pull → 1 = a long smudge. The LOAD rate is
        // fixed + fast (RETAIN_LOAD) so the pickup is always strong.
        let unload =
            (RETAIN_UNLOAD_MIN + (0.98 - RETAIN_UNLOAD_MIN) * p * (2.0 - p)).clamp(0.0, 0.98);
        let mut out = Vec::with_capacity(dabs.len());
        let mut mix = self.paint.wet_mix;
        for d in dabs {
            // ── 1. Resample the frozen surface under the disc every dab (star), then ASYMMETRIC
            //    running-average in PREMULTIPLIED space: LOAD fast toward more paint (strong pickup
            //    on entry), UNLOAD slow toward bare ground (the carried colour lingers past the pool
            //    → the EXIT bleed mirrors the ENTRY). Bare ground (`sw = 0`) only DEPLETES the load
            //    (× update); it never pulls the carried hue toward the ground (premul), so `rgb / w`
            //    stays the picked-up colour. ──
            let (srgb, sw) = sample_surface(&base, &backdrop, fw, fh, d.center, d.radius_px);
            let update = if sw >= mix.w { RETAIN_LOAD } else { unload };
            for (channel, &s) in mix.rgb.iter_mut().zip(srgb.iter()) {
                *channel = update * *channel + (1.0 - update) * sw * s;
            }
            mix.w = update * mix.w + (1.0 - update) * sw;
            // ── 2. Deposit: blend the brush colour toward the (unpremultiplied) reservoir colour by
            //    the priority `t = pickup × load`. Load `w` decays downstream ⇒ the carried colour
            //    fades with distance, AND `t` (the deposit priority) decays with it — so a fading
            //    exit dab can't overwrite a stronger in-pool deposit (symmetric crossing). ──
            let t = (pickup * mix.w).clamp(0.0, 1.0);
            let mut col = d.color;
            if mix.w > 1e-4 {
                let inv = 1.0 / mix.w;
                for ((out_c, &m), &base_c) in col.iter_mut().zip(mix.rgb.iter()).zip(d.color.iter())
                {
                    let sc = m * inv; // unpremultiply
                    *out_c = base_c + (sc - base_c) * t;
                }
            }
            out.push((col, t));
        }
        self.paint.wet_mix = mix;
        out
    }
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
