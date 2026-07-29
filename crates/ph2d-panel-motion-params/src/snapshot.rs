//! Params-panel state channels (Motion Nodes M1.P1) — the publish/return seam
//! between the shell bridge and this panel (mold of the graph panel's
//! `snapshot` + `intent` channels).
//!
//! The bridge builds a [`ParamsSnapshot`] each frame from the selected node's
//! manifest params + the registry's `ParamUiHint`s + the graph's per-instance
//! overrides, and hands it over [`set_current_params`]; `paint` reads it with
//! [`current_params`]. A row edit returns as a [`MotionParamIntent`] the bridge
//! drains + applies (`Graph::set_param`). Neither side downcasts the other.

use ph2d_a11y::NodeId;
use std::cell::RefCell;

/// One editable param row, resolved to primitives the panel paints without
/// touching the registry / graph:
/// - `Scalar` — a slider + numeric chip.
/// - `Color` — a swatch -> OKLCH picker (folds four RGBA channel params).
/// - `Toggle` — a real checkbox (never a 0/1 slider).
/// - `Enum` — a named segmented-button selector (never a number slider).
/// - `Angle` — a numeric box with a `deg` unit chip (never a raw-radians slider).
/// - `Seed` — a whole-number box + a re-roll button (never a slider to drag).
#[derive(Clone, Debug, PartialEq)]
pub enum ParamRow {
    Scalar(ScalarRow),
    Color(ColorRow),
    Toggle(ToggleRow),
    Enum(EnumRow),
    Angle(AngleRow),
    Seed(SeedRow),
    Text(TextRow),
    Curve(CurveRow),
    Gradient(GradientRow),
    Channels(ChannelsRow),
    Source(SourceRow),
}

/// A **named-channel picker** row (plan §1.1) — segmented channel buttons plus a
/// trailing "Custom…", the artist-facing face of a stream-column TEXT param.
///
/// `selected` is the channel index, or `channels.len()` for **Custom** (the live
/// value matches no channel, so the raw text field is shown to edit `custom`).
/// Picking channel `i` writes `text_param = channels[i].column` and `mode_param =
/// channels[i].mode` (two intents); Custom writes only `text_param` (from the field).
#[derive(Clone, Debug, PartialEq)]
pub struct ChannelsRow {
    pub label: String,
    /// The text param the chosen column lands in (`Graph::set_text_param`).
    pub text_param: &'static str,
    /// The sibling f32 param a channel also sets (`Graph::set_param`).
    pub mode_param: &'static str,
    /// One `(label, column, mode)` per channel — resolved primitives, so the panel
    /// stays registry-free. "Custom…" is appended by the paint, not stored here.
    pub channels: Vec<(&'static str, &'static str, i32)>,
    /// `channels.len()` = Custom (no channel matches the live column + mode).
    pub selected: usize,
    /// The live text-param value — shown in (and edited via) the Custom field.
    pub custom: String,
    /// The scalar columns the UPSTREAM stream actually carries right now (minus the
    /// ones the curated channels already cover) — the roadmap's *dropdown populated
    /// at runtime*. Shown as clickable chips under the Custom field so the artist
    /// picks a real column (`id`, `Index`, a custom attribute) by name instead of
    /// guessing it blind. Empty when nothing upstream cooked → just the text field.
    pub extra: Vec<String>,
}

/// A **source picker** row (doc 65) — clickable chips of the names the app has
/// published into the graph's external channel (drawn shapes), plus a text field for a
/// name not yet drawn. The artist-facing face of a TEXT param that references an
/// external by name; picking a chip writes `param = options[j]`, the field writes it raw.
///
/// `current` highlights the matching chip (or fills the field when it matches none). The
/// substrate is untouched — the text param stays the source of truth, so this is pure UI
/// sugar over [`TextRow`], the same way [`ChannelsRow`] is for a stream column.
#[derive(Clone, Debug, PartialEq)]
pub struct SourceRow {
    /// English label (from the `ParamUiHint`), e.g. "Shape".
    pub label: String,
    /// The text param the chosen name lands in (`Graph::set_text_param`).
    pub param: &'static str,
    /// The live published names (`Cook::externals` keys) — clickable chips. Empty when
    /// the app has published nothing yet → just the text field (the honest escape).
    pub options: Vec<String>,
    /// The current text-param value — highlights the matching chip and fills the field.
    pub current: String,
}

/// An interactive **transfer-curve editor** (A1) — a `ph2d-curve` serialized in a
/// text param (`Graph::set_text_param`), drawn as a graph with **draggable control
/// points** (the foundational `InteractiveState::CurvePoint` 2-D drag, reused from
/// the Painter's falloff editor) + add/remove buttons. Like [`TextRow`] the value is
/// a `String`, so an edit rides [`MotionParamIntent::SetTextParam`]; unlike it, the
/// artist never sees the string — only the curve and the handles.
#[derive(Clone, Debug, PartialEq)]
pub struct CurveRow {
    /// The text-param key (`Graph::set_text_param`) — echoed in the intent.
    pub name: &'static str,
    /// English label (from the `ParamUiHint`).
    pub label: String,
    /// The current serialized curve (the text-param override, else empty → the
    /// panel opens on the identity diagonal).
    pub value: String,
}

/// An interactive **gradient editor** (doc 85) — the colour sibling of [`CurveRow`]: a
/// `ph2d_color::ColorRamp` serialized in a text param (`Graph::set_text_param`,
/// `serialize_gradient`), drawn as a gradient BAR with **draggable position markers** (the
/// same `InteractiveState::CurvePoint` x-drag the curve editor uses, y ignored) + a
/// per-stop **OKLCH swatch** (`register_picker_swatch`, the shell reads the pick back into
/// the string) + add/remove and an interp cycle. Like [`CurveRow`] the value is a `String`,
/// so a position/add/remove edit rides [`MotionParamIntent::SetTextParam`]; a colour edit
/// rides the shell's picker read-back (mirror of [`ColorRow`]). The artist never sees the
/// string — only the bar and the stops.
#[derive(Clone, Debug, PartialEq)]
pub struct GradientRow {
    /// The text-param key (`Graph::set_text_param`) — echoed in the intent + the
    /// per-stop swatch ids ([`param_grad_swatch_id`]).
    pub name: &'static str,
    /// English label (from the `ParamUiHint`).
    pub label: String,
    /// The current serialized gradient (the text-param override, else empty → the
    /// panel opens on the default black→white ramp).
    pub value: String,
}

/// A free-text row editing a **text param** (a `motion.expression` formula) — the
/// shared single-line `TextInput` widget. The value is a `String` (not a number), so it
/// rides a dedicated [`MotionParamIntent::SetTextParam`] rather than the `f64` `SetParam`.
#[derive(Clone, Debug, PartialEq)]
pub struct TextRow {
    /// The text-param key (`Graph::set_text_param`) — echoed in the intent.
    pub name: &'static str,
    /// English label (from the `ParamUiHint`).
    pub label: String,
    /// The current formula (the text-param override, else empty).
    pub value: String,
}

/// An angle row: a number box with a `deg` unit chip. **Degrees end to end** —
/// the app's one authored-angle unit, so the param already stores what the box
/// shows and there is nothing to convert in either direction. Whatever consumes
/// the value (a cycle-based trig, a rotation basis) converts at its own edge.
#[derive(Clone, Debug, PartialEq)]
pub struct AngleRow {
    /// Canonical `ParamSpec::name` — echoed back in [`MotionParamIntent::SetParam`].
    pub name: &'static str,
    /// English label (from the `ParamUiHint`).
    pub label: String,
    /// Current value, in degrees.
    pub deg: f64,
    pub min_deg: f64,
    pub max_deg: f64,
    pub step_deg: f64,
}

/// A random-seed row: a whole-number box plus a re-roll button. A seed has no
/// meaningful magnitude, so it is never a slider — the artist wants *another*
/// seed, not a *bigger* one.
#[derive(Clone, Debug, PartialEq)]
pub struct SeedRow {
    pub name: &'static str,
    pub label: String,
    /// Current seed (a whole number).
    pub value: f64,
    pub min: f64,
    pub max: f64,
}

/// A checkbox row for a boolean param (`>= 0.5` = on).
#[derive(Clone, Debug, PartialEq)]
pub struct ToggleRow {
    pub name: &'static str,
    pub label: String,
    pub on: bool,
}

/// A named single-select row. `selected` is the current option index (the param
/// value rounded); `labels` are the option captions painted as segmented buttons.
#[derive(Clone, Debug, PartialEq)]
pub struct EnumRow {
    pub name: &'static str,
    pub label: String,
    pub selected: usize,
    pub labels: &'static [&'static str],
}

/// A slider + numeric-chip row for one scalar `ParamSpec`.
#[derive(Clone, Debug, PartialEq)]
pub struct ScalarRow {
    /// Canonical `ParamSpec::name` — echoed back in [`MotionParamIntent::SetParam`].
    pub name: &'static str,
    /// English label (from the `ParamUiHint`, else the param name).
    pub label: String,
    /// Current value: the per-instance override if set, else the manifest default.
    pub value: f64,
    pub min: f64,
    /// The **soft** maximum: how far the slider drags.
    pub max: f64,
    /// The **hard** maximum: how far a TYPED value may go. Equals [`Self::max`]
    /// unless the node registered a `ParamHardMax` (Blender's soft/hard limits).
    /// The slider cannot represent anything above `max`, so a value up here is
    /// authored by the box and the box alone — see the params panel's paint.
    pub hard_max: f64,
    pub step: f64,
    /// The chip snaps to whole numbers (count / index / seed).
    pub integer: bool,
    /// **A wire is driving this param** (doc 58) — `value` is the live number coming down it.
    ///
    /// The row is then READ-ONLY: it paints the number and registers no widget at all. A
    /// knob that still turns under the finger while a wire decides the value is a control
    /// that lies once per drag — and dimming it would not help, because a dimmed widget
    /// still dispatches ([[feedback_disabled_button_still_dispatches]]).
    pub driven: bool,
}

/// A colour-swatch row driving four **linear-straight** RGBA channel params
/// (the canonical colour UI). `srgb` is the bridge-converted display colour
/// (linear→sRGB) the panel paints + the shell seeds the OKLCH picker with;
/// `channels` are the params a pick writes back to (sRGB→linear). The swatch's
/// widget id is [`param_swatch_id`]`(channels[0])`.
#[derive(Clone, Debug, PartialEq)]
pub struct ColorRow {
    /// English label (from the `ParamUiHint`).
    pub label: String,
    /// The four RGBA channel param names, in order — echoed to the bridge.
    pub channels: [&'static str; 4],
    /// sRGB8 (straight) for painting the swatch + seeding the picker.
    pub srgb: [u8; 4],
}

/// The selected node's params, resolved for the panel (M1.P1).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParamsSnapshot {
    /// `NodeId.0` of the selected node (echoed in the intent so a stale edit
    /// never lands on a different node).
    pub node: u32,
    /// The node's display name (panel header).
    pub title: String,
    pub rows: Vec<ParamRow>,
}

/// A param edit the panel asks the shell to apply (M1.P1). Tagged with the node
/// id + canonical param name so it is unambiguous even if the selection changed
/// between the edit and the drain.
#[derive(Clone, Debug, PartialEq)]
pub enum MotionParamIntent {
    SetParam {
        node: u32,
        param: &'static str,
        value: f64,
    },
    /// A **text** param edit (a formula) — carries a `String` (the `f64` `SetParam` cannot).
    /// The bridge applies it via `Graph::set_text_param`.
    SetTextParam {
        node: u32,
        param: &'static str,
        value: String,
    },
}

thread_local! {
    static CURRENT: RefCell<Option<ParamsSnapshot>> = const { RefCell::new(None) };
    static INTENTS: RefCell<Vec<MotionParamIntent>> = const { RefCell::new(Vec::new()) };
}

/// Publish the selected node's params (shell bridge → panel). `None` when no
/// single node is selected or the Motion tool is inactive.
pub fn set_current_params(snapshot: Option<ParamsSnapshot>) {
    CURRENT.with(|c| *c.borrow_mut() = snapshot);
}

/// Read the published params (panel `paint` / `apply_event`).
pub(crate) fn current_params() -> Option<ParamsSnapshot> {
    CURRENT.with(|c| c.borrow().clone())
}

/// Queue a param edit for the bridge to apply (panel → shell).
pub(crate) fn push_param_intent(intent: MotionParamIntent) {
    INTENTS.with(|c| c.borrow_mut().push(intent));
}

/// Drain the queued param edits (shell bridge, each frame). Capacity-retaining.
pub fn drain_param_intents() -> Vec<MotionParamIntent> {
    INTENTS.with(|c| std::mem::take(&mut *c.borrow_mut()))
}

/// Max param rows the pooled slider/chip widgets support (grid/transform/clone
/// have 3; the ceiling covers the fan-out nodes without a per-node id scheme).
pub(crate) const MAX_PARAM_ROWS: usize = 8;

/// Max named options a single `Enum` row's segmented selector supports (covers
/// the behaviours' channel / wave / easing sets with headroom).
pub(crate) const MAX_ENUM_OPTIONS: usize = 8;

/// Option-id base for a [`ChannelsRow`]'s live-column chips (the Custom picker).
/// Well above `MAX_ENUM_OPTIONS` so a chip's `param_enum_id(slot, BASE + j)` never
/// collides with a curated segment's `param_enum_id(slot, opt)` (`opt < 9`).
pub(crate) const CHANNELS_EXTRA_BASE: usize = 32;

/// Max control points a single Curve row's editor supports (matches the field.remap
/// text param's practical ceiling; a handful of points shape any transfer). The
/// per-point `CurvePoint` widgets are pooled positionally like the enum options.
pub(crate) const MAX_CURVE_POINTS: usize = 8;

/// Max stops a single Gradient row's editor offers (doc 85). The model
/// (`ph2d_color::MAX_RAMP_STOPS`) allows 32, but the panel is narrow and the swatch
/// strip must stay legible — `+` refuses beyond this, the display ceiling. The
/// per-stop `CurvePoint` markers are registered per-paint like the Curve handles.
pub(crate) const MAX_GRADIENT_STOPS: usize = 8;

/// Stable widget id for the `slot`-th param row's slider (pooled, positional —
/// row `i` of whichever node is selected uses slot `i`).
pub(crate) fn param_slider_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/slider/{slot}"))
}

/// Stable widget id for the `slot`-th param row's numeric chip.
pub(crate) fn param_chip_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/chip/{slot}"))
}

/// Stable widget id for a colour-swatch row, keyed by its **anchor channel**
/// param name (unique within a node) — NOT positional like the slider/chip pool,
/// so the shell bridge computes the same id from the node's hints without
/// agreeing on row order. `pub` for the bridge's picker read-back / seeding.
pub fn param_swatch_id(anchor: &str) -> NodeId {
    fnv_id(&format!("motion_param/swatch/{anchor}"))
}

/// Stable widget id for the `slot`-th param row's checkbox (Toggle rows).
pub(crate) fn param_checkbox_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/check/{slot}"))
}

/// Stable widget id for option `opt` of the `slot`-th param row's segmented
/// selector (Enum rows).
pub(crate) fn param_enum_id(slot: usize, opt: usize) -> NodeId {
    fnv_id(&format!("motion_param/enum/{slot}/{opt}"))
}

/// Stable widget id for the `slot`-th param row's standalone numeric box — the
/// app-standard `NumberInput` (Angle + Seed rows; Scalar rows use the slider's
/// own chip instead).
pub(crate) fn param_number_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/number/{slot}"))
}

/// Stable widget id for the `slot`-th Seed row's re-roll button.
pub(crate) fn param_reroll_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/reroll/{slot}"))
}

/// Stable widget id for the `slot`-th Text row's `TextInput` field (formula editor).
pub(crate) fn param_text_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/text/{slot}"))
}

/// The `slot`-th Curve row's **editor parent** id — the `CurvePoint.parent` every
/// handle carries, so `apply_event` routes the drained drag to the right row (the
/// dispatch emits `ValueChanged(parent)` on a handle drag).
pub(crate) fn param_curve_editor_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/curve/{slot}"))
}

/// The `slot`-th Curve row's `point`-th draggable control-point handle.
pub(crate) fn param_curve_point_id(slot: usize, point: usize) -> NodeId {
    fnv_id(&format!("motion_param/curve/{slot}/pt/{point}"))
}

/// The `slot`-th Curve row's **add-point** button.
pub(crate) fn param_curve_add_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/curve/{slot}/add"))
}

/// The `slot`-th Curve row's **remove-point** button.
pub(crate) fn param_curve_remove_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/curve/{slot}/remove"))
}

/// The `slot`-th Curve row's **interp** button — cycles the selected point's
/// segment interpolation (Linear → Smooth → Hold).
pub(crate) fn param_curve_interp_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/curve/{slot}/interp"))
}

/// The `slot`-th Gradient row's **editor parent** id — the `CurvePoint.parent` every
/// position marker carries, so `apply_event` routes the drained drag to the right row.
pub(crate) fn param_grad_editor_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/grad/{slot}"))
}

/// The `slot`-th Gradient row's `stop`-th draggable position marker.
pub(crate) fn param_grad_stop_id(slot: usize, stop: usize) -> NodeId {
    fnv_id(&format!("motion_param/grad/{slot}/stop/{stop}"))
}

/// Stable widget id for a Gradient row's `stop`-th colour swatch, keyed by the text-param
/// **name** + index (NOT positional) — so the shell bridge computes the same id from the
/// node's hints to seed the swatch colour and read the OKLCH pick back into the string.
/// `pub` for the bridge, exactly like [`param_swatch_id`].
pub fn param_grad_swatch_id(name: &str, stop: usize) -> NodeId {
    fnv_id(&format!("motion_param/grad_swatch/{name}/{stop}"))
}

/// The `slot`-th Gradient row's **add-stop** button.
pub(crate) fn param_grad_add_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/grad/{slot}/add"))
}

/// The `slot`-th Gradient row's **remove-stop** button.
pub(crate) fn param_grad_remove_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/grad/{slot}/remove"))
}

/// The `slot`-th Gradient row's **interp** button — cycles the ramp's global
/// interpolation (Linear → Ease → Constant → Cardinal → B-Spline).
pub(crate) fn param_grad_interp_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/grad/{slot}/interp"))
}

/// The `slot`-th Gradient row's `p`-th **preset seed** chip (Rainbow / Heat / Ice /
/// Grayscale) — clicking it LOADS that preset's stops into the editable ramp (doc 85).
pub(crate) fn param_grad_preset_id(slot: usize, p: usize) -> NodeId {
    fnv_id(&format!("motion_param/grad/{slot}/preset/{p}"))
}

/// FNV-1a-64 of `key` (same scheme as the graph panel's dynamic hit ids).
fn fnv_id(key: &str) -> NodeId {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in key.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    NodeId(h)
}
