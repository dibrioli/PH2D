//! Node UI side-metadata (Motion Nodes M1.R1).
//!
//! Keyed by `NodeTypeId`, **additive and non-frozen** — it lives beside the ops
//! on [`super::NodeRegistry`], entirely outside the frozen `NodeManifest` (8
//! fields) and its contract gate. The motion-graph editor reads it to label,
//! categorize, and shape each node card; a node registers its UI manifest right
//! next to its op in `register`.
//!
//! Param hints (`ParamUiHint`) + attribute access (`AttrAccess`) land with the
//! params panel (M1.P1) as further additive fields here.

/// Visual category of a node — selects the card's header tint (the `node-cat-*`
/// ColorTokens) so the palette's colors teach the library map (plan §2.4).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NodeUiCategory {
    /// Generators (grid, random, …) — green.
    Source,
    /// Distribution / cloning — muted green.
    Distribute,
    /// Spatial transforms (move, scale, rotate) — blue.
    Transform,
    /// Falloffs / focus fields — amber.
    Focus,
    /// Color / stylistic effects — magenta.
    Fx,
    /// Terminal / output — red.
    Output,
    /// Values, adapters, utilities — grey.
    Utility,
}

/// Card silhouette — the 7 body shapes (plan §2.4). `Rect` is the neutral
/// modifier default; the others read the node's role at a glance.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NodeSilhouette {
    /// Rounded rectangle — a modifier (default).
    Rect,
    /// Cigar (fully rounded ends) — merge / group.
    Cigar,
    /// Circle — a terminal / sink.
    Circle,
    /// Diamond — a value / gate.
    Diamond,
    /// Trapezoid widening downward — a source / generator.
    TrapezoidDown,
    /// Trapezoid narrowing downward — a sink.
    TrapezoidUp,
    /// Tabbed rect — an event / signal node.
    Tabbed,
}

/// Per-node-type UI metadata (M1.R1). `display_name` is the English,
/// result-named label (HR-15); `category` + `silhouette` drive the card look.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NodeUiManifest {
    pub display_name: &'static str,
    pub category: NodeUiCategory,
    pub silhouette: NodeSilhouette,
}

/// The unit a [`ParamWidget::Angle`] param stores its value in. The panel always
/// *displays* degrees (the artist-facing unit, with a `deg` chip); this says what
/// to convert from. Motion has both: a node's own cycle-based trig reads
/// [`Turns`](AngleUnit::Turns), while anything written into the stream's `rot`
/// column reads [`Radians`](AngleUnit::Radians) (the renderer's basis unit).
/// Making it explicit beats a per-node comment nobody reads.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AngleUnit {
    /// Whole turns / cycles: `1.0` = a full revolution.
    Turns,
    /// Radians: `2π` = a full revolution (the `rot` stream column's unit).
    Radians,
}

impl AngleUnit {
    /// Convert a value in this unit to degrees (what the panel shows).
    #[must_use]
    pub fn to_degrees(self, native: f32) -> f32 {
        match self {
            AngleUnit::Turns => native * 360.0,
            AngleUnit::Radians => native.to_degrees(),
        }
    }

    /// Factor that converts a degrees value back to this unit — the panel
    /// multiplies by it so the emitted param value is always node-native.
    #[must_use]
    pub fn deg_to_native(self) -> f32 {
        match self {
            AngleUnit::Turns => 1.0 / 360.0,
            AngleUnit::Radians => core::f32::consts::PI / 180.0,
        }
    }
}

/// Which widget a param row renders in the params panel (M1.P1). The frozen
/// [`ph2d_nodegraph::node::ParamSpec`] only carries `{name, default: f32}`, so
/// the editable range + control + label live here as additive side-metadata.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ParamWidget {
    /// Continuous float slider.
    Slider,
    /// Whole-number slider — the chip rounds to an integer (count / index).
    IntSlider,
    /// An angle: a numeric box with a `deg` unit chip (type `90deg`, drag-scrub),
    /// always shown in **degrees**. `unit` says what the param itself stores, so
    /// the panel converts both ways — a turns-based node and a radians-based one
    /// both read as degrees to the artist.
    Angle { unit: AngleUnit },
    /// On/off toggle (0.0 / 1.0). Rendered as a slider clamped to `0..1` in v1.
    Toggle,
    /// Random seed — a whole-number box plus a **re-roll** button (a seed has no
    /// meaningful drag range; the artist wants "another one", not "a bigger one").
    Seed,
    /// An RGBA colour authored via a single swatch → OKLCH picker (the canonical
    /// colour UI, not four raw channel sliders). Declared on ONE hint whose
    /// `param` anchors the group; `channels` names the four **linear-straight**
    /// RGBA channel params it drives (each a declared [`ph2d_nodegraph::node::ParamSpec`]).
    /// The params panel paints one swatch for the group (suppressing the four
    /// scalar rows); the shell bridge reads the pick back into these params
    /// (sRGB→linear) and seeds the picker (linear→sRGB). Reusable by every colour
    /// node (tint, colour-array, gradient, …).
    Color { channels: [&'static str; 4] },
    /// A single-select from a fixed set of **named** options, rendered as a
    /// segmented-button row (the same selector the Vector panel uses for
    /// Cap / Join / Draw) — never a number slider the user must decode. The param
    /// stores the selected option **index** as its `f32` value (`0..labels.len()`).
    /// Reusable by every enum-valued param (channel, waveform, easing, …).
    Enum { labels: &'static [&'static str] },
}

impl ParamWidget {
    /// Whether the chip displays / commits whole numbers (drives the
    /// integer-snapping link in the params panel).
    #[must_use]
    pub fn is_integer(self) -> bool {
        matches!(self, ParamWidget::IntSlider | ParamWidget::Seed)
    }
}

/// UI hint for one declared param (M1.P1) — the editable range, step, control,
/// and English label for a `ParamSpec` of some node type. Registered as an
/// additive side-table on [`super::NodeRegistry`] (keyed by `NodeTypeId`),
/// entirely outside the frozen `NodeManifest`. A param with no hint falls back
/// to a plain [`ParamWidget::Slider`] over a default range in the panel.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ParamUiHint {
    /// The `ParamSpec::name` this hint annotates.
    pub param: &'static str,
    /// English, result-named label (HR-15).
    pub label: &'static str,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub widget: ParamWidget,
}
