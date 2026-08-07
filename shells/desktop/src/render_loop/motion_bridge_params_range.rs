//! **A FAIXA de uma row de param** — um `#[path]` filho de `motion_bridge_params.rs`,
//! cortado pelo cap de 600 LOC da shell (HR-18). `super` é o pai, então o
//! `params_channel` dele está no escopo.
//!
//! As duas funções respondem à MESMA pergunta por dois lados: uma alarga a faixa para o
//! widget nunca mentir sobre um valor que já existe, a outra alarga para o que um CANAL
//! significa. Juntas porque uma faixa que sai de dois lugares diverge.

use super::params_channel;

/// A behaviour's magnitude param needs a **channel-aware widget range**, not just
/// a channel-aware value: a `ParamUiHint`'s range is static, and the behaviours'
/// were authored for position (`±10` world units). On the Rotation channel the
/// same param means DEGREES, where `±10` is a barely-visible tilt — and, worse, a
/// range that cannot even represent the `±90` preset: the slider would saturate,
/// display `-10`, and overwrite the doc with `-10` on the first touch.
///
/// Widen `[min, max]` so it **contains** `value` — the invariant every row must
/// satisfy before it reaches the panel.
///
/// A `ParamUiHint`'s range is a suggestion, not a constraint: `Graph::set_param`
/// never clamps, so a preset, an undo, or a loaded document can hold a value
/// outside it. A row whose range does not contain its value is a *lying widget* —
/// `normalized_track` clamps it to the track end, the panel PAINTS the clamped
/// number instead of the real one, and the first touch emits that clamped number
/// straight back into the doc, silently destroying the authored value. (That is
/// exactly the bug the Enio caught with Stagger on the Rotation channel.)
///
/// Containing the value costs a degraded slider span in the pathological case and
/// self-heals the moment the value returns inside the hint's range — a cheap
/// price for a widget that can never lie or destroy.
pub(super) fn contain(min: f32, max: f32, value: f32) -> (f32, f32) {
    (min.min(value), max.max(value))
}

/// Returns `(min, max, step)` to use instead of the hint's, or `None` to keep it.
/// Only Rotation needs widening (Position / Size are already world-unit-scaled).
pub(super) fn channel_range_override(
    type_name: &str,
    param: &str,
    channel: i32,
) -> Option<(f32, f32, f32)> {
    if channel != params_channel::CHANNEL_ROTATION {
        return None;
    }
    // A full turn each way, dialled in whole degrees.
    const TURN: f32 = 360.0;
    match (type_name, param) {
        ("motion.stagger", "min" | "max") => Some((-TURN, TURN, 1.0)),
        ("motion.oscillator", "offset") => Some((-TURN, TURN, 1.0)),
        ("motion.oscillator" | "motion.wiggle", "amplitude") => Some((0.0, TURN, 1.0)),
        _ => None,
    }
}
