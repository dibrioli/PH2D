//! **Whether a stroke has a HEAD to taper** — the one question [`crate::taper`] still has to ask.
//!
//! A tapered dab needs one distance: how far it is from the stroke's start. That is free —
//! [`crate::Dab::arc_len`] already carries it, exactly, for every method — so unlike the far end it
//! never costs a wait, a buffer or a replay.
//!
//! ## ⛔ Two things were built here and are GONE — do not rebuild either
//!
//! **The withheld tail** (built, shipped, rejected on screen — Enio 2026-08-08). The first cut of the
//! taper held dabs back until the cursor had travelled past the end-taper window, then released them
//! shaped. It is exact, and it is wrong as a product: the mark stops following the hand. The verdict was
//! *"o algoritmo que vc usou para o taper é ruim, tem um super delay e um stabilize ruim. O traço não
//! pode ter nenhum delay e nenhum stabilize"* — and the two complaints are one mechanism, because a
//! stroke that lags and then catches up in a lump is exactly what a heavy stabilizer feels like.
//!
//! **The far end itself** (Enio 2026-08-10: *"quanto à cauda do taper vamos desativar para todos os
//! modos de pintura; deixe o ajuste apenas para o início do traço"*). What used to live here was a
//! three-case `TaperSpan` whose whole job was to say where the far end was: exact for the whole-path
//! fills, unknown for a live drag, absent for a loop. With no end term in the law there is nothing left
//! to measure against, so it collapses to the one fact that survives. The reasoning behind the removal,
//! and what it cost, is in [`crate::taper`].
//!
//! So the law here is structural, not a setting: **no dab is ever withheld, and no dab is ever moved
//! from where the pointer put it.**
//!
//! ## Who answers it
//!
//! The same rule the span had, and for the same reason: **whoever knows the geometry declares it.** A
//! method table consulted from the outside would be a second answer to a question the fill is already
//! holding the geometry for — [`super::Stroke::fill_ellipse`] and [`super::Stroke::fill_polygon`] set it
//! to `false` as they build their perimeter, and everything else keeps what
//! [`method_starts_with_head`] gave it.

use crate::stroke_method::StrokeMethod;

/// Whether a stroke of `method` **starts out** with a beginning a taper may shape. The closed fills
/// overwrite it the moment they know they are a loop.
///
/// ⚠️ `false` is not "the taper is off" — it is *"this mark has no head"*, and the two cases that answer
/// it are different shapes of the same fact:
///
/// - the **single stamps** (`Anchored`, `DragDot`), decided here: one dab has no along-the-path extent,
///   so the whole window lands on the same point and the mark simply comes out shrunk — a taper that
///   silently reads as a size change.
/// - the **closed** fills (Ellipse / Polygon), decided by the fill itself: a loop's first dab lands at
///   whatever point the fill happened to start at, so tapering there puts a notch in a circle.
///
/// The whole-path fills (Line / Arc / Curve / Free Hand) and the plain drag all answer `true`: their
/// head is the pen-down, exactly, from the first dab.
pub(super) fn method_starts_with_head(method: StrokeMethod) -> bool {
    !matches!(method, StrokeMethod::Anchored | StrokeMethod::DragDot)
}
