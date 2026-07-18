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

/// The **bow-wave lobe**: how much of the rim at this offset lies AHEAD of the moving blade —
/// `cos²θ` for `cos θ > 0`, zero behind. `0` for a tap (no direction: a pressed stamp squeezes
/// radially, and [`lateral_weight`]'s `1` already covers the whole annulus).
///
/// This is the half of the physics the first build deliberately deferred (*"o traço desloca como
/// uma forma, não como uma lâmina — não há bow wave"*, plano §13.1): a blade pushes a wave AHEAD
/// of its tip. Banking forward was tried and reverted once (MUT S: the next dab painted over the
/// forward bank and half the paint ended up inside the channel) — what makes it sound now is that
/// the bite **re-takes the accumulated plane** ([`PushBite::plane`] rides into the take), so the
/// wave the last dab banked is picked up by the next and shoved on: it ROLLS with the tip and
/// rests at the stroke's frontier, which is where IMPaSTo (NPAR 2004, `v_p = −c∇p`) says the
/// displaced paint accumulates. The lateral wake still forms on its own: whatever the rolling wave
/// sheds sideways lands outside the swath, where no later dab ever re-bites it.
#[inline]
fn forward_weight(dx: f32, dy: f32, dir: Option<[f32; 2]>) -> f32 {
    let Some(u) = dir else { return 0.0 };
    let len2 = dx * dx + dy * dy;
    if len2 <= 1e-6 {
        return 0.0;
    }
    let along = dx * u[0] + dy * u[1];
    if along <= 0.0 {
        return 0.0;
    }
    (along * along) / len2
}

/// The share of each dab's displaced paint that travels with the BOW WAVE instead of shedding
/// into the lateral wake (the `forward_share` the deposit passes to [`bank_dab_push`]). Measured
/// on the zone probe: per-dab forward *banking* cannot make a wave — the next dab paints over it
/// and the envelope bite (a one-shot `ground × Δm` extraction) cannot re-transport it (0.4% ever
/// landed ahead; 16% fossilised inside the swath — MUT S, quantified). So the wave is carried as a
/// session SCALAR and painted as a lobe at the CURRENT tip ([`wave_lobe`]), exactly removable,
/// finally resting at the stroke's frontier — which is where IMPaSTo (NPAR 2004, `v_p = −c∇p`)
/// puts it. The Sculpt's Conserve passes `0.0`: all paint sheds laterally, byte-identical to its
/// approved drawing. // CLAMP-OK
pub const DEPOSIT_FORWARD_SHARE: f32 = 0.6;

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

/// The rim's INNER anchor, in normalised dab distance `t` — where the banked ridge begins.
///
/// It is where the brush's paint stops carrying a body ([`crate::height_film::body_edge_t`]), so
/// displaced paint piles up against the paint it was shoved by, not against the dab's geometric
/// circle. Anchoring at the circle (`t = 1`) left a hard collar of banked paint a silhouette's-width
/// of bare canvas out from the soft paint (Enio's smoke, 2026-07-15 — see [`crate::height_film::body_edge_t`]).
///
/// A **Shape** silhouette (Image or procedural) has no radial body edge — a stamp keeps its hard edge
/// by definition — so the anchor stays at the geometric rim there, byte-identical to a stamped dab.
///
/// **One door for BOTH kernels** ([`bank_dab_push`] and [`wave_lobe`]) and BOTH callers (the deposit's
/// Push and the Sculpt's Conserve, which share this motor): a second copy of "where does the rim
/// start" would let the lateral bank and the forward lobe be born on different circles. Hoist it once
/// per stroke — the falloff, hardness and Shape state do not change mid-stroke.
#[must_use]
pub fn rim_t0(spec: &crate::BrushSpec, has_shape: bool) -> f32 {
    if has_shape {
        1.0
    } else {
        crate::height_film::body_edge_t(spec)
    }
}

/// The rim's lift at one texel: zero inside the body (`t ≤ t0`), else the C¹ bank profile
/// ([`push_rim_weight`]) across a reach that starts AT the body edge `t0`.
///
/// Shared by [`bank_dab_push`] and [`wave_lobe`] so the lateral bank and the forward lobe cannot
/// anchor on different circles. `inv_reach = radius / reach`, so `u = (t − t0) · radius / reach` is
/// the rim's width measured in PIXELS from `t0` — a **constant-width** bank however far in the body
/// edge sits (receding the anchor without receding the reach would stretch the bank into a spike, and
/// a narrow bank is the spike that MUT-history's `push_reach_px` fix already paid for). `t0 = 1`
/// reproduces the pre-2026-07-15 rim bit-for-bit.
#[inline]
#[must_use]
fn rim_lift(t: f32, t0: f32, inv_reach: f32) -> f32 {
    if t <= t0 {
        return 0.0;
    }
    push_rim_weight((t - t0) * inv_reach)
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
/// `forward_share` splits the displaced volume: `share × displaced` is RETURNED to the caller
/// (the bow wave's cargo, painted by [`wave_lobe`] at the tip), the rest banks laterally here.
/// `0.0` reproduces the purely lateral banking bit for bit (the Conserve's approved drawing). A
/// directionless dab (a tap) always banks everything radially — a pressed stamp has no "ahead".
/// `t0` is the rim's inner anchor ([`rim_t0`]): the ridge rises from where the paint's body ends,
/// not from the dab's geometric rim. Returns the banked rect and the carried volume.
#[allow(clippy::too_many_arguments)]
pub fn bank_dab_push(
    plane: &mut [f32],
    paint: &[f32],
    scratch: &mut Vec<f32>,
    width: u32,
    height: u32,
    dab: &HeightDab<'_>,
    t0: f32,
    displaced: f32,
    forward_share: f32,
) -> (Option<crate::dab::DirtyRect>, f32) {
    let n = (width as usize) * (height as usize);
    if plane.len() < n || paint.len() < n || width == 0 || height == 0 {
        return (None, 0.0);
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
        return (None, 0.0);
    }
    let inv_radius = 1.0 / radius;
    let inv_reach = radius / reach; // u = (t - 1) * radius / reach
    let sweep = sweep_axis(dab);
    let dir = sweep.map(|(u, _)| u);
    // The wave's cargo leaves FIRST: only a moving dab feeds it (a tap sheds everything radially).
    let carried = if dir.is_some() {
        displaced * forward_share.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let displaced = displaced - carried;
    if displaced == 0.0 {
        return (None, carried);
    }

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
            // The ridge rises from the paint's body edge `t0`, not the geometric rim: `rim_lift` is
            // zero inside the body (`t ≤ t0`) and past the reach, so the bank butts against the paint
            // instead of standing a hard circle out at `t = 1` (Enio's smoke, 2026-07-15).
            let k = rim_lift(t, t0, inv_reach);
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
        // The rim is entirely off-canvas: the paint really was shoved off the edge.
        return (None, carried);
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
    (
        Some(crate::dab::DirtyRect {
            x: x0 as u32,
            y: y0 as u32,
            w: (x1 - x0) as u32,
            h: (y1 - y0) as u32,
        }),
        carried,
    )
}

/// Paint (or exactly un-paint) the **bow wave** — the carried volume standing as a lobe AHEAD of
/// the tip. Weights: the rim profile ([`push_rim_weight`]) × the forward lobe
/// ([`forward_weight`]), preferring un-painted texels exactly like the lateral bank. `sign` is
/// `+1.0` to lay the lobe and `-1.0` to take it back — the caller re-lays it at every dab (remove
/// at the OLD tip with the OLD volume, lay at the new tip with the new volume), which is what
/// makes the wave visibly TRAVEL with the brush and finally REST at the stroke's frontier at
/// pen-up. Writes through the same single book (`plane`), so `Σ plane = 0` needs no second ledger:
/// lay and un-lay are exact negations of each other.
///
/// A directionless dab has no "ahead" — the lobe is refused (`None`) and the caller keeps the
/// cargo for the next dab that moves.
#[allow(clippy::too_many_arguments)]
pub fn wave_lobe(
    plane: &mut [f32],
    paint: &[f32],
    scratch: &mut Vec<f32>,
    width: u32,
    height: u32,
    dab: &HeightDab<'_>,
    t0: f32,
    volume: f32,
    sign: f32,
) -> Option<crate::dab::DirtyRect> {
    let n = (width as usize) * (height as usize);
    if plane.len() < n || paint.len() < n || width == 0 || height == 0 || volume <= 0.0 {
        return None;
    }
    let sweep = sweep_axis(dab);
    // `sweep_axis` points BACK toward the previous dab (it is the sweep's axis); the wave stands
    // AHEAD of the travel, which is its negation.
    let dir = sweep.map(|(u, _)| [-u[0], -u[1]])?;
    let radius = dab.radius.max(0.5);
    let reach = push_reach_px(radius);
    let (cx, cy) = (dab.center[0], dab.center[1]);
    let span = radius + reach;
    let x0 = (cx - span).floor().max(0.0) as i64;
    let y0 = (cy - span).floor().max(0.0) as i64;
    let x1 = ((cx + span).ceil() as i64 + 1).min(width as i64);
    let y1 = ((cy + span).ceil() as i64 + 1).min(height as i64);
    if x0 >= x1 || y0 >= y1 {
        return None;
    }
    let inv_radius = 1.0 / radius;
    let inv_reach = radius / reach;
    let (bw, bh) = ((x1 - x0) as usize, (y1 - y0) as usize);
    scratch.clear();
    scratch.resize(bw * bh, 0.0);
    let mut rim_weight = 0.0f32;
    let mut lobe_weight = 0.0f32;
    for by in 0..bh {
        let py = y0 + by as i64;
        let dy = (py as f32 + 0.5) - cy;
        for bx in 0..bw {
            let px = x0 + bx as i64;
            let dx = (px as f32 + 0.5) - cx;
            // The lobe stands on the dab's own annulus (no sweep: the wave lives at the TIP), rising
            // from the paint's body edge `t0` — the SAME anchor the lateral bank uses, so the wave and
            // the wake are born on one circle, not two.
            let t = dab.footprint.falloff_t(dx * inv_radius, dy * inv_radius);
            let k = rim_lift(t, t0, inv_reach) * forward_weight(dx, dy, Some(dir));
            if k <= 0.0 {
                continue;
            }
            let i = (py as usize) * (width as usize) + px as usize;
            scratch[by * bw + bx] = k;
            rim_weight += k * (1.0 - paint[i].clamp(0.0, 1.0));
            lobe_weight += k;
        }
    }
    let (weight, use_clear) = if rim_weight > 0.0 {
        (rim_weight, true)
    } else if lobe_weight > 0.0 {
        (lobe_weight, false)
    } else {
        return None; // the whole lobe is off-canvas
    };
    let scale = sign * volume / weight;
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
