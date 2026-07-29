//! **The stack** — rows folded into one formula.

use crate::catalog::by_id;
use crate::emit::EmitCtx;
use crate::knob::KnobValue;
use crate::recipe::{RecipeId, RowKind};

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
        })
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
            if row.bypass {
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
            match r.kind {
                RowKind::Value | RowKind::Raw => acc = out,
                RowKind::Time => clock = out,
            }
        }
        acc
    }
}
