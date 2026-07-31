//! The passthrough a **bypassed** node cooks instead of running its op (H — the mute). Split from
//! `cook.rs` for the HR-18 LOC cap; declared there as a `#[path]` sibling, so `super` is `cook`.

use super::CookValue;

/// The outputs of a bypassed node: its primary input (port 0) passed straight to its primary
/// output (port 0), every other output `Empty`. `input[0] -> output[0]` is the Houdini/Nuke
/// convention; an unconnected input passes `Empty`, which reads as "nothing flows here", the same
/// as a disconnected node (a bypassed `force.wind` adds no force; a bypassed deformer leaves the
/// geometry it received untouched). The manifest's output ARITY is honoured so the caller's count
/// check holds for the passthrough exactly as for a real eval.
pub(super) fn bypass_outputs(inputs: &[CookValue], n_out: usize) -> Vec<CookValue> {
    (0..n_out)
        .map(|i| {
            if i == 0 {
                inputs.first().cloned().unwrap_or_default()
            } else {
                CookValue::Empty
            }
        })
        .collect()
}
