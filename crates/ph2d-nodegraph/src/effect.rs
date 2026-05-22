//! Effect classes — the type-checked membrane (ADR-0030 / ADR-0032).
//!
//! Every node declares one effect class. The membrane is a single rule
//! ([`Effect::can_feed`]): a presentation node (`Pure` / `Temporal`, the pull
//! side) must never depend **directly** on a `Stateful` node (the push side
//! that writes the `SimWorld`). That crossing must go through a designated
//! membrane export, not a plain edge. The rule is *checked* — not just trusted
//! — by [`crate::graph::Graph::validate`] (see `tests/validate.rs`). The CI
//! arch-gate that runs `validate` on every authored graph lands with the
//! evaluators (W2.T3, `docs/plans/2026-05-node-waves.md`); until then `validate`
//! is the enforcement point, callable by any consumer. Only the `Stateful`
//! (gameplay) side is bound by determinism (HR-5); presentation is exempt, like
//! Radiance Cascades.

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Effect {
    /// No state, no time: same inputs → same output. Freely cacheable.
    Pure,
    /// Reads the playhead/clock but holds no mutable state (e.g. an
    /// oscillator). Still pull-side / presentation.
    Temporal,
    /// Mutates simulated state (gameplay/logic, push side). Deterministic
    /// (HR-5); writes the `SimWorld`.
    Stateful,
}

impl Effect {
    /// Is this node on the pull (presentation) side of the membrane?
    pub fn is_pull_side(self) -> bool {
        matches!(self, Effect::Pure | Effect::Temporal)
    }

    /// May a `self`-effect producer feed a `consumer`-effect node by a plain
    /// edge? The one forbidden case is push→pull (a `Stateful` node feeding a
    /// presentation node): that must cross the membrane via an export node,
    /// never a direct edge. Everything else is allowed.
    pub fn can_feed(self, consumer: Effect) -> bool {
        !(self == Effect::Stateful && consumer.is_pull_side())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stateful_cannot_feed_presentation_directly() {
        assert!(!Effect::Stateful.can_feed(Effect::Pure));
        assert!(!Effect::Stateful.can_feed(Effect::Temporal));
    }

    #[test]
    fn presentation_and_stateful_chains_are_allowed() {
        assert!(Effect::Pure.can_feed(Effect::Temporal));
        assert!(Effect::Temporal.can_feed(Effect::Pure));
        assert!(Effect::Stateful.can_feed(Effect::Stateful));
        // A presentation node may read a stateful one only via the export
        // (the reverse direction, pull reading an exported value) — that is
        // modelled as Pure/Temporal feeding, which is allowed:
        assert!(Effect::Pure.can_feed(Effect::Stateful));
    }
}
