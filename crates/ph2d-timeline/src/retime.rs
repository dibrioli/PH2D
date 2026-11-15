//! ⭐ **Retime por trecho** (plano 18, dono 15/11): o RITMO de um trecho editado na vista de velocidade
//! sem destruir o easing. A lei mora em [`ph2d_anim::Retime`] (`valor(u) = lerp(v0, v1, E(W(u)))`);
//! aqui ficam o comando e a matemática da vista.
//!
//! **A curva do ritmo** que a vista desenha (a segunda cor, com as alças) é `ref · W'(u)`, com
//! `ref = (v1 − v0) / span` — a velocidade que um movimento LINEAR teria com este ritmo: sem retime é
//! a recta horizontal da velocidade média, e cada alça é a do AE (comprimento = influência, altura =
//! velocidade na ponta). Um trecho parado (`v0 == v1`) não tem `ref`: não mostra alças de ritmo.

use ph2d_anim::{AnimTarget, KeyId, Retime};

use crate::intent_apply::edit;
use crate::snapshot::KeyView;
use crate::state::TimelineState;

/// The retime commands (one undo step each, or one per `BeginEdit`/`EndEdit` bracket).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RetimeIntent {
    /// Set (or, with `None`, reset) the rhythm of the segment leaving `key`.
    Set {
        /// Track target.
        target: AnimTarget,
        /// The key whose OUTGOING segment is retimed.
        key: KeyId,
        /// The new warp; `None` = as the easing plays («Reset timing»).
        retime: Option<Retime>,
    },
    /// «Reset timing» on every selected key.
    ResetSelected,
}

pub(crate) fn apply(state: &mut TimelineState, intent: RetimeIntent) {
    match intent {
        RetimeIntent::Set {
            target,
            key,
            retime,
        } => edit(state, |doc, _| {
            if let Some(track) = doc.active_clip_mut().track_mut(target) {
                track.set_retime(key, retime);
            }
        }),
        RetimeIntent::ResetSelected => edit(state, |doc, sel| {
            for sk in sel.keys() {
                if let Some(track) = doc.active_clip_mut().track_mut(sk.target) {
                    track.set_retime(sk.key, None);
                }
            }
        }),
    }
}

/// AE's minimum influence (0.1 %) — the same floor as the Override speed handle.
const MIN_INFLUENCE: f64 = 1.0e-3;

/// `ref = (v1 − v0) / span` of the segment, or `None` for a zero-span or a still segment.
#[must_use]
pub fn rhythm_reference(k0: &KeyView, k1: &KeyView) -> Option<f64> {
    let span = k1.t_seconds - k0.t_seconds;
    let dv = f64::from(k1.value) - f64::from(k0.value);
    (span > 0.0 && dv.abs() > f64::from(f32::EPSILON)).then(|| dv / span)
}

/// The rhythm curve `ref · W'(u)` at `t` seconds, inside the segment `k0 → k1`.
#[must_use]
pub fn segment_rhythm(k0: &KeyView, k1: &KeyView, t: f64) -> Option<f64> {
    let r = rhythm_reference(k0, k1)?;
    let u = (t - k0.t_seconds) / (k1.t_seconds - k0.t_seconds);
    let s = ph2d_anim::retime::warp_slope(k0.retime, u);
    s.is_finite().then_some(r * s)
}

/// The rhythm handle TIP of one side (`which`: `0` = start, `1` = end): `(time, speed)`.
#[must_use]
pub fn rhythm_handle_tip(k0: &KeyView, k1: &KeyView, which: u8) -> Option<(f64, f64)> {
    let r = rhythm_reference(k0, k1)?;
    let w = k0.retime.unwrap_or(Retime::LINEAR);
    let span = k1.t_seconds - k0.t_seconds;
    let (x, u) = if which == 0 { (w.x1, 0.0) } else { (w.x2, 1.0) };
    let s = w.slope(u);
    s.is_finite()
        .then(|| (k0.t_seconds + f64::from(x) * span, r * s))
}

/// The warp one rhythm-handle drag writes — the AE gesture on `ref · W'`: the pointer's `t` sets
/// that side's influence and `v` the speed at that end (`W'(0) = y1/x1`, `W'(1) = (1−y2)/(1−x2)`).
/// The other side keeps its handle. `None` on a segment with no rhythm (see [`rhythm_reference`]).
/// `px = (px per second, px per value-per-second)` of the view: where the pointer asks for more
/// than `W` can give, the tip goes to the NEAREST reachable point ON SCREEN.
#[must_use]
pub fn retime_with_speed_handle(
    k0: &KeyView,
    k1: &KeyView,
    which: u8,
    (t, v): (f64, f64),
    px: (f64, f64),
) -> Option<Retime> {
    let r = rhythm_reference(k0, k1)?;
    let w = k0.retime.unwrap_or(Retime::LINEAR);
    let span = k1.t_seconds - k0.t_seconds;
    let u = (t - k0.t_seconds) / span;
    // Monotone: the rate never goes below zero (a speed on the far side of the zero line pauses).
    let s = (v / r).max(0.0);
    let a = if which == 0 { u } else { 1.0 - u };
    let a = a.clamp(MIN_INFLUENCE, 1.0); // CLAMP-OK: const bounds, min<max
    let (a, s) = alcancavel((a, s), (span * px.0, r.abs() * px.1));
    Some(if which == 0 {
        Retime::new(a as f32, (s * a) as f32, w.x2, w.y2)
    } else {
        Retime::new(w.x1, w.y1, (1.0 - a) as f32, (1.0 - s * a) as f32)
    })
}

/// The arm `a` (influence) and the rate `s` meet `y = s·a`, and `y ≤ 1` is what keeps `W` monotone:
/// the reachable tips are `s·a ≤ 1`. A pointer beyond goes to the point of the edge `s = 1/a` NEAREST
/// to it in screen pixels (`k = px per unit of a, px per unit of s`). Measured on the owner's audit
/// (15/11): clamping `y` left the tip up to 254/s below the pointer, and shortening the arm at fixed
/// speed made it run LEFT while the pointer went right.
fn alcancavel((a, s): (f64, f64), (ka, ks): (f64, f64)) -> (f64, f64) {
    if s * a <= 1.0 || !(ka > 0.0 && ks > 0.0) {
        return (a, s.min(1.0 / a));
    }
    let custo = |b: f64| (ka * (b - a)).powi(2) + (ks * (1.0 / b - s)).powi(2);
    // A coarse scan in log space, then a golden-section polish around the best cell: the cost is
    // not unimodal over the whole edge, and both steps are deterministic.
    const N: usize = 128;
    let (lo, hi) = (MIN_INFLUENCE.ln(), a.ln());
    let em = |i: usize| (lo + (hi - lo) * i as f64 / N as f64).exp();
    let melhor = (0..=N)
        .min_by(|&i, &j| custo(em(i)).total_cmp(&custo(em(j))))
        .unwrap_or(N);
    let (mut x0, mut x1) = (em(melhor.saturating_sub(1)), em((melhor + 1).min(N)));
    let g = 0.5 * (5.0f64.sqrt() - 1.0);
    for _ in 0..60 {
        let (c, d) = (x1 - g * (x1 - x0), x0 + g * (x1 - x0));
        if custo(c) < custo(d) {
            x1 = d;
        } else {
            x0 = c;
        }
    }
    let b = 0.5 * (x0 + x1);
    (b, 1.0 / b)
}

#[cfg(test)]
#[path = "retime_tests.rs"]
mod tests;
