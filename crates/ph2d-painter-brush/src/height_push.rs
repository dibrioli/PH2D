//! **Impasto — volume conservation.** The paint the brush shoves out of its way, and where it goes.
//!
//! Sibling of [`crate::height`], which is the DEPOSIT: what the brush lays down. This is the other half of
//! the physics, and the one that separates paint from a bump map — real paint cannot interpenetrate, so a
//! brush dragged through it ploughs a channel and banks the displaced material into a ridge along the
//! stroke's edges.

use crate::height::{HeightDab, sweep_axis, sweep_len, sweep_residual};

/// The widest rim the displacement can throw paint into, in pixels — the pad the commit's window needs.
/// A cap, not the reach: the reach itself is [`push_reach_px`]. // CLAMP-OK
pub const PUSH_REACH_MAX_PX: f32 = 24.0;

/// How far THIS brush shoves paint beyond its own silhouette, in pixels.
///
/// It scales with the brush and is capped. A wide knife banks a wide ridge; a liner banks a thin one —
/// but the window the commit works in is padded by [`PUSH_REACH_MAX_PX`], and a reach that could grow
/// without bound would make that window a function of the brush instead of the stroke (§11).
///
/// The fraction is **0.8 of the radius**, and it was 0.35, and that was the whole of Enio's *"a tinta
/// deslocada ficou com bordas duras"*. A bank four texels wide holding the bite of a twelve-pixel brush is
/// not a bank, it is a **spike** — and the light, which reads the derivative of the height, draws a spike
/// as a hard bright line. Volume is conserved either way; what changes is whether the paint reads as paint.
/// // CLAMP-OK
#[inline]
#[must_use]
pub fn push_reach_px(radius: f32) -> f32 {
    (radius * 0.8).clamp(6.0, PUSH_REACH_MAX_PX)
}

/// How much of the rim at a given offset is **beside** the brush rather than **ahead of** it — `|sin θ|`
/// between the offset and the direction of travel.
///
/// A moving blade does not leave paint standing in front of itself: it runs it over on the next dab and
/// shoves it further. Only what goes out **sideways** survives, and that is why the ridge stands along the
/// EDGES of a stroke. Banking radially — the first build — left half the displaced paint inside the
/// channel the stroke was about to cut (measured: the channel came out at 55% of the slab instead of 0),
/// because each dab banked ahead of itself and the next dab simply painted over it.
///
/// `1` when there is no direction (a tap: the whole annulus is "beside").
///
/// Returns `sin²θ`, not `|sin θ|` — a tighter lateral lobe, and no square root in a loop that runs over
/// every texel of every dab's rim.
#[inline]
fn lateral_weight(dx: f32, dy: f32, dir: Option<[f32; 2]>) -> f32 {
    let Some(u) = dir else { return 1.0 };
    let len2 = dx * dx + dy * dy;
    if len2 <= 1e-6 {
        return 0.0;
    }
    let cross = dx * u[1] - dy * u[0];
    (cross * cross) / len2
}

/// The rim's profile: how the displaced paint banks up outside the cut, as a function of `u` — the
/// distance beyond the silhouette, normalised to the reach (`0` at the edge of the cut, `1` at the far
/// side of the rim).
///
/// Smooth at both ends, with zero slope at each — replacing the first build's rim, which was
/// `blur(footprint) − footprint` with a **box blur**: a box blur of a step is a *linear ramp*, continuous
/// but with a **discontinuous derivative**, and the light reads the derivative of the height.
///
/// **Stated honestly, because the gate says so:** swapping this back for a kinked (triangle) profile does
/// NOT turn `impasto_push_banks_a_smooth_ridge_with_no_crease_scored_along_it` red. It cannot — the bank is
/// accumulated **dab by dab**, and neighbouring dabs' profiles overlap and average any kink away. What the
/// old build actually got wrong was doing the rim as ONE global blur of the whole footprint, where there is
/// no superposition to hide behind, and — measurably — banking it into too narrow a strip
/// ([`push_reach_px`]). The `C¹` profile here is the *right shape to reach for*, not a fix the tests defend;
/// the width is, and that one has its red.
///
/// (Transcendental-free, HR-5: two smoothsteps.)
#[inline]
#[must_use]
pub fn push_rim_weight(u: f32) -> f32 {
    if !(0.0..=1.0).contains(&u) {
        return 0.0;
    }
    let s = |x: f32| x * x * (3.0 - 2.0 * x);
    s((u * 2.0).min(1.0)) * s(((1.0 - u) * 2.0).min(1.0))
}

/// The bite, carried into [`crate::height::accumulate_dab_height`]'s own walk — the paint the brush is
/// taking out from under itself, and the running total of it.
///
/// It rides along in that loop rather than living in a kernel of its own, and the reason is a number:
/// evaluating the dab's silhouette twice per texel (once to lay paint, once to take it) put the whole
/// impasto cost at **5.0 ms/move**, past its 4 ms budget, on every stroke — whether the artist had the
/// knob up or not.
pub struct PushBite<'a> {
    /// The layer's relief as the stroke FOUND it — read, never written. A brush shoves the paint it
    /// finds, not the paint it is laying.
    pub ground: &'a [f32],
    /// The stroke's displacement field at `Push = 1`: negative where paint was taken, positive where it
    /// was banked, **summing to exactly zero**.
    pub plane: &'a mut [f32],
    /// How much this dab took. [`bank_dab_push`] puts precisely this much back.
    pub displaced: f32,
}

/// **Volume conservation, one dab at a time** — the other half of [`PushBite`]: the paint the brush took
/// has to GO somewhere, and it banks up as a ridge just outside the cut.
///
/// Call it **after** [`crate::height::accumulate_dab_height`] has taken the bite, with the volume that
/// walk reported. It touches only the RIM (`t > 1`) — the silhouette was already walked, and walking it
/// again is what made the first build too slow to ship.
///
/// It accumulates into `plane` — the stroke's displacement field **at `push = 1`**. That is what keeps the
/// knob alive: the whole displacement is *linear* in Push, so the committed relief is just
/// `ground + push·R₁`, and the artist can dial Push after the stroke and watch the ridge grow, exactly
/// like every other knob in the Body card. And because each dab banks into its OWN rim — locally,
/// `O(dab)` — the ridge appears **under the brush, as it moves**, instead of at pen-up (the first build
/// derived the displacement from the whole footprint at commit, which is idempotent and live but *only
/// exists once the stroke is over*: Enio, 2026-07-12, *"precisa ser em tempo real"*).
///
/// Conservation is exact **per dab**: what the bite took is normalised into what this banks, so paint
/// shoved against the canvas edge lands in whatever rim is left instead of vanishing.
pub fn bank_dab_push(
    plane: &mut [f32],
    paint: &[f32],
    scratch: &mut Vec<f32>,
    width: u32,
    height: u32,
    dab: &HeightDab<'_>,
    displaced: f32,
) -> Option<crate::dab::DirtyRect> {
    let n = (width as usize) * (height as usize);
    if plane.len() < n || paint.len() < n || width == 0 || height == 0 {
        return None;
    }
    let radius = dab.radius.max(0.5);
    let reach = push_reach_px(radius);
    let (cx, cy) = (dab.center[0], dab.center[1]);
    let span = radius + reach + sweep_len(dab);
    let x0 = (cx - span).floor().max(0.0) as i64;
    let y0 = (cy - span).floor().max(0.0) as i64;
    let x1 = ((cx + span).ceil() as i64 + 1).min(width as i64);
    let y1 = ((cy + span).ceil() as i64 + 1).min(height as i64);
    if x0 >= x1 || y0 >= y1 || displaced == 0.0 {
        return None;
    }
    let inv_radius = 1.0 / radius;
    let inv_reach = radius / reach; // u = (t - 1) * radius / reach
    let sweep = sweep_axis(dab);
    let dir = sweep.map(|(u, _)| u);

    // The weight of the rim at one texel: its profile across the bank, TIMES how far to the SIDE of the
    // brush it lies (a blade leaves nothing standing in front of itself — it runs it over on the next dab),
    // TIMES how much of it the stroke has not already painted over (banking under the swath you just laid
    // is banking into your own channel). All three are needed, and each was a bug before it was a factor.
    // Prefer to bank OUTSIDE the stroke's own swath. But a brush scrubbing deep inside its own path can
    // find that every texel of its rim is already painted — and then there is nowhere clear to put the
    // paint. Banking it into the swath is worse than ideal; **losing** it is not an option at all, because
    // the one thing this whole feature asserts is that paint is never destroyed. So: fall back.
    // Prefer to bank OUTSIDE the stroke's own swath. But a brush scrubbing deep inside its own path can
    // find that every texel of its rim is already painted — and then there is nowhere clear to put the
    // paint. Banking it into the swath is worse than ideal; **losing** it is not an option at all, because
    // the one thing this whole feature asserts is that paint is never destroyed. So: fall back.
    //
    // The weights are CACHED as they are computed. The rim has to be walked twice — once to know its total
    // weight, once to divide the paint among it — and walking it twice meant a `falloff_t` and a
    // `lateral_weight` per texel per pass. The scratch buffer is owned by the caller and reused across
    // every dab of the stroke, so this allocates nothing.
    let (bw, bh) = ((x1 - x0) as usize, (y1 - y0) as usize);
    scratch.clear();
    scratch.resize(bw * bh, 0.0);
    let mut rim_weight = 0.0f32;
    let mut swath_weight = 0.0f32;
    for by in 0..bh {
        let py = y0 + by as i64;
        let dy = (py as f32 + 0.5) - cy;
        for bx in 0..bw {
            let px = x0 + bx as i64;
            let dx = (px as f32 + 0.5) - cx;
            let (rx, ry) = sweep_residual(dx, dy, sweep);
            let t = dab.footprint.falloff_t(rx * inv_radius, ry * inv_radius);
            if t <= 1.0 {
                continue; // the bite already landed, inside the deposit's own walk
            }
            let k = push_rim_weight((t - 1.0) * inv_reach);
            if k <= 0.0 {
                continue;
            }
            let i = (py as usize) * (width as usize) + px as usize;
            // The GEOMETRIC weight (profile × how far to the side); the swath factor is one cheap read
            // away, so only this is worth caching.
            let base = k * lateral_weight(dx, dy, dir);
            scratch[by * bw + bx] = base;
            rim_weight += base * (1.0 - paint[i].clamp(0.0, 1.0));
            swath_weight += base;
        }
    }
    let (weight, use_clear) = if rim_weight > 0.0 {
        (rim_weight, true)
    } else if swath_weight > 0.0 {
        (swath_weight, false)
    } else {
        return None; // the rim is entirely off-canvas: the paint really was shoved off the edge
    };
    let scale = displaced / weight;
    for by in 0..bh {
        let row = (y0 + by as i64) as usize * (width as usize);
        for bx in 0..bw {
            let base = scratch[by * bw + bx];
            if base <= 0.0 {
                continue;
            }
            let i = row + (x0 + bx as i64) as usize;
            let k = if use_clear {
                base * (1.0 - paint[i].clamp(0.0, 1.0))
            } else {
                base
            };
            if k > 0.0 {
                plane[i] += k * scale;
            }
        }
    }
    Some(crate::dab::DirtyRect {
        x: x0 as u32,
        y: y0 as u32,
        w: (x1 - x0) as u32,
        h: (y1 - y0) as u32,
    })
}
