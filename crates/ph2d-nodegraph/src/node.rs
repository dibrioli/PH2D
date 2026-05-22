//! The node contract: what an agent implementing a node crate fills in.
//!
//! A node is an FBP black box (ADR-0031): its **only** contract is its ports +
//! effect + clock. It sees nothing of the rest of the graph. That is exactly
//! what makes a node implementable by a parallel agent in isolation, and
//! testable on its own (golden input → golden output).

use crate::cook::EvalCtx;
use crate::effect::Effect;
use crate::port::{Clock, PortType};

/// Stable, deterministic id derived from a node type's canonical name via
/// FNV-1a 64-bit. Uses the same FNV-1a algorithm/constants as
/// `ph2d-tool-registry::hash_node_id` (reimplemented dependency-free so the
/// substrate stays leaf-level), but **without** that function's `NodeId(0)`
/// reservation, which is specific to the a11y root. Const so it can be
/// computed in a `const MANIFEST`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeTypeId(pub u64);

impl NodeTypeId {
    pub const fn of(name: &str) -> Self {
        Self(fnv1a_64(name.as_bytes()))
    }
}

const fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        i += 1;
    }
    hash
}

/// One named, typed port on a node.
#[derive(Copy, Clone, Debug)]
pub struct PortSpec {
    pub name: &'static str,
    pub ty: PortType,
}

/// A static node parameter — a named scalar constant across the stream (e.g.
/// `rows`, `seed`), with a default. A node reads its live value in
/// `NodeOp::eval` via [`crate::cook::EvalCtx::param`] (the graph's per-instance
/// override if set, else this `default`); `ph2d-expr` reads it via `Expr::Param`
/// through the same binding. (Vector/enum params are a later extension; scalar
/// `f32` matches `ph2d-expr`'s value type.)
#[derive(Copy, Clone, Debug)]
pub struct ParamSpec {
    pub name: &'static str,
    pub default: f32,
}

/// A safety ceiling for turning an *untrusted* `f32` parameter into an
/// allocation size ([`param_as_count`]). Streams are `Vec`-backed SoA
/// everywhere, so an out-of-range count is an allocation-overflow / OOM vector.
/// `1 << 24` (~16.7M elements) is well above the 100k-instance target (MEMORY
/// M5) yet far below where `count * count` overflows `usize`. A domain may pass
/// its own budget; this is the recommended default.
pub const RECOMMENDED_MAX_ELEMENTS: usize = 1 << 24;

/// Interpret an `f32` parameter as a non-negative element count, **totally**:
/// non-finite (`NaN`/`±∞`) and negative values map to `0`, fractional values
/// floor, and the result is clamped to `max`. This mirrors `ph2d-expr`'s
/// totality discipline (a bad value degrades to a defined number, never a
/// panic or a silent `as usize` overflow) — necessary because [`NodeOp::eval`]
/// cannot return a `Result`. Use it (with a per-element-product guard, e.g.
/// [`usize::saturating_mul`] then `.min(max)`) wherever a param drives a
/// `Vec::with_capacity` / element count, so a corrupt scene value can never
/// trigger a capacity-overflow abort.
pub fn param_as_count(value: f32, max: usize) -> usize {
    if value.is_finite() && value >= 0.0 {
        // The `f32 -> u64` cast is *saturating* in Rust (no UB), so a value far
        // beyond `u64` — e.g. `1e30` — becomes `u64::MAX`, and `.min(max)` then
        // clamps it to the budget. For in-range values (`< 2^24 <= 2^53`) the
        // floored f32 is exactly representable, so the count is exact.
        (value.floor() as u64).min(max as u64) as usize
    } else {
        0
    }
}

/// Which backends a node type can lower to (ADR-0033/0034). Every node supports
/// [`LoweringKind::Cpu`] (its [`NodeOp::eval`]); a node whose math is expressed
/// via `ph2d-expr` also lists `Wgsl` (GPU) and/or `Luau` (gameplay). The domain
/// evaluator picks the lowering it can run.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LoweringKind {
    Cpu,
    Wgsl,
    Luau,
}

/// The static descriptor of a node **type** (not an instance). Declared as a
/// `const MANIFEST` in each node crate and registered by codegen (ADR-0031).
/// The full contract is `(ports, effect, clock, params, lowerings)`.
#[derive(Clone, Debug)]
pub struct NodeManifest {
    pub id: NodeTypeId,
    /// Canonical name, e.g. `"motion.clone"`. Stable; the id is its hash.
    pub name: &'static str,
    pub inputs: &'static [PortSpec],
    pub outputs: &'static [PortSpec],
    pub effect: Effect,
    /// The clock this node cooks under. Must match its ports' clock unless it
    /// is a membrane-crossing node.
    pub clock: Clock,
    /// Static parameters (constant across the stream).
    pub params: &'static [ParamSpec],
    /// Backends this node can lower to. Always includes [`LoweringKind::Cpu`].
    pub lowerings: &'static [LoweringKind],
}

impl NodeManifest {
    /// The declared default of parameter `name`, or `None` if this node type has
    /// no such parameter. A node's `eval` does not call this directly — it reads
    /// the live value (per-instance override else this default) via
    /// [`crate::cook::EvalCtx::param`]; this is the resolver `EvalCtx::param`
    /// falls back to, and the way the cook/validate paths look up a default.
    ///
    /// Returning `Option` (rather than a silent fallback) makes a *misspelled*
    /// param name — a programmer error, since the name is a literal constant of
    /// the node's own crate — a checkable error: `EvalCtx::param` panics on an
    /// undeclared name (caught by the node's golden test) instead of silently
    /// reading `0.0` in production (gold-standard rule: no silent failure).
    pub fn param_default(&self, name: &str) -> Option<f32> {
        self.params
            .iter()
            .find(|p| p.name == name)
            .map(|p| p.default)
    }
}

/// Implemented by every concrete node type. `manifest` is the static contract;
/// `eval` is the pure cook step — it reads typed inputs and emits typed
/// outputs through the [`EvalCtx`], seeing nothing of the rest of the graph
/// (FBP black box, ADR-0031).
pub trait NodeOp: Send + Sync {
    fn manifest(&self) -> &'static NodeManifest;
    fn eval(&self, ctx: &mut EvalCtx<'_>);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_is_stable_and_deterministic() {
        // Same name → same id across runs (FNV-1a is pure).
        assert_eq!(
            NodeTypeId::of("motion.clone"),
            NodeTypeId::of("motion.clone")
        );
        assert_ne!(
            NodeTypeId::of("motion.clone"),
            NodeTypeId::of("motion.grid")
        );
    }

    #[test]
    fn fnv1a_known_vector() {
        // FNV-1a of empty input is the offset basis.
        assert_eq!(fnv1a_64(b""), 0xcbf2_9ce4_8422_2325);
    }

    static PARAMMED: NodeManifest = NodeManifest {
        id: NodeTypeId::of("test.parammed"),
        name: "test.parammed",
        inputs: &[],
        outputs: &[],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[ParamSpec {
            name: "rows",
            default: 3.0,
        }],
        lowerings: &[LoweringKind::Cpu],
    };

    #[test]
    fn param_default_finds_declared_and_rejects_unknown() {
        assert_eq!(PARAMMED.param_default("rows"), Some(3.0));
        // A misspelled name is None (the call site `.expect`s it) — never a
        // silent 0.0.
        assert_eq!(PARAMMED.param_default("rowz"), None);
    }

    #[test]
    fn param_as_count_is_total_over_pathological_floats() {
        let max = RECOMMENDED_MAX_ELEMENTS;
        // happy: floors fractional, passes integral.
        assert_eq!(param_as_count(3.0, max), 3);
        assert_eq!(param_as_count(2.9, max), 2);
        // non-finite and negative degrade to 0 (no panic, no garbage).
        assert_eq!(param_as_count(f32::NAN, max), 0);
        assert_eq!(param_as_count(f32::INFINITY, max), 0);
        assert_eq!(param_as_count(f32::NEG_INFINITY, max), 0);
        assert_eq!(param_as_count(-5.0, max), 0);
        // huge finite values clamp to the budget instead of saturating to a
        // `usize` that would overflow a `Vec` allocation.
        assert_eq!(param_as_count(1e30, max), max);
        assert_eq!(param_as_count(max as f32 * 4.0, max), max);
    }
}
