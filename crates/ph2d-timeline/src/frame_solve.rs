//! The frame solver's shared state — what the blend knows about *other* channels
//! this frame, so a prop-link (`Name.prop`) can read a source that is itself faded
//! (ADR-0146). It is the `snap`+`names` the ADR-0144 post-pass built, lifted to a
//! type the blend can be handed.
//!
//! **Through W2 this is threaded EMPTY.** In W0 it is never read (the codegen of
//! `eval_frame`/`sample_stack` gains a parameter while the IEEE-754 arithmetic is
//! untouched — the fade fingerprint `tests/fade_fingerprint.rs` stays byte-for-byte); in
//! W1 a per-clip expression reads it (via [`eval_expr`]) but every `Name.prop` link
//! resolves to 0 because the map is empty. W3 fills this module with the two-phase
//! scheduler that BUILDS the map (the retired `expr_pass` machinery:
//! `collect_links`/`resolve_link`/`topo_order`), and a prop-link finally reads a value.
//!
//! An empty [`LinkFrame`] allocates nothing — an empty `BTreeMap` never touches the
//! heap until its first insert — so a formula-free apply carries one at zero cost
//! (HR-3; gate `no_expression_allocates_no_link_frame`).

use std::collections::BTreeMap;

use ph2d_ecs::stable_name_id;
use ph2d_expr::{Bindings, Expr, eval};

use crate::prop::PropKind;

/// What the blend knows about other channels this frame, for prop-links (ADR-0146).
///
/// Empty on the formula-free path (the common case), where it is never read: an
/// empty `BTreeMap` never allocates, so carrying one costs nothing. The blend reads it
/// through [`ExprBindings`] when a per-clip expression names a `Name.prop` link.
#[derive(Default)]
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

/// Spread between two bindings' wiggle seeds — large enough that the noise phases of
/// distinct properties never coincide (the hash decorrelates any distinct input, so this
/// only needs to keep the numbers apart). `LITERAL-PX-OK`: a seed spacing, not a UI value.
pub(crate) const SEED_SPACING: f32 = 100.0;

/// Evaluate a parsed expression `ir` as a channel's value: `value` is this channel's
/// pre-expression value (the keyed sample or rest), `time` is the strip-LOCAL clip clock,
/// `seed` is the wiggle seed, and `links` carries the faded value of any source channel a
/// `Name.prop` link reads (ADR-0146).
///
/// The evaluator is f32 (`ph2d-expr`); the blend works in f64 and casts the result back.
/// A `Name.prop` whose source is not in `links` resolves to 0.0 (the evaluator's total
/// contract) — through W2 the map is empty, so every prop-link is 0 until W3 fills it.
pub(crate) fn eval_expr(ir: &Expr, value: f64, time: f64, seed: f64, links: &LinkFrame) -> f32 {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the expression evaluator is f32; f64 is the blend's working precision"
    )]
    let bindings = ExprBindings {
        links,
        value: value as f32,
        time: time as f32,
        seed: seed as f32,
    };
    eval(ir, &bindings)
}

/// The [`Bindings`] the blend hands an expression: `time`, `value`, `__seed`, and
/// `Name.prop` prop-links resolved against the frame's [`LinkFrame`] (the faded value of
/// the source channel this sweep composed — ADR-0146). An unknown name is 0.0.
struct ExprBindings<'a> {
    links: &'a LinkFrame,
    value: f32,
    time: f32,
    seed: f32,
}

impl Bindings for ExprBindings<'_> {
    fn attr(&self, name: &str) -> f32 {
        match name {
            "time" => self.time,
            "__seed" => self.seed,
            "value" => self.value,
            #[expect(
                clippy::cast_possible_truncation,
                reason = "link values are stored f64 for the blend; the evaluator is f32"
            )]
            dotted => resolve_link(dotted, &self.links.names)
                .and_then(|k| self.links.links.get(&k).copied())
                .map(|v| v as f32)
                .unwrap_or(0.0),
        }
    }

    fn param(&self, _name: &str) -> f32 {
        0.0
    }
}

/// A `Name.prop` prop-link identifier -> the `(entity, prop)` it names, if the name
/// resolves to an animated object and the tail is a known property. `None` for a bare
/// identifier (no dot) or an unresolved name. Shared by the blend (W1) and the retired
/// post-pass's dependency collector (`expr_pass::collect_links`), so a link resolves the
/// same way wherever it is read.
pub(crate) fn resolve_link(name: &str, names: &BTreeMap<u64, u64>) -> Option<(u64, PropKind)> {
    let (nm, pr) = name.rsplit_once('.')?;
    Some((
        *names.get(&stable_name_id(nm))?,
        PropKind::from_expr_name(pr)?,
    ))
}
