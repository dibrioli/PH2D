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
#[must_use]
pub fn retime_with_speed_handle(
    k0: &KeyView,
    k1: &KeyView,
    which: u8,
    t: f64,
    v: f64,
) -> Option<Retime> {
    let r = rhythm_reference(k0, k1)?;
    let w = k0.retime.unwrap_or(Retime::LINEAR);
    let u = (t - k0.t_seconds) / (k1.t_seconds - k0.t_seconds);
    // Monotone: the rate never goes below zero (a speed on the far side of the zero line pauses).
    let s = (v / r).max(0.0);
    // The arm `a` (influence) and the rate `s` meet `y = s·a`, and `y ≤ 1` keeps `W` monotone. Past
    // that the SPEED is honoured and the arm shortens to `1/s`: clamping `y` instead left the tip
    // stuck while the pointer went on — 254/s away from it (auditoria do dono 15/11).
    let braco = |a: f64| a.min(1.0 / s.max(1.0)).max(MIN_INFLUENCE);
    Some(if which == 0 {
        let x1 = braco(u.clamp(MIN_INFLUENCE, 1.0)); // CLAMP-OK: const bounds, min<max
        Retime::new(x1 as f32, (s * x1) as f32, w.x2, w.y2)
    } else {
        let a = braco((1.0 - u).clamp(MIN_INFLUENCE, 1.0)); // CLAMP-OK: const bounds, min<max
        Retime::new(w.x1, w.y1, (1.0 - a) as f32, (1.0 - s * a) as f32)
    })
}

#[cfg(test)]
#[path = "retime_tests.rs"]
mod tests;
