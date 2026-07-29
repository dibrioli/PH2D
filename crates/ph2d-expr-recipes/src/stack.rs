//! **The stack** — rows folded into one formula.

use crate::catalog::by_id;
use crate::emit::EmitCtx;
use crate::knob::KnobValue;
use crate::recipe::{Combine, RecipeId, RowKind};

/// One row of the sheet.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub recipe: RecipeId,
    /// Positionally matched to the recipe's knobs.
    pub knobs: Vec<KnobValue>,
    /// Muted. ⚠️ Byte-identical to removing the row — gated, because a bypass that
    /// merely zeroes a knob leaves the row's arithmetic in the formula, and the
    /// artist toggling it would see the eye change and the picture not.
    pub bypass: bool,
    /// **How this row's contribution lands on the value the rows above produced.**
    ///
    /// Seeded from [`Recipe::combine`] and only CONSULTED for a source (a recipe
    /// whose `combine` is `Some`); a modifier folds `inner` itself and this field is
    /// inert for it. Keeping the field unconditional — rather than an `Option`
    /// mirroring the recipe — means the chip can be re-offered for a row whose
    /// recipe changes without a second place to keep in step.
    ///
    /// [`Recipe::combine`]: crate::Recipe::combine
    pub combine: Combine,
}

impl Row {
    /// A row at the recipe's defaults.
    #[must_use]
    pub fn new(recipe: RecipeId) -> Option<Self> {
        let r = by_id(recipe)?;
        Some(Self {
            recipe,
            knobs: r.default_knobs(),
            bypass: false,
            combine: r.combine.unwrap_or_default(),
        })
    }

    /// A row at the recipe's NEUTRAL point, or `None` when it has none.
    #[must_use]
    pub fn neutral(recipe: RecipeId) -> Option<Self> {
        let r = by_id(recipe)?;
        Some(Self {
            recipe,
            knobs: r.neutral_knobs()?,
            bypass: false,
            combine: r.combine.unwrap_or_default(),
        })
    }

    /// Whether this row's [`Row::combine`] is the artist's to choose — true exactly
    /// for a SOURCE. The card asks it to decide whether to paint the chip, and
    /// [`RecipeStack::to_formula`] asks it to decide whether to honour the field: two
    /// answers to *"does this row have a mode?"* would let the picture and the chip
    /// disagree.
    #[must_use]
    pub fn combines(&self) -> bool {
        by_id(self.recipe).is_some_and(|r| r.combine.is_some())
    }

    /// Overwrite one knob by key. Returns `false` when the recipe has no such knob.
    pub fn set(&mut self, key: &str, value: KnobValue) -> bool {
        let Some(r) = by_id(self.recipe) else {
            return false;
        };
        let Some(i) = r.knobs.iter().position(|k| k.key == key) else {
            return false;
        };
        self.knobs[i] = value;
        true
    }

    /// Overwrite one numeric knob by key.
    pub fn set_num(&mut self, key: &str, v: f32) -> bool {
        self.set(key, KnobValue::Num(v))
    }

    /// **The knob this row is still waiting for** — the label of the first empty
    /// link or text knob, or `None` when the row is ready to run.
    ///
    /// ⚠️ This exists because of a measured defect, and the mechanism is worth
    /// keeping written down. A link knob is born EMPTY, and [`EmitCtx::link`] lowers
    /// an empty link to the literal `0` — deliberately, for a reason that is still
    /// correct: the formula bar and the preview render on every keystroke, so a row
    /// that emitted `.x` would blank the whole card the moment a `Follow` was added.
    ///
    /// What was never priced is the CONSEQUENCE. `0` parses, and it also **replaces
    /// the property with a constant**: measured across the catalog at defaults,
    /// `follow` lowers to `0*1 + 0`, so picking it from the gallery teleported the
    /// object to the origin and froze it there; `offset-copy` parked it at `0.2`;
    /// `fade-by-distance` pinned it at `1`. Fourteen recipes, one cause — and it is
    /// the same cause as *"não produz a curva do grafo de preview"*, because a
    /// constant draws a flat line. One defect, two symptoms.
    ///
    /// ⚠️ The answer is NOT to make the lowering emit something else: it is that a
    /// row nobody has finished should not be in the fold at all. That keeps the
    /// original invariant (the formula always parses — an empty stack is `"value"`)
    /// and drops the teleport, and it is a rule the sheet already has a shape for,
    /// because it is exactly what `bypass` does.
    ///
    /// One door: [`RecipeStack::to_formula`] asks it to decide whether to fold the
    /// row, and the panel asks it to say what is missing. Two answers to *"is this
    /// row finished?"* would let the picture and the label disagree.
    #[must_use]
    pub fn waiting_for(&self) -> Option<&'static str> {
        let r = by_id(self.recipe)?;
        r.knobs.iter().enumerate().find_map(|(i, k)| {
            let empty = matches!(
                self.knobs.get(i),
                Some(KnobValue::Link(s) | KnobValue::Text(s)) if s.trim().is_empty()
            );
            empty.then_some(k.label)
        })
    }

    /// Whether this row contributes to the fold at all.
    #[must_use]
    pub fn is_live(&self) -> bool {
        !self.bypass && self.waiting_for().is_none()
    }
}

/// The sheet: rows applied **top to bottom**.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RecipeStack {
    pub rows: Vec<Row>,
}

/// What the bottom of the fold starts from — the property's pre-expression value.
pub const SEED_VALUE: &str = "value";
/// The clock a stack starts on, before any [`RowKind::Time`] row changes it.
pub const SEED_CLOCK: &str = "time";

impl RecipeStack {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Build from recipe ids at their defaults; unknown ids are skipped.
    #[must_use]
    pub fn of(ids: &[RecipeId]) -> Self {
        Self {
            rows: ids.iter().filter_map(|id| Row::new(id)).collect(),
        }
    }

    pub fn push(&mut self, row: Row) {
        self.rows.push(row);
    }

    /// **The formula this stack produces** — the ONE door, and the only string the
    /// UI ever shows.
    ///
    /// The fold carries two accumulators: the value expression built so far, and
    /// the **clock** in force. A [`RowKind::Value`] row rewrites the value; a
    /// [`RowKind::Time`] row rewrites the clock for every row BELOW it. That is
    /// why a Time row costs nothing extra — we own the emitted text, so the clock
    /// is simply the string the next row is handed.
    ///
    /// ⚠️ An empty stack is `"value"`, not `""`: the property is driven by a
    /// formula that returns exactly what it already had, which is what an empty
    /// sheet means and is what keeps the preview alive while the artist decides.
    #[must_use]
    pub fn to_formula(&self) -> String {
        let mut acc = SEED_VALUE.to_string();
        let mut clock = SEED_CLOCK.to_string();
        for row in &self.rows {
            // ⚠️ A muted row and an UNFINISHED row are skipped by the same `continue`,
            // and that is the whole fix for the fourteen link recipes that used to
            // teleport the object (see [`Row::waiting_for`]). Both are "this row does
            // not contribute", and giving them one door is what keeps the eye, the
            // readout and the picture from disagreeing about it.
            if !row.is_live() {
                continue;
            }
            let Some(r) = by_id(row.recipe) else { continue };
            let ctx = EmitCtx {
                knobs: &row.knobs,
                defs: r.knobs,
                inner: &acc,
                clock: &clock,
            };
            let out = (r.emit)(&ctx);
            match (r.kind, r.combine) {
                (RowKind::Time, _) => clock = out,
                // A MODIFIER already folded `inner` — `Limit` clamped it, `Quantize`
                // rounded it. There is nothing to choose, and `row.combine` is inert.
                (RowKind::Value | RowKind::Raw, None) => acc = out,
                // A SOURCE emitted its contribution ALONE; the row says how it lands.
                (RowKind::Value | RowKind::Raw, Some(_)) => acc = row.combine.fold(&acc, &out),
            }
        }
        acc
    }
}
