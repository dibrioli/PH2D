//! Which stroke METHODS the Stroke section's Method dropdown offers — the pure decision,
//! split from `paint_stroke.rs` (at the panel file-LOC cap) so the law is testable from
//! `tests/` (a menu that silently offers a method the route refuses is a lying menu).
//!
//! The wire values are Blender's `eBrushStrokeType` discriminants (`StrokeMethod::to_u8`).

use ph2d_tool_painter::BrushSettings;

/// The methods offered for `brush`, in menu order.
///
/// - **Clone**: the incremental methods + Anchored (a stationary growing stamp clones fine
///   without motion, unlike Smear); the fill / editable-curve methods don't produce the
///   per-move dab chain it processes along.
/// - **Smear / Blur** (`paints_no_color`): the incremental methods only.
/// - **Wet Paint**: the FULL list (doc 21 — deposit-at-commit): every method authors a
///   flat static preview and the fluid receives the final dab list once, at commit
///   (pen-up / Enter), so no method is incompatible. (The W3 narrowing rested on a
///   refuted premise — [[feedback_a_nonidempotent_target_excludes_nothing_split_authoring_from_deposit]].)
/// - Otherwise: the full list.
#[must_use]
pub fn offered_stroke_methods(brush: Option<&BrushSettings>) -> &'static [u8] {
    if brush.is_some_and(|b| b.is_clone) {
        &[0, 1, 3, 4]
    } else if brush.is_some_and(|b| b.paints_no_color()) {
        &[0, 1, 3]
    } else {
        &[0, 4, 3, 1, 2, 5, 6, 7, 8, 9]
    }
}
