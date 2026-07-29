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
    ///
    /// ⚠️ **In the units of the property being driven** for anything that carries a
    /// value — see [the crate docs](crate) for the one rule that makes a single
    /// number read sensibly in metres, radians, scale factors and `0..1` alike.
    pub default: f32,
    /// Soft range: what the number box's **stepper arrows and drag** honour. Typing
    /// outside it is the artist's business — the commit path never clamps, only the
    /// arrows and the drag do — so this is a working range, not a validity claim.
    /// The one exception is [`KnobKind::Literal`], where it IS validity.
    pub range: (f32, f32),
    /// Stepper increment. `0` means *derive it from [`Self::range`]* — see
    /// [`Self::step_value`], which is right for every knob in the catalog but one.
    pub step: f32,
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
            step: 0.0,
        }
    }

    /// Override the derived stepper increment.
    ///
    /// ⚠️ Exactly ONE knob in the catalog needs this (`wiggle` octaves, which are
    /// integers), and that is the measure of whether [`Self::step_value`]'s
    /// derivation is pulling its weight. A second caller is a hint that the
    /// derivation is wrong, not that the knob is special.
    #[must_use]
    pub const fn step(mut self, step: f32) -> Self {
        self.step = step;
        self
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
            step: 0.0,
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
            step: 0.0,
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
            step: 0.0,
        }
    }

    /// What one click of a stepper arrow is worth.
    ///
    /// Derived from the range unless [`Self::step`] overrode it: a two-hundredth of
    /// the span, snapped DOWN to a round number so the box counts in 0.05s and 0.1s
    /// instead of 0.3987s. Snapping down (never up) keeps a click from overshooting
    /// a knob whose whole working range is small.
    ///
    /// ⚠️ Registering this is not decoration: without a registered step the dispatch
    /// falls back to a buffer heuristic (`0.01` when the text has a dot, else `1`),
    /// and a `1.0` click on an Amount whose default is `0.3` moves three canvases.
    #[must_use]
    pub fn step_value(&self) -> f32 {
        if self.step > 0.0 {
            return self.step;
        }
        /// Clicks it should take to cross the whole range.
        const CLICKS_ACROSS: f32 = 200.0; // LITERAL-PX-OK: CONTAGEM de cliques, nao medida
        let raw = (self.range.1 - self.range.0).abs() / CLICKS_ACROSS;
        const NICE: [f32; 9] = [
            0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0,
            10.0, // LITERAL-PX-OK: grade de incrementos redondos, nao medidas
        ];
        let mut out = NICE[0];
        for n in NICE {
            if n <= raw {
                out = n;
            }
        }
        out
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
