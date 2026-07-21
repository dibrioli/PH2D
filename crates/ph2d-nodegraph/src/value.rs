//! The value carried on one node output port — the currency of the cook.
//!
//! Motion / field edges carry an instance [`Stream`] (columnar SoA, the
//! original substrate value). A **geometry** edge (domain `vector`) carries a
//! rich, variable-topology value (a `VectorNetwork`) that does **not**
//! decompose into per-element columns. Rather than teach the leaf substrate
//! about any one domain's data type — which would force `ph2d-nodegraph` (zero
//! deps) to depend on `ph2d-vector-doc` and pollute the value model shared by
//! motion/field/signal — the substrate carries such values **type-erased**
//! behind `Arc<dyn Any>`. The domain layer (e.g. `ph2d-vector-graph`)
//! downcasts at the edge. This is the Houdini/USD pattern: a type-erased
//! payload handle with typed accessors at the domain boundary.
//!
//! Decided in **ADR-0058-amendment-1**. The frozen node-author contract
//! (`NodeOp` / `OpResolver` / `NodeManifest`, gate `architecture_contract_surface`)
//! is untouched: `CookValue` is substrate-internal cook plumbing, not a node
//! crate's surface. `Arc` keeps the cook's clone-per-tick snapshot model cheap
//! (a refcount bump, never a deep copy of geometry).

use crate::attr::Stream;
use std::any::Any;
use std::sync::Arc;

/// The empty instance stream, returned by [`CookValue::as_stream`] for a value
/// that is not an instance stream (so motion consumers read it uniformly,
/// getting `None` for every column — the same as an unconnected input).
static EMPTY_STREAM: Stream = Stream::empty();

/// What flows on one node output port (and is read on the consumer's input).
///
/// - `Empty` — an unconnected input (or a not-yet-cooked output).
/// - `Instances` — a columnar instance [`Stream`] (motion / field).
/// - `Opaque` — a domain-specific rich value behind `Arc<dyn Any>` (geometry /
///   `VectorNetwork`), downcast by the domain layer.
#[derive(Clone, Default)]
pub enum CookValue {
    #[default]
    Empty,
    Instances(Stream),
    Opaque(Arc<dyn Any + Send + Sync>),
}

impl CookValue {
    /// The value's stream bytes ([`Stream::approx_bytes`]); an `Opaque` payload
    /// is type-erased and counts 0 — a budget that cannot see a cost must not
    /// invent one, and no motion checkpoint carries opaque state today.
    pub fn approx_bytes(&self) -> usize {
        match self {
            CookValue::Instances(s) => s.approx_bytes(),
            CookValue::Empty | CookValue::Opaque(_) => 0,
        }
    }
}

impl std::fmt::Debug for CookValue {
    /// An `Opaque` payload is type-erased, so it prints as a placeholder — the
    /// substrate cannot know the domain type. `Empty`/`Instances` print fully.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CookValue::Empty => write!(f, "CookValue::Empty"),
            CookValue::Instances(s) => f.debug_tuple("CookValue::Instances").field(s).finish(),
            CookValue::Opaque(_) => write!(f, "CookValue::Opaque(<dyn Any>)"),
        }
    }
}

impl CookValue {
    /// View as an instance stream. An `Empty`/`Opaque` value reads as the empty
    /// stream — so a motion node/consumer can call `ctx.input(p)` and read its
    /// columns without first matching on the value kind (a wrongly-typed edge
    /// is barred upstream by `PortType` domain checking, ADR-0030).
    pub fn as_stream(&self) -> &Stream {
        match self {
            CookValue::Instances(s) => s,
            CookValue::Empty | CookValue::Opaque(_) => &EMPTY_STREAM,
        }
    }

    /// The type-erased opaque payload, if this is an `Opaque` value. The domain
    /// layer downcasts it (e.g. `ph2d-vector-graph` to `&VectorNetwork`).
    pub fn as_any(&self) -> Option<&(dyn Any + Send + Sync)> {
        match self {
            CookValue::Opaque(a) => Some(a.as_ref()),
            CookValue::Empty | CookValue::Instances(_) => None,
        }
    }

    /// Whether this output carries nothing meaningful: `Empty`, or an instance
    /// stream with zero elements. An `Opaque` value is never empty (it holds a
    /// geometry payload by construction).
    pub fn is_empty(&self) -> bool {
        match self {
            CookValue::Empty => true,
            CookValue::Instances(s) => s.is_empty(),
            CookValue::Opaque(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attr::Column;

    #[test]
    fn empty_and_opaque_read_as_the_empty_stream() {
        assert!(CookValue::Empty.as_stream().is_empty());
        let v = CookValue::Opaque(Arc::new(42u32));
        assert!(v.as_stream().is_empty());
        // ...and an opaque value exposes no columns to a motion consumer.
        assert!(v.as_stream().get("P").is_none());
    }

    #[test]
    fn instances_round_trip_as_stream() {
        let s = Stream::new(1).with("P", Column::Vec2(vec![[1.0, 2.0]]));
        let v = CookValue::Instances(s);
        assert_eq!(v.as_stream().count(), 1);
        assert!(v.as_any().is_none());
        assert!(!v.is_empty());
    }

    #[test]
    fn opaque_downcasts_at_the_boundary() {
        let v = CookValue::Opaque(Arc::new(7i64));
        let got = v.as_any().and_then(|a| a.downcast_ref::<i64>());
        assert_eq!(got, Some(&7));
        // wrong type → None, never a silent reinterpret.
        assert!(v.as_any().and_then(|a| a.downcast_ref::<u8>()).is_none());
    }

    #[test]
    fn default_is_empty() {
        assert!(matches!(CookValue::default(), CookValue::Empty));
        assert!(CookValue::default().is_empty());
    }
}
