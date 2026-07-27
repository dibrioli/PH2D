//! The frame solver's shared state — what the blend knows about *other* channels
//! this frame, so a prop-link (`Name.prop`) can read a source that is itself faded
//! (ADR-0146). It is the `snap`+`names` the ADR-0144 post-pass built, lifted to a
//! type the blend can be handed.
//!
//! **In W0 this is threaded EMPTY through the blend and never read**, so the codegen
//! of `eval_frame`/`sample_stack` gains a parameter while the IEEE-754 arithmetic is
//! untouched — the fade fingerprint (`tests/fade_fingerprint.rs`) stays byte-for-byte.
//! W1 makes a per-clip expression a first-class lane source that reads this map; W3
//! fills this module with the two-phase scheduler that BUILDS it (the retired
//! `expr_pass` machinery: `collect_links`/`resolve_link`/`topo_order`).
//!
//! An empty [`LinkFrame`] allocates nothing — an empty `BTreeMap` never touches the
//! heap until its first insert — so a formula-free apply carries one at zero cost
//! (HR-3; gate `no_expression_allocates_no_link_frame`).

use std::collections::BTreeMap;

use crate::prop::PropKind;

/// What the blend knows about other channels this frame, for prop-links (ADR-0146).
///
/// Empty on the formula-free path (the common case), where it is never read: an
/// empty `BTreeMap` never allocates, so carrying one costs nothing.
///
/// The fields are `allow(dead_code)` only through W0, where the frame is threaded but
/// never read; W1 (the lane source) reads `links`/`names` in `eval_frame` and the allow
/// comes off.
#[derive(Default)]
#[allow(dead_code)]
pub(crate) struct LinkFrame {
    /// The composed value per `(entity, prop)` — what a `Name.prop` link reads. A
    /// source composes *before* its reader (W3's topological order) and writes its
    /// already-faded value here; the reader reads it, and its own strip fades the
    /// result again (the "double fade" of ADR-0146).
    pub links: BTreeMap<(u64, PropKind), f64>,
    /// `stable_name_id(Name)` -> entity bits, resolving the name half of a
    /// `Name.prop` link to the entity whose channel it names.
    pub names: BTreeMap<u64, u64>,
}
