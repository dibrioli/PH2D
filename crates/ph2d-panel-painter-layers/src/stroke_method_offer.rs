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
/// - **Wet Paint armed**: the incremental methods only (`StrokeMethod::is_incremental`) —
///   the fluid deposit is not idempotent, so the re-stamping methods and shape editors
///   would pile paint per preview frame; the wet route refuses them, and entering the mode
///   coerces a non-incremental slot to Space (`set_paint_tool_mode`) — menu and slot must
///   agree. Ordered AFTER the clone/smear arms: those modes' own narrowings win while
///   their tool is held (the arm only owns the BRUSH).
/// - Otherwise: the full list.
#[must_use]
pub fn offered_stroke_methods(brush: Option<&BrushSettings>) -> &'static [u8] {
    if brush.is_some_and(|b| b.is_clone) {
        &[0, 1, 3, 4]
    } else if brush.is_some_and(|b| b.paints_no_color()) {
        &[0, 1, 3]
    } else if brush.is_some_and(|b| b.wetpaint) {
        &[0, 1, 3]
    } else {
        &[0, 4, 3, 1, 2, 5, 6, 7, 8, 9]
    }
}
