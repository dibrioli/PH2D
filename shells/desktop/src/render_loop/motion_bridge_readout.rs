//! **Inline readouts** (Motion Nodes F2) — the number each node produced this frame,
//! stamped onto its own card. Declared by `motion_bridge` as a `#[path]` sibling, so
//! `super` is `render_loop::motion_bridge`.
//!
//! ## Why this is free, and why it must be
//!
//! The frame's cook has already evaluated every node that feeds a sink; the results are
//! sitting in the pump's memo. So a readout is a **lookup** (`Cook::peek`), never a cook.
//!
//! The tempting alternative — `cook()` each card — is *correct* and still wrong: it would
//! evaluate nodes the render never needed, once per card per frame, and turn glancing at
//! the graph into a second full evaluation of it. A 79-node document would pay for it every
//! frame, forever, so that some cards could show a number.
//!
//! ## Blank is the most useful reading there is
//!
//! A node the cook never pulled has no memo entry, and gets **no readout at all**. That is
//! not a gap — it is the diagnosis: *nothing downstream consumes this card*. A chain the
//! artist forgot to wire into the Output, a branch orphaned by the knife, a node just
//! dropped from the menu: all blank, and the blankness says exactly why the canvas did not
//! change. (This is the module's own "unit-green ≠ alive" alarm, made visible.)

use super::MotionState;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::graph::NodeId;
use ph2d_nodegraph::value::CookValue;

/// How many characters a readout may take on a card — a card is 190 units wide and the
/// readout shares the row with nothing, but a stream of a million instances must not push
/// the text past the border.
const MAX_LEN: usize = 16;

/// One node's reading, or `None` when this frame's cook never pulled it.
///
/// **What the number is**, mirroring the probe (F2) so the two never disagree: a VALUE
/// stream reads out its scalar (the `v` column); anything else reads out how many instances
/// it carries. Those are the two questions an artist actually asks a wire — *what is it
/// worth?* and *how many are there?* — and the readout says which one is on screen.
fn readout_of(outputs: &[CookValue]) -> Option<String> {
    let value = outputs.first()?;
    let stream = value.as_stream();
    let text = match stream.get("v") {
        Some(Column::Scalar(v)) if !v.is_empty() => format!("{:.3}", v[0]),
        // A stream with no elements is not "0 instances" in the sense of a count — it is a
        // node that produced NOTHING, and saying so plainly beats a bare zero.
        _ if stream.count() == 0 => "empty".to_string(),
        _ => format!("{} inst", stream.count()),
    };
    Some(text.chars().take(MAX_LEN).collect())
}

/// Stamp every card with what it produced this frame. Called with the snapshot the panel is
/// about to receive, right after the cook that filled the memo.
pub(super) fn stamp(motion: &MotionState, snap: &mut ph2d_panel_motion_graph::GraphViewSnapshot) {
    for node in &mut snap.nodes {
        node.readout = motion.pump.cook.peek(NodeId(node.id)).and_then(readout_of);
    }
}

#[cfg(test)]
#[path = "motion_bridge_readout_tests.rs"]
mod tests;
