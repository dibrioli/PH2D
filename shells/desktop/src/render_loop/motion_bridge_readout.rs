//! **What each card is doing right now** (Motion Nodes F2 readouts + F3 flow) — the number it
//! produced this frame, how much it carries, and whether that changed. Declared by
//! `motion_bridge` as a `#[path]` sibling, so `super` is `render_loop::motion_bridge`.
//!
//! ## Why this is free, and why it must be
//!
//! The frame's cook has already evaluated every node that feeds a sink; the results are
//! sitting in the pump's memo. So a readout is a **lookup** (`Cook::peek`), never a cook.
//!
//! The tempting alternative — `cook()` each card — is *correct* and still wrong: it would
//! evaluate nodes the render never needed, once per card per frame, and turn glancing at
//! the graph into a second full evaluation of it. An 80-node document would pay for it every
//! frame, forever, so that some cards could show a number.
//!
//! ## Blank is the most useful reading there is
//!
//! A node the cook never pulled has no memo entry, and gets **no readout at all**. That is
//! not a gap — it is the diagnosis: *nothing downstream consumes this card*. A chain the
//! artist forgot to wire into the Output, a branch orphaned by the knife, a node just
//! dropped from the menu: all blank, and the blankness says exactly why the canvas did not
//! change. (This is the module's own "unit-green ≠ alive" alarm, made visible. The panel
//! draws the VEIL from reachability, not from this blank — see `flow::live_set`.)

use super::MotionState;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::graph::NodeId;
use ph2d_nodegraph::value::CookValue;

/// How many characters a readout may take on a card — a card is 190 units wide and the
/// readout shares the row with nothing, but a stream of a million instances must not push
/// the text past the border.
const MAX_LEN: usize = 16;

/// How many elements of a column the digest looks at. The digest answers *"did this change
/// since last frame?"* for a wire animation — it is not a checksum, and reading 10 000
/// instances × every column × every node, every frame, to decide whether to draw dashes would
/// cost more than the cook it is reporting on.
///
/// The honest limit: a change confined entirely to instances the stride skips reads as still.
/// Animation moves populations, not needles in haystacks, so this is the right trade — but it
/// IS a trade, and this is where it is written down.
const DIGEST_SAMPLES: usize = 48;

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

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

/// **Did this node's output change?** — a digest of what it emitted, compared frame to frame.
///
/// It hashes the element COUNT *and the VALUES*. Count alone would be worthless for exactly
/// the case that matters: a grid of 400 instances being moved by an oscillator is 400
/// instances every frame, and it is the most alive thing on the canvas.
fn digest_of(outputs: &[CookValue]) -> u64 {
    let mut h = FNV_OFFSET;
    for value in outputs {
        digest_stream(&mut h, value.as_stream());
    }
    h
}

fn digest_stream(h: &mut u64, stream: &Stream) {
    let n = stream.count();
    fold(h, n as u64);
    for (name, col) in stream.columns() {
        for b in name.as_bytes() {
            fold(h, *b as u64);
        }
        // Stride so the cost is bounded by DIGEST_SAMPLES, whatever the stream's size.
        let step = n.div_ceil(DIGEST_SAMPLES).max(1);
        let mut i = 0;
        while i < n {
            match col {
                Column::Scalar(v) => fold(h, v[i].to_bits() as u64),
                Column::Vec2(v) => v[i].iter().for_each(|f| fold(h, f.to_bits() as u64)),
                Column::Vec3(v) => v[i].iter().for_each(|f| fold(h, f.to_bits() as u64)),
                Column::Vec4(v) => v[i].iter().for_each(|f| fold(h, f.to_bits() as u64)),
            }
            i += step;
        }
    }
}

fn fold(h: &mut u64, x: u64) {
    *h = (*h ^ x).wrapping_mul(FNV_PRIME);
}

/// Stamp every card with what it produced this frame: its readout, the MASS of its stream (the
/// wire's width), whether the value CHANGED since last frame (the wire's march), and whether it
/// is a sink (where the panel's reachability walk starts).
///
/// Called with the snapshot the panel is about to receive, right after the cook that filled
/// the memo.
pub(super) fn stamp(
    motion: &mut MotionState,
    snap: &mut ph2d_panel_motion_graph::GraphViewSnapshot,
) {
    for node in &mut snap.nodes {
        let id = NodeId(node.id);
        let cooked = motion.pump.cook.peek(id);
        node.readout = cooked.and_then(readout_of);
        node.count = cooked
            .and_then(|o| o.first())
            .map(|v| v.as_stream().count() as u32);
        node.is_sink = motion.sinks.contains(&id);

        // A node the cook never pulled is NEVER hot — no data flows through a wire nothing
        // consumes, and a dead branch flickering with dashes would be the loudest lie on the
        // canvas. (It also means the digest map holds only cooked nodes.)
        node.hot = match cooked.map(digest_of) {
            Some(now) => {
                let before = motion.flow_digest.insert(node.id, now);
                before.is_some_and(|b| b != now)
            }
            None => {
                motion.flow_digest.remove(&node.id);
                false
            }
        };
    }
    // A DELETED node's digest would otherwise sit in the map for the rest of the session.
    // Zero-alloc (a linear scan of a document-sized list, once a frame).
    motion
        .flow_digest
        .retain(|id, _| snap.nodes.iter().any(|n| n.id == *id));
}

#[cfg(test)]
#[path = "motion_bridge_readout_tests.rs"]
mod tests;
