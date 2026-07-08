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

/// One editable param row (resolved to primitives the panel paints without
/// touching the registry / graph).
#[derive(Clone, Debug, PartialEq)]
pub struct ParamRow {
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

/// Stable widget id for the `slot`-th param row's slider (pooled, positional —
/// row `i` of whichever node is selected uses slot `i`).
pub(crate) fn param_slider_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/slider/{slot}"))
}

/// Stable widget id for the `slot`-th param row's numeric chip.
pub(crate) fn param_chip_id(slot: usize) -> NodeId {
    fnv_id(&format!("motion_param/chip/{slot}"))
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
