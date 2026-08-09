//! Which stroke METHODS the Stroke section's Method dropdown offers — the panel's view of a law that
//! lives in the engine, split from `paint_stroke.rs` (at the panel file-LOC cap) so it is testable
//! from `tests/` (a menu that silently offers a method the route refuses is a lying menu).
//!
//! The wire values are Blender's `eBrushStrokeType` discriminants (`StrokeMethod::to_u8`).

use ph2d_tool_painter::{BrushSettings, MethodOffer, offered_methods};

/// The methods offered for `brush`, in menu order.
///
/// ⚠️ **Pure delegation, on purpose.** The list is the engine's
/// [`offered_methods`] — the same one the tool asks when the
/// paint medium changes, so a method the menu stops offering cannot stay armed behind it. This
/// function's job is the translation from the panel's snapshot to that law's three named facts, and
/// nothing else; re-stating the branches here is how the menu and the tool start to disagree.
#[must_use]
pub fn offered_stroke_methods(brush: Option<&BrushSettings>) -> &'static [u8] {
    offered_methods(MethodOffer {
        is_clone: brush.is_some_and(|b| b.is_clone),
        paints_no_color: brush.is_some_and(|b| b.paints_no_color()),
        digital: brush.is_some_and(|b| b.media == DIGITAL),
    })
}

/// O discriminante de `PaintMedia::Digital`. Nomeado porque a comparação é uma REGRA de produto, e um
/// `== 0` solto num `if` não diz qual meio é o zero.
const DIGITAL: u8 = 0;
