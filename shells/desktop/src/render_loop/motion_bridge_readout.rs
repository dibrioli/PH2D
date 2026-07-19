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
use std::collections::BTreeMap;

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

/// How many points a card's **postage stamp** may carry (F3). The cost of the stamps is then
/// bounded by the number of CARDS, never by the size of the streams — which is what lets them
/// be on by default. (Nuke's thumbnails render the real image, so a heavy script has to turn
/// them off or freeze them to a static frame; a scatter of a few dozen dots has no such cliff.)
///
/// It is a SUBSAMPLE, and it says so: the stamp shows the SHAPE of what a node emits, not every
/// instance of it. At 48 dots a grid still reads as a grid and a spiral as a spiral.
///
/// It was 96, and the panel drew each dot as its own fill — ~4 000 draw objects a frame across a
/// full canvas, which is where the frame rate went (doc 53). The panel now draws a whole stamp in
/// ONE path, and halving the dots halves what is left of the cost for a preview nobody counts the
/// dots of.
const PREVIEW_POINTS: usize = 48;

/// One node's reading, or `None` when this frame's cook never pulled it.
///
/// **What the number is**, mirroring the probe (F2) so the two never disagree: a VALUE
/// stream reads out its scalar (the `v` column); anything else reads out how many instances
/// it carries. Those are the two questions an artist actually asks a wire — *what is it
/// worth?* and *how many are there?* — and the readout says which one is on screen.
fn readout_of(outputs: &[CookValue]) -> Option<String> {
    let stream = outputs.first()?.as_stream();
    let n = stream.count();
    Some(readout_text(stream, n))
}

/// **What a wire is worth** — the one question the panel asks it, in the one
/// place that answers.
///
/// A VALUE stream reads out its scalar; anything else reads out how many
/// instances it carries. Those are the two questions an artist actually asks a
/// wire (*what is it worth?* / *how many are there?*).
///
/// The readout row and the probe both need this and used to decide it
/// separately, each with its own `match stream.get("v")` — under a comment
/// promising they would "never disagree", which is a promise a copy cannot keep
/// ([[feedback_two_doors_to_the_same_question_diverge]]). They differ in how they
/// PRESENT it (a formatted string vs a label and a raw `f32`), not in what it is.
pub(super) enum Reading {
    Value(f32),
    Instances(u32),
}

/// `count` is the node's TRUE element count — see [`readout_text`] for why it is
/// an argument and not `stream.count()`.
pub(super) fn reading_of(stream: &Stream, count: u32) -> Reading {
    match stream.get("v") {
        Some(Column::Scalar(v)) if !v.is_empty() => Reading::Value(v[0]),
        _ => Reading::Instances(count),
    }
}

/// The reading, given a stream and **the element count to quote**.
///
/// ⚠️ **The count is an ARGUMENT and that is the whole point.** On the CPU path it
/// is the stream's own length, because the memo holds the real thing. On the GPU
/// path the stream is a 48-row SUBSAMPLE from the tap and its length is 48
/// whatever the node emitted — quoting it would print `48 inst` for a grid of
/// four million. The exact number comes from `CookShape`, which is what the host
/// SIZED the dispatch with. Two sources, one answer, one function to write it
/// ([[feedback_two_doors_to_the_same_question_diverge]]).
fn readout_text(stream: &Stream, count: usize) -> String {
    let text = match reading_of(stream, count as u32) {
        Reading::Value(v) => format!("{v:.3}"),
        // A stream with no elements is not "0 instances" in the sense of a count — it is a
        // node that produced NOTHING, and saying so plainly beats a bare zero.
        Reading::Instances(0) => "empty".to_string(),
        Reading::Instances(n) => format!("{n} inst"),
    };
    text.chars().take(MAX_LEN).collect()
}

/// **Did this node's output change?** — a digest of what it emitted, compared frame to frame.
///
/// It hashes the element COUNT *and the VALUES*. Count alone would be worthless for exactly
/// the case that matters: a grid of 400 instances being moved by an oscillator is 400
/// instances every frame, and it is the most alive thing on the canvas.
fn digest_of(outputs: &[CookValue]) -> u64 {
    let mut h = FNV_OFFSET;
    for value in outputs {
        let stream = value.as_stream();
        let n = stream.count();
        digest_stream(&mut h, stream, n);
    }
    h
}

/// `count` is the node's TRUE element count, not the stream's length — they part
/// on the GPU path, where the stream is a 48-row subsample. Folding the sampled
/// length instead would make a population change invisible to the digest whenever
/// the sampled values happened to land the same, and the digest's one job is
/// answering *"did this change?"*.
fn digest_stream(h: &mut u64, stream: &Stream, count: usize) {
    let n = stream.count();
    fold(h, count as u64);
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

/// The card's postage stamp: up to `PREVIEW_POINTS` positions, evenly strided through the
/// stream so the SHAPE survives the subsampling (taking the FIRST 48 of a 5 000-point spiral would
/// draw the first hundredth of one turn and call it a spiral).
///
/// `None` when the node emits no positions — a VALUE node's stamp is the number it already
/// shows, and drawing an empty box under it would be a promise of a picture that is not coming.
fn preview_of(outputs: &[CookValue]) -> Option<Vec<[f32; 2]>> {
    preview_points(outputs.first()?.as_stream())
}

/// The stamp's points from a stream. Striding again over an already-tapped
/// stream is a no-op (`step == 1` at 48 rows or fewer), so the two paths share
/// this rather than one of them getting a private "already sampled" variant.
fn preview_points(stream: &Stream) -> Option<Vec<[f32; 2]>> {
    let Some(Column::Vec2(p)) = stream.get("P") else {
        return None;
    };
    if p.is_empty() {
        return None;
    }
    let step = p.len().div_ceil(PREVIEW_POINTS).max(1);
    Some(p.iter().step_by(step).copied().collect())
}

/// Stamp every card with what it produced this frame: its readout, the MASS of its stream (the
/// wire's width), whether the value CHANGED since last frame (the wire's march), and whether it
/// is a sink (where the panel's reachability walk starts).
///
/// Called with the snapshot the panel is about to receive, right after the cook that filled
/// the memo.
/// **The GPU's answer to the same question**, taken at the one place that knows
/// whether the device drove this frame.
///
/// A GPU cook does not feed the CPU memo, so every card would go blank exactly on
/// the documents worth watching. `GpuCook::tap` samples 48 strided rows per staged
/// node for a measured **+0,075 ms** on top of a 4,19 M cook — 0,5% of a 60 fps
/// frame (`bounded_readback_cost_probe`). Only when the device actually drove the
/// last cook: on the CPU path the memo already holds the real thing and tapping
/// would be a second answer to a question already answered.
///
/// It is computed HERE and handed to [`stamp`] as data rather than `stamp` taking
/// an `Option<&GpuContext>` and deciding for itself. The device is never optional
/// in the product — only in a test — so an optional-device parameter would make
/// "pass the wrong thing and the GPU readouts silently vanish" a reachable bug,
/// for the sake of a shape only tests want.
pub(super) fn take_tap(
    motion: &mut MotionState,
    gpu: &ph2d_gpu::GpuContext,
) -> Option<BTreeMap<NodeId, Stream>> {
    motion
        .gpu_live
        .then(|| motion.gpu_cook.tap(gpu, ph2d_gpu_cook::tap::TAP_SAMPLES))
        .flatten()
}

/// `tapped` is [`take_tap`]'s result — `None` on a CPU-driven frame.
///
/// ⚠️ **One frame behind — and so is the CPU path.** This runs before BOTH cooks
/// (`cook_gpu` and the pump's `advance_or_scrub_scoped` are further down
/// `dispatch`), and `Cook::peek` is an unkeyed cache read, so the memo it returns
/// is the previous frame's too. The tap therefore introduces no staleness; it
/// matches. (The note that used to sit on `count` implied the CPU readout was
/// fresh and the GPU count was the exception. It was not.)
pub(super) fn stamp(
    motion: &mut MotionState,
    tapped: Option<&BTreeMap<NodeId, Stream>>,
    snap: &mut ph2d_panel_motion_graph::GraphViewSnapshot,
) {
    for node in &mut snap.nodes {
        let id = NodeId(node.id);
        let cooked = motion.pump.cook.peek(id);
        // The exact element count — the memo's own when the CPU cooked, else the
        // sequencer's, which SIZED the dispatch with it. NEVER the tap's: see
        // `readout_text`.
        let exact = cooked
            .and_then(|o| o.first())
            .map(|v| v.as_stream().count() as u32)
            .or_else(|| motion.gpu_cook.node_count(id));
        let sampled = tapped.and_then(|t| t.get(&id));
        node.readout = match (cooked, sampled) {
            (Some(o), _) => readout_of(o),
            (None, Some(s)) => Some(readout_text(s, exact.unwrap_or(0) as usize)),
            (None, None) => None,
        };
        // The wire's mass — see `exact` above. Without it every wire flattens to
        // the same thread the moment the device takes over, and the taper — the
        // panel's one answer to "how much is moving?" — dies exactly where the
        // counts got interesting.
        node.count = exact;
        node.is_sink = motion.sinks.contains(&id);
        node.preview = match (cooked, sampled) {
            (Some(o), _) => preview_of(o),
            (None, Some(s)) => preview_points(s),
            (None, None) => None,
        };

        // A node the cook never pulled is NEVER hot — no data flows through a wire nothing
        // consumes, and a dead branch flickering with dashes would be the loudest lie on the
        // canvas. (It also means the digest map holds only cooked nodes.)
        //
        // ⚠️ The two paths index their samples differently (the CPU strides by
        // `n.div_ceil(48)` from 0, the tap by `i * n / 48`), so switching between
        // them mid-session reads as ONE hot frame. That is a dash animation
        // blinking once on a route change, and paying for it would mean making
        // one sampler imitate the other's rounding for no gain.
        let now = match (cooked, sampled) {
            (Some(o), _) => Some(digest_of(o)),
            (None, Some(s)) => {
                let mut h = FNV_OFFSET;
                digest_stream(&mut h, s, exact.unwrap_or(0) as usize);
                Some(h)
            }
            (None, None) => None,
        };
        node.hot = match now {
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
