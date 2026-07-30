//! What a **recipe** is.

use crate::emit::EmitCtx;
use crate::knob::Knob;

/// Stable identity of a recipe — what a row references, so it never changes once
/// shipped; the label may.
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
            Family::PhysicsLite => "Physics",
            Family::Raw => "Raw",
        }
    }

    /// Gallery order.
    /// ⚠️ **Sete, não nove** (FASE A do plano 12): `Logic` e `Field` foram removidas com as
    /// receitas delas. Uma família vazia é uma gaveta que abre para nada — é o que o gate
    /// `the_catalog_covers_every_family` existe para impedir, e manter os variants "por
    /// enquanto" seria exatamente isso. As duas respondem por `refusal::REFUSALS`.
    pub const ALL: [Family; 7] = [
        Family::Life,
        Family::Wave,
        Family::Link,
        Family::Shape,
        Family::Time,
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

/// **How a SOURCE row folds into the value the rows above it produced.**
///
/// ⚠️ This exists because of a report, and the report was right: *"expressões não
/// podem ser somadas, multiplicadas"*. Measured across the catalog, **29 of 55
/// recipes ignored [`EmitCtx::inner`]** — `Blink`, `Pulse`, every `Link`, every
/// `Field` — so stacking `Sway` and then `Blink` produced `select(...)` and the
/// Sway was **silently discarded**. And the 13 that did compose had the operator
/// **baked into the emitted text** (`Sway` spelt `{inner} + sin(...)`), so there
/// was no way to ask for a product instead of a sum.
///
/// The split that fixes both: a **SOURCE** emits only its own contribution and the
/// FOLD decides how it lands ([`Recipe::combine`] is `Some`); a **MODIFIER**
/// transforms `inner` and its combination is intrinsic — `Limit` clamps, it does not
/// "add" (`combine` is `None`, and the card offers no chip).
///
/// ⚠️ **Every recipe's default is what it did before**, so the whole catalog is
/// value-identical at its defaults — gated termwise by
/// `the_catalog_is_value_identical_to_the_pre_combine_world`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Combine {
    /// `acc + contribution` — the sum. What every generator did.
    #[default]
    Add,
    /// `acc * contribution` — the product. What `Flicker` did.
    Multiply,
    /// The contribution alone; the rows above are dropped. What a `Link` does, and
    /// the honest default for one: `Follow` means *be where that is*.
    Replace,
}

impl Combine {
    /// UI label — one glyph, because it sits on a row header (English, HR-15).
    #[must_use]
    pub const fn glyph(self) -> &'static str {
        match self {
            Combine::Add => "+",
            Combine::Multiply => "x",
            Combine::Replace => "=",
        }
    }

    /// The full name, for the tooltip and the gallery.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Combine::Add => "Add",
            Combine::Multiply => "Multiply",
            Combine::Replace => "Replace",
        }
    }

    /// Cycle order for the chip.
    pub const ALL: [Combine; 3] = [Combine::Add, Combine::Multiply, Combine::Replace];

    /// The next mode the chip lands on.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Combine::Add => Combine::Multiply,
            Combine::Multiply => Combine::Replace,
            Combine::Replace => Combine::Add,
        }
    }

    /// **Fold a source's contribution into the accumulated value.**
    ///
    /// ⚠️ The parenthesisation is [`crate::emit::paren`]'s job and not a choice made
    /// here: `acc` may be a sum, and `value + sin(t)` multiplied without brackets
    /// means `value + (sin(t)*k)` — a different animation, that parses.
    #[must_use]
    pub fn fold(self, acc: &str, contribution: &str) -> String {
        match self {
            Combine::Add => format!("{acc} + {contribution}"),
            Combine::Multiply => format!(
                "{}*{}",
                crate::emit::paren(acc),
                crate::emit::paren(contribution)
            ),
            Combine::Replace => contribution.to_string(),
        }
    }
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
    /// **`Some` makes this a SOURCE; `None` makes it a MODIFIER** — see [`Combine`].
    ///
    /// A source's [`Recipe::emit`] returns only its own contribution and MUST NOT read
    /// [`EmitCtx::inner`]; the value it names is the default mode a new row gets, and
    /// the artist can change it per row. A modifier's `emit` reads `inner` and folds
    /// it however it likes; no chip is offered, because there is nothing to choose.
    ///
    /// ⚠️ Gated both ways (`a_source_never_reads_inner_and_a_modifier_always_does`):
    /// a source that reads `inner` would have its contribution counted twice under
    /// Add, and a modifier that ignores it silently drops the rows above.
    pub combine: Option<Combine>,
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
