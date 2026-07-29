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

/// One named read-channel for a [`ParamWidget::Channels`] picker (M1.P1 · plan §1.1).
/// `label` is the human word the artist picks; `column` is the stream column the text
/// param is set to; `mode` is the sibling f32 param's value (for `value.attribute`,
/// `0` = read the scalar column, `1` = the magnitude of a Vec2 column). `i32` so the
/// enclosing [`ParamWidget`] can stay `Eq`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ReadChannel {
    pub label: &'static str,
    pub column: &'static str,
    pub mode: i32,
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
    /// An angle: a numeric box with a `deg` unit chip (type `90deg`, drag-scrub,
    /// stepper). **The param stores DEGREES** — the app's one authored-angle unit
    /// (the Painter's `*_angle_deg` fields, the Inspector's `deg` boxes). Radians
    /// and turns are implementation details of whatever consumes the value, never
    /// of the value itself, so there is nothing for the panel to convert.
    Angle,
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
    /// A **named-channel picker** for a TEXT param that names a stream column, with a
    /// sibling f32 `mode_param` folded in (plan §1.1). The panel paints the channel
    /// LABELS as a segmented selector plus a trailing "Custom…"; picking a channel sets
    /// the text param to its `column` and the mode param to its `mode`, and only Custom
    /// reveals the raw text field (the power-user escape). It is artist-facing sugar
    /// over [`Self::Text`] — the substrate is untouched, the text param stays the source
    /// of truth — so an artist reads "Speed" instead of typing `vel` and a magic mode.
    Channels {
        mode_param: &'static str,
        channels: &'static [ReadChannel],
    },
    /// A **source picker** for a TEXT param that names a value the APP published into
    /// the graph's external channel (doc 65) — e.g. a drawn vector shape, by the name it
    /// carries in the scene. The panel offers the LIVE list of published names as
    /// clickable chips (populated at runtime from `Cook::externals`) plus a text field
    /// for a name not yet drawn (a forward reference is legal — it is a reference, not a
    /// lookup). Artist-facing sugar over [`Self::Text`]: the artist picks "Star" from the
    /// shapes they drew instead of typing its exact internal name, and the substrate is
    /// untouched — the text param stays the source of truth. Reusable by every node that
    /// reads an external by name (`motion.path`, and later any `*.external` reader).
    Source,
    /// A free-text field editing a **text param** (a `motion.expression` formula) —
    /// NOT a `ParamSpec` (which is f32-only). The hint's `param` names the text-param
    /// key (`Graph::set_text_param`), read/written through the additive text channel
    /// (doc 32), and the panel renders the shared `TextInput` widget. `min/max/step`
    /// are inert for a text field.
    Text,
    /// An interactive **transfer-curve editor** — the sibling of [`Self::Text`],
    /// on the SAME text channel: the hint's `param` names a text-param key whose
    /// value is a `ph2d-curve` serialized curve (`Graph::set_text_param`, doc
    /// 32), never a `ParamSpec` (a curve is not one number). The panel draws the
    /// unit-square editor (draggable points + per-point interp) and writes the
    /// string back on edit. Reusable by every curve-valued transfer — the
    /// `field.remap` **Curve** contour, and (later) `value.curve` / `force.curve`.
    /// `min/max/step` are inert.
    Curve,
    /// An interactive **gradient editor** — the colour sibling of [`Self::Curve`],
    /// on the SAME text channel: the hint's `param` names a text-param key whose
    /// value is a `ph2d_color::ColorRamp` serialized with `serialize_gradient`
    /// (`Graph::set_text_param`, doc 32), never a `ParamSpec` (a multi-stop
    /// gradient is not a fixed set of `f32`s). The panel draws a gradient bar
    /// with **draggable stops** (position via the same `InteractiveState::CurvePoint`
    /// x-drag the curve editor uses) and a per-stop **OKLCH swatch** (the canonical
    /// colour UI, `register_picker_swatch`), plus add/remove and an interp cycle.
    /// A stop's colour is read back through the shell's sRGB↔linear boundary
    /// exactly as [`Self::Color`] is — the string stays the source of truth.
    /// Reusable by every multi-stop-gradient param (`motion.color_ramp` Custom,
    /// and later any colour-over-value node). `min/max/step` are inert.
    Gradient,
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
    /// The **soft** maximum: how far the SLIDER drags. Also the typed ceiling
    /// unless a [`ParamHardMax`] widens it.
    pub max: f32,
    pub step: f32,
    pub widget: ParamWidget,
}

/// A param whose typed entry reaches further than its slider drags — Blender's
/// **soft vs hard limits**, and the same reason: the useful drag range and the
/// legal range are different questions. `rate` on `motion.emitter` is the case
/// that forced it — a fountain is authored between 0 and ~12k particles/sec, but
/// a one-frame burst is a legitimate `rate` in the millions paired with a
/// millisecond `life`, and no slider should have to span that to allow it.
///
/// Registered as its own additive side-table rather than a field on
/// [`ParamUiHint`] **for a mechanical reason worth stating**: the hint is a
/// struct literal at 275 sites across 78 crates, and a new field is 275 edits to
/// express one exception. A param with no entry here types to its soft `max`,
/// which is what every existing hint already means — so nothing moves by
/// default.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ParamHardMax {
    /// The `ParamSpec::name` this widens.
    pub param: &'static str,
    /// The typed-entry ceiling. Must be ≥ the hint's `max` to mean anything.
    pub max: f32,
}
