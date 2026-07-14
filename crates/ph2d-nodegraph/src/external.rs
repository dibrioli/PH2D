//! **External values — the door from the APP into the cook** (Motion Nodes doc 65).
//!
//! A node gets three things: its params, its inputs, and the playhead. That is deliberate — it is
//! what makes a node a pure function of the graph, and it is why the cook can memoize, scrub and
//! replay bit-exactly. But it also means a node **cannot see anything the application owns**: the
//! vector document, the selection, the mouse. And `motion.path` — "walk this drawn curve" — needs
//! exactly that: a shape the artist drew, which lives in `ph2d-vec-scene`, in the shell.
//!
//! The plan said *"integra `vector.*`"* — but that node family was **RETIRED** (ADR-0108), and the
//! geometry moved into a document the graph has no reach into. So the question stopped being *how
//! do I import a curve* and became **how does anything outside the graph get in at all**.
//!
//! ## The answer is the one this crate keeps giving
//!
//! Not a new port kind (`NodeManifest.inputs` is `&'static` — ADR-0039, frozen). Not a dependency
//! from `ph2d-nodegraph` on the vector crates (a leaf substrate that knows one domain's data type
//! is not a substrate). **A named channel on the `Cook`**, published by whoever owns the data:
//!
//! ```text
//! shell:  cook.set_external("Track", polyline_of(the_path_named_Track))
//! node:   ctx.external("Track")   // -> a Stream, or empty if nobody published one
//! ```
//!
//! It is the third turn of the same crank — the text-param channel (doc 32) and the driven-param
//! channel (doc 58) both got here by asking *"where does state that the manifest cannot describe
//! actually live?"* and answering **"beside it, not inside it"**.
//!
//! ## The memo has to see it, and that is the whole difficulty
//!
//! An external is an INPUT the fingerprint does not know about. Edit the curve, and every node
//! downstream must recompute — but the cook decides whether to recompute *before* it evaluates,
//! and it only learns which external a node read *during* evaluation. Chicken and egg.
//!
//! So the cook **remembers what the node read last time** (`Cook` keeps the names beside the memo)
//! and folds *those* names' current revisions into the fingerprint. If the node has never cooked,
//! the field is 0 and it cooks. If it starts reading a *different* external, the text param that
//! names it changed — and that is already in the fingerprint. Precise, and it costs one `Vec<String>`
//! per cached node.
//!
//! The revision is the **content**: `set_external` hashes the stream, so a caller cannot get the
//! bookkeeping wrong by forgetting to bump a counter. Publishing the same curve twice is free.

use crate::attr::{Column, Stream};
use std::collections::BTreeMap;

/// One published value, and the revision that IS its content.
#[derive(Clone, Debug, PartialEq)]
pub struct External {
    /// FNV-1a over the stream's columns. Two identical publishes have the same revision, so a
    /// shell that republishes every frame (which is the simple thing to do) invalidates nothing.
    pub rev: u64,
    pub value: Stream,
}

/// Everything the app has published, by name.
pub type All = BTreeMap<String, External>;

/// FNV-1a over a stream's shape and contents.
///
/// It walks the columns in `BTreeMap` order, so the hash is deterministic — the same curve, on any
/// machine, is the same revision.
///
/// **Every column kind is hashed, by value.** A hash that skipped one would make a change to it
/// read as *unchanged*, and the memo would hand back the pre-edit curve forever — the exact bug the
/// driven-param fingerprint had to be rescued from (doc 58 §4). The `f32`s go in as BITS: `NaN` is
/// not equal to itself and `-0.0 == 0.0`, and a revision has to be a bitwise fact.
pub fn fingerprint(s: &Stream) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |bytes: &[u8]| {
        for b in bytes {
            h ^= *b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    mix(&(s.count() as u64).to_le_bytes());
    for (name, col) in s.columns() {
        mix(name.as_bytes());
        let mut floats = |xs: &[f32]| {
            for x in xs {
                mix(&x.to_bits().to_le_bytes());
            }
        };
        match col {
            Column::Scalar(v) => floats(v),
            Column::Vec2(v) => v.iter().for_each(|p| floats(p)),
            Column::Vec3(v) => v.iter().for_each(|p| floats(p)),
            Column::Vec4(v) => v.iter().for_each(|p| floats(p)),
        }
    }
    h
}

/// The revisions of the externals `names` refer to, folded into one number for the fingerprint. A
/// name nobody published contributes its absence (not zero — *absence*), so a curve that appears or
/// disappears recomputes the nodes that were asking for it.
pub fn revs_of(all: &All, names: &[String]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |bytes: &[u8]| {
        for b in bytes {
            h ^= *b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    for n in names {
        mix(n.as_bytes());
        match all.get(n) {
            Some(e) => mix(&e.rev.to_le_bytes()),
            None => mix(b"<absent>"),
        }
    }
    h
}
