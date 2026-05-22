//! Algebraic port types — the typed vocabulary every node connects through.
//!
//! A port type is **not** just a value kind. Per ADR-0030/0032 it carries
//! three axes, and two ports may connect by a plain edge only when all three
//! match. Crossing a clock or domain boundary is a typed membrane traversal
//! (a `pre`/delay or a designated export), never a silent edge. The `clock`
//! axis is the load-bearing one: it is what prevents a 60 Hz field from
//! silently feeding a 48 kHz synth (ADR-0032 leak B).

/// Which data family flows on the edge (ADR-0030 §1.2).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Domain {
    /// Ephemeral per-instance stream (motion / geometry). Lowers to GPU
    /// instancing or vector. No identity, **not** ECS components (ADR-0035).
    Instances,
    /// Per-element field sampled on a grid/pixel (shader / SDF). Lowers to WGSL.
    Field,
    /// Time-sampled audio signal (sound). Lowers to a DSP graph.
    Signal,
    /// Control / dataflow for gameplay logic. Lowers to Luau (deterministic).
    Control,
}

/// Dimensionality of the value carried per element.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Dim {
    Scalar,
    Vec2,
    Vec3,
    Vec4,
    Mat2,
    Mat3,
    Mat4,
}

/// The sampling clock — two ports under different clocks cannot connect by a
/// plain edge; the crossing is a membrane traversal.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Clock {
    /// Cooked once, never re-evaluated at runtime (static subgraph → asset).
    Static,
    /// Pull-sampled once per render frame (~60 Hz): motion, shader.
    Frame,
    /// Synchronous-dataflow audio block rate (e.g. 48 kHz): sound.
    Audio,
    /// Discrete push event: gameplay messages, input.
    Event,
}

/// A fully-resolved port type: `(domain, dim, clock)`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PortType {
    pub domain: Domain,
    pub dim: Dim,
    pub clock: Clock,
}

impl PortType {
    pub const fn new(domain: Domain, dim: Dim, clock: Clock) -> Self {
        Self { domain, dim, clock }
    }

    /// Two ports connect directly inside one region iff every axis matches.
    /// A mismatched clock/domain is legal only across a membrane-crossing
    /// node, never a plain edge. This is the type-level encoding of the
    /// ADR-0030 pocket rule ("same scheduler, same clock, same lowering
    /// target?"). Dimensionality is strict in v1 (broadcasting is a later
    /// refinement).
    pub fn connects_directly(self, consumer: PortType) -> bool {
        self.domain == consumer.domain
            && self.dim == consumer.dim
            && self.clock == consumer.clock
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_type_connects() {
        let a = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
        assert!(a.connects_directly(a));
    }

    #[test]
    fn different_clock_does_not_connect_directly() {
        // The leak-B case: same value shape, different sampling clock.
        let frame = PortType::new(Domain::Field, Dim::Scalar, Clock::Frame);
        let audio = PortType::new(Domain::Field, Dim::Scalar, Clock::Audio);
        assert!(!frame.connects_directly(audio));
    }

    #[test]
    fn different_domain_does_not_connect_directly() {
        let inst = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
        let field = PortType::new(Domain::Field, Dim::Vec2, Clock::Frame);
        assert!(!inst.connects_directly(field));
    }
}
