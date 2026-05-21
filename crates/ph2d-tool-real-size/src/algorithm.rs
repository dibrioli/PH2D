//! Real Size — pure scale-reset logic.
//!
//! `std`-only, no editor/ECS coupling: operates on a plain `[f32; 2]`
//! (`[scale_x, scale_y]`) so this crate stays a leaf island. The desktop
//! shell reads the selected sprite's `Transform.scale` into this array,
//! calls [`real_size_scale`], and writes the result back.
//!
//! ### Contract the Implementer fills
//!
//! Reset each axis to unit scale **preserving the flip sign** — the
//! legacy `toggle_realsize` behaviour:
//!
//! ```text
//!   scale_x = if scale_x < 0.0 { -1.0 } else { 1.0 }
//!   scale_y = if scale_y < 0.0 { -1.0 } else { 1.0 }
//! ```
//!
//! Edge cases to pin with tests:
//! - already 1:1 (`[1.0, 1.0]`) → unchanged (caller skips the undo entry).
//! - flipped (`[-2.5, 1.0]`) → `[-1.0, 1.0]` (sign kept, magnitude reset).
//! - zero / NaN scale → decide + document a defined result (legacy used
//!   `Math.sign(x) || 1`, i.e. a `0` or `NaN` sign falls back to `+1`).

/// Reset a sprite's `[scale_x, scale_y]` to 1:1, preserving the flip sign
/// of each axis.
///
/// SCAFFOLD STUB — the Implementer fills the body per the module-level
/// contract and adds the unit tests. Returns the input unchanged for now
/// so the (future) shell drain wiring compiles and is inert.
pub fn real_size_scale(scale: [f32; 2]) -> [f32; 2] {
    // TODO(impl): reset each axis to ±1 preserving flip sign; handle the
    // 0 / NaN sign fallback to +1; add unit tests for the edge cases
    // listed in the module doc.
    scale
}
