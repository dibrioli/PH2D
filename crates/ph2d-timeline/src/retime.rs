//! ⭐ **Retime por trecho** (plano 18, dono 15/11): o RITMO de um trecho editado na vista de velocidade
//! sem destruir o easing. A lei mora em [`ph2d_anim::Retime`] (`valor(u) = lerp(v0, v1, E(W(u)))`);
//! aqui ficam o comando e a matemática da vista.
//!
//! **A caixa do tempo** (pesquisa 19, dono 15/11 «vamos tentar A»): em Retime a vista desenha o
//! próprio `W` — de canto a canto de uma caixa do tamanho do trecho, como a curva *Retime Frame* do
//! Resolve e o editor de `cubic-bezier` do CSS — e cada alça é o ponto `(x1, y1)`/`(x2, y2)` dentro
//! dela. A caixa É o limite da monotonia (`[0,1]⁴`): o gesto além da borda fica na borda, debaixo do
//! rato. Medido e recusado: a alça AE sobre `ref · W'` (o limite era a hipérbole `s = 1/a`, invisível,
//! e a ponta escorregava — `docs/Timeline/19_pesquisa_retime.md` §5).
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

/// `ref = (v1 − v0) / span` of the segment, or `None` for a zero-span or a still segment (a still
/// segment plays the same value under any rhythm: it shows no time box).
#[must_use]
pub fn rhythm_reference(k0: &KeyView, k1: &KeyView) -> Option<f64> {
    let span = k1.t_seconds - k0.t_seconds;
    let dv = f64::from(k1.value) - f64::from(k0.value);
    (span > 0.0 && dv.abs() > f64::from(f32::EPSILON)).then(|| dv / span)
}

/// The time-box handle TIP of one side (`which`: `0` = start, `1` = end), in the segment's unit
/// square: `(u, W)` = `(x1, y1)` or `(x2, y2)`. `None` on a segment with no rhythm.
#[must_use]
pub fn time_handle_tip(k0: &KeyView, k1: &KeyView, which: u8) -> Option<(f64, f64)> {
    rhythm_reference(k0, k1)?;
    let w = k0.retime.unwrap_or(Retime::LINEAR);
    let (x, y) = if which == 0 {
        (w.x1, w.y1)
    } else {
        (w.x2, w.y2)
    };
    Some((f64::from(x), f64::from(y)))
}

/// The warp one time-box drag writes: the pointer `(u, W)` in the unit square becomes that side's
/// handle; beyond the box it goes to the box's nearest point — per axis, so the same at any pixel
/// scale, and a function of the pointer alone (never of the path). The other side keeps its handle.
#[must_use]
pub fn retime_with_time_handle(
    k0: &KeyView,
    k1: &KeyView,
    which: u8,
    (u, y): (f64, f64),
) -> Option<Retime> {
    rhythm_reference(k0, k1)?;
    let w = k0.retime.unwrap_or(Retime::LINEAR);
    let na_caixa = |v: f64| {
        if v.is_finite() {
            v.clamp(0.0, 1.0) as f32 // CLAMP-OK: the unit square, NaN handled
        } else {
            0.0
        }
    };
    let (u, y) = (na_caixa(u), na_caixa(y));
    Some(if which == 0 {
        Retime::new(u, y, w.x2, w.y2)
    } else {
        Retime::new(w.x1, w.y1, u, y)
    })
}

#[cfg(test)]
#[path = "retime_tests.rs"]
mod tests;
