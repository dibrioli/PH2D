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
    pub max: f64,
    pub step: f64,
    /// The chip snaps to whole numbers (count / index / seed).
    pub integer: bool,
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

/// FNV-1a-64 of `key` (same scheme as the graph panel's dynamic hit ids).
fn fnv_id(key: &str) -> NodeId {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in key.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    NodeId(h)
}
