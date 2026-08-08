//! **Where a stroke's ends are** — the one question [`crate::taper`] has to ask, and the one this
//! engine answers by GEOMETRY rather than by waiting.
//!
//! A tapered dab needs two distances: how far it is from the stroke's start, and how far it is from the
//! stroke's end. The first is free — [`crate::Dab::arc_len`] already carries it, exactly, for every
//! method. The second is the whole problem: for a freehand drag *the end has not happened yet*.
//!
//! ## ⛔ The withheld tail was built, shipped and REJECTED on screen (Enio 2026-08-08)
//!
//! The first cut answered it by holding dabs back until the cursor had travelled past the end-taper
//! window, then releasing them shaped. It is exact, and it is **wrong as a product**: the mark stops
//! following the hand. The verdict was *"o algoritmo que vc usou para o taper é ruim, tem um super delay
//! e um stabilize ruim. O traço não pode ter nenhum delay e nenhum stabilize"* — and the two complaints
//! are one mechanism, because a stroke that lags and then catches up in a lump is exactly what a heavy
//! stabilizer feels like.
//!
//! So the law here is now structural, not a setting: **no dab is ever withheld, and no dab is ever moved
//! from where the pointer put it.** Whatever the taper cannot know, it does not wait for.
//!
//! ## What is left, and why it is not a compromise
//!
//! The far end is answered by the SHAPE of the stroke, which the method already knows:
//!
//! - [`TaperSpan::Open`] — the whole-path fills (Line / Arc / Curve / Free Hand). The geometry exists
//!   before a single dab is stamped, so both ends are exact, live, and cost nothing at all.
//! - [`TaperSpan::Closed`] — the closed-loop fills and the single stamps. They have **no ends**: the only
//!   place a taper could land is the arbitrary point the fill happened to start at, and a circle with a
//!   notch in it is a defect, not a taper.
//! - [`TaperSpan::OpenUnknownEnd`] — the incremental drag. The start tapers exactly; the far end is not
//!   tapered, because the only two ways to taper it are to withhold (the delay that was rejected) or to
//!   re-stamp the tail from a restored base every batch — a **stroke buffer**, which is how Procreate
//!   affords it, and which is a wave with its own acceptance: it has to restore four planes plus the
//!   per-stroke coverage buffer, and the Wet Paint fluid cannot be rewound at all.
//!
//! ⚠️ So the End control is **not dead** — it is live on every method that knows its path, which is the
//! same statement as *"a taper is a property of a mark, and a mark you are still making has only one
//! end so far."*

use crate::stroke_method::StrokeMethod;

/// What a stroke's [`crate::taper`] measures its FAR end against.
///
/// Set by whoever knows the geometry — the fills declare theirs as they build it, and a plain drag never
/// declares one at all. Deliberately three cases and not an `Option<f32>`: *"a loop has no ends"* and
/// *"this end has not happened yet"* are different facts with different right answers, and an `Option`
/// would collapse them into one and make the caller guess.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum TaperSpan {
    /// The path grows under the pointer and its far end is in the future — taper the start only.
    OpenUnknownEnd,
    /// A path of known total length (px): both ends are exact from the first dab.
    Open(f32),
    /// A closed contour: no ends to taper.
    Closed,
}

impl TaperSpan {
    /// The span a stroke of `method` starts with. The fills overwrite it the moment they know their own
    /// geometry ([`super::Stroke::fill_curve`] and friends); the rest keep what they are given.
    ///
    /// ⚠️ `Anchored` and `DragDot` are `Closed` because they are ONE stamp: a single dab has no
    /// along-the-path extent, so both windows would land on the same point and the mark would simply
    /// come out shrunk — a taper that silently reads as a size change.
    pub(super) fn for_method(method: StrokeMethod) -> Self {
        match method {
            StrokeMethod::Anchored | StrokeMethod::DragDot => Self::Closed,
            _ => Self::OpenUnknownEnd,
        }
    }
}
