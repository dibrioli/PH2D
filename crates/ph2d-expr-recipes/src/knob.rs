//! **The knobs a recipe exposes** — the thing that separates a catalog from a
//! preset library.
//!
//! An After Effects *Expression Preset* drops opaque text you cannot reconfigure
//! without reading it, and that is precisely why nobody uses them after two
//! decades. A knob is the difference between "wiggle" and "wiggle, a bit slower".

/// What a knob accepts. The kind decides the widget AND the emitted text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KnobKind {
    /// A plain number the artist drags.
    Number,
    /// A number that must reach the formula as a **literal**.
    ///
    /// ⚠️ Not decoration: `wiggle`'s `octaves`/`amp_mult` size the unrolled noise
    /// tree, so the parser REFUSES anything but a numeric literal there. A knob
    /// that let an expression through would emit a formula that cannot parse.
    Literal,
    /// A `Name.prop` link — the pick-whip fills this one.
    Link,
    /// Free text — the artist's own formula (the `Custom Formula` recipe).
    ///
    /// The escape hatch is an **item of the catalog**, not a hidden mode: Cinema 4D
    /// (`Formula`), Cavalry (`JavaScript Deformer`) and Rive (`Formula`) all put it
    /// there, and hiding it is what makes people think the tool cannot do it.
    Text,
}

/// One knob on a recipe.
#[derive(Clone, Copy, Debug)]
pub struct Knob {
    /// Stable key — what [`Neutrality::Additive`](crate::Neutrality) names and
    /// what a saved row would reference. Never the label.
    pub key: &'static str,
    /// UI label (English — HR-15).
    pub label: &'static str,
    pub kind: KnobKind,
    /// Where the knob sits when the recipe is first dropped on a row.
    pub default: f32,
    /// Soft range for the slider. Typing outside it is the artist's business;
    /// this is the drag range, not a validity claim.
    pub range: (f32, f32),
}

impl Knob {
    /// A number knob.
    #[must_use]
    pub const fn num(
        key: &'static str,
        label: &'static str,
        default: f32,
        range: (f32, f32),
    ) -> Self {
        Self {
            key,
            label,
            kind: KnobKind::Number,
            default,
            range,
        }
    }

    /// A knob whose value must reach the text as a literal (see [`KnobKind::Literal`]).
    #[must_use]
    pub const fn lit(
        key: &'static str,
        label: &'static str,
        default: f32,
        range: (f32, f32),
    ) -> Self {
        Self {
            key,
            label,
            kind: KnobKind::Literal,
            default,
            range,
        }
    }

    /// A `Name.prop` link knob.
    #[must_use]
    pub const fn link(key: &'static str, label: &'static str) -> Self {
        Self {
            key,
            label,
            kind: KnobKind::Link,
            default: 0.0,
            range: (0.0, 0.0),
        }
    }

    /// A free-text knob.
    #[must_use]
    pub const fn text(key: &'static str, label: &'static str) -> Self {
        Self {
            key,
            label,
            kind: KnobKind::Text,
            default: 0.0,
            range: (0.0, 0.0),
        }
    }

    /// The value this knob starts at.
    #[must_use]
    pub fn default_value(&self) -> KnobValue {
        match self.kind {
            KnobKind::Number | KnobKind::Literal => KnobValue::Num(self.default),
            KnobKind::Link => KnobValue::Link(String::new()),
            KnobKind::Text => KnobValue::Text(String::new()),
        }
    }
}

/// What a knob currently holds.
#[derive(Clone, Debug, PartialEq)]
pub enum KnobValue {
    Num(f32),
    /// `"Ball.x"` — or **empty**, which is a row the artist has not finished.
    ///
    /// ⚠️ An empty link still has to produce a formula that PARSES (it emits `0`),
    /// because the formula bar and the preview render on every keystroke, long
    /// before the pick-whip has been used. A row that emits `.x` would blank the
    /// whole modal the moment a Follow row is added.
    Link(String),
    /// Free text (the `Custom Formula` recipe). Empty means "pass the value
    /// through" — an unfinished Custom row must not break the formula either.
    Text(String),
}

impl KnobValue {
    /// The number, or `0.0` for the string-shaped kinds (callers that want a
    /// number ask a knob that has one).
    #[must_use]
    pub fn as_num(&self) -> f32 {
        match self {
            KnobValue::Num(n) => *n,
            KnobValue::Link(_) | KnobValue::Text(_) => 0.0,
        }
    }

    /// The text of a `Link`/`Text` knob, trimmed; empty for a number.
    #[must_use]
    pub fn as_text(&self) -> &str {
        match self {
            KnobValue::Link(s) | KnobValue::Text(s) => s.trim(),
            KnobValue::Num(_) => "",
        }
    }
}
