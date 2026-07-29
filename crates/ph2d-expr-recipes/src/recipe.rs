//! What a **recipe** is.

use crate::emit::EmitCtx;
use crate::knob::Knob;

/// Stable identity of a recipe. It is what [`RecipeStack::recover`] compares and
/// what a row references, so it never changes once shipped — the label may.
///
/// [`RecipeStack::recover`]: crate::RecipeStack::recover
pub type RecipeId = &'static str;

/// The nine drawers of the gallery. Families are the artist's vocabulary, not the
/// implementation's: `Shake` and `Drift` are both *Life* even though one is sugar
/// and the other a raw `noise`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    /// Organic imperfection — shake, drift, breathe.
    Life,
    /// Rhythm — sine, bounce, ping-pong, blink.
    Wave,
    /// One object reading another — follow, mirror, distance.
    Link,
    /// The value itself — limit, remap, quantize.
    Shape,
    /// Rows that rewrite the clock.
    Time,
    /// Conditionals.
    Logic,
    /// Gradients and proximity.
    Field,
    /// Motion that looks physical without a solver.
    PhysicsLite,
    /// The artist's own formula.
    Raw,
}

impl Family {
    /// UI label (English — HR-15).
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Family::Life => "Life",
            Family::Wave => "Wave",
            Family::Link => "Link",
            Family::Shape => "Shape",
            Family::Time => "Time",
            Family::Logic => "Logic",
            Family::Field => "Field",
            Family::PhysicsLite => "Physics",
            Family::Raw => "Raw",
        }
    }

    /// Gallery order.
    pub const ALL: [Family; 9] = [
        Family::Life,
        Family::Wave,
        Family::Link,
        Family::Shape,
        Family::Time,
        Family::Logic,
        Family::Field,
        Family::PhysicsLite,
        Family::Raw,
    ];
}

/// What a row does to the fold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowKind {
    /// Transforms the value. Receives the accumulated text as [`EmitCtx::inner`].
    Value,
    /// **Rewrites the clock for the rows below it.** Emits the new clock, not a
    /// value; `inner` passes through untouched.
    Time,
    /// The artist's raw formula, passed through.
    Raw,
}

/// Whether the recipe honours a [`RowKind::Time`] row above it.
///
/// ⚠️ This exists because of a measured fact about the frozen sugar, not a taste:
/// `wiggle` builds `time + __seed` **inside `ph2d-expr-parse`**, so no clock we
/// choose can reach it. Declaring it in the data is the only way the UI can say so
/// instead of letting the artist discover that "Stepped Time does nothing to my
/// Shake".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockUse {
    /// Spells the current clock — a Time row above it applies.
    Explicit,
    /// Carries its own clock (`wiggle`) — a Time row does NOT reach it.
    Own,
    /// Reads no time at all — a Time row is simply irrelevant.
    None,
}

/// Whether the recipe has a setting that makes it the identity.
///
/// ⚠️ Not every recipe has one, and pretending otherwise makes the neutrality gate
/// vacuous. `Shake` at Amount 0 returns `value` to the bit; `Blink` **produces**
/// the value and has no neutral at any setting. The split is data, and both halves
/// are gated (a catalog that declared everything [`Neutrality::Replacing`] would
/// satisfy a gate that merely skipped them).
#[derive(Clone, Copy, Debug)]
pub enum Neutrality {
    /// These knob overrides (by [`Knob::key`]) make the recipe return `value`
    /// exactly. Knobs not listed stay at their default.
    Additive(&'static [(&'static str, f32)]),
    /// The recipe PRODUCES or CONSTRAINS the value; no knob setting makes it the
    /// identity. `Blink` replaces the value outright; `Limit` is only the identity
    /// with an infinite range, which is not a default anyone would choose.
    NoNeutral,
}

/// One entry of the catalog.
pub struct Recipe {
    pub id: RecipeId,
    pub family: Family,
    /// UI label (English — HR-15).
    pub label: &'static str,
    /// One line, shown under the label and in the search results.
    pub blurb: &'static str,
    /// The names this recipe answers to.
    ///
    /// ⚠️ With a catalog this size **the search IS the interface**, and the artist
    /// types the name they learned in another product (`wiggle`, `posterizeTime`,
    /// `Oscillate`, `linear`). A card that only answers to its own label is
    /// invisible.
    pub aliases: &'static [&'static str],
    pub knobs: &'static [Knob],
    pub kind: RowKind,
    pub clock: ClockUse,
    pub neutral: Neutrality,
    /// The other half of a pair recipe (Orbit X ↔ Orbit Y), if any. A pair is
    /// offered as one card and inserts two rows on two properties — half a circle
    /// is not a feature.
    pub pair: Option<RecipeId>,
    /// Text this recipe contributes. For [`RowKind::Value`] it is the new value
    /// expression; for [`RowKind::Time`] it is the new clock.
    pub emit: fn(&EmitCtx) -> String,
}

impl Recipe {
    /// The knob values this recipe starts with.
    #[must_use]
    pub fn default_knobs(&self) -> Vec<crate::KnobValue> {
        self.knobs.iter().map(Knob::default_value).collect()
    }

    /// The knob values at the recipe's neutral point, or `None` when it has none.
    #[must_use]
    pub fn neutral_knobs(&self) -> Option<Vec<crate::KnobValue>> {
        let Neutrality::Additive(overrides) = self.neutral else {
            return None;
        };
        let mut vals = self.default_knobs();
        for (key, v) in overrides {
            let i = self
                .knobs
                .iter()
                .position(|k| k.key == *key)
                .expect("a neutral names a knob this recipe has");
            vals[i] = crate::KnobValue::Num(*v);
        }
        Some(vals)
    }
}

impl core::fmt::Debug for Recipe {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Recipe")
            .field("id", &self.id)
            .field("family", &self.family)
            .finish_non_exhaustive()
    }
}
