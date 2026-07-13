//! The attribute stream — columnar per-element data flowing on edges.
//!
//! Houdini's attribute model is the gold standard and is what this implements:
//! a stream is not a scalar, it is a set of **named, typed columns of equal
//! length** (the element count). Modifiers read/write columns by name; a
//! cloner multiplies the element count. Stored SoA (struct-of-arrays) — cache-
//! and GPU-instancing-friendly, and the realized form of a Blender-style field
//! once cooked.
//!
//! Iteration order is deterministic (`BTreeMap`, per ADR-0022) so a cooked
//! stream hashes identically across runs.

use crate::port::Dim;
use std::collections::BTreeMap;

/// The identity of the reserved `size` column: **unit scale**.
///
/// A node only writes the columns it changes, so every node that MATERIALIZES
/// `size` on a stream that had none must start from this base (`motion.scale`,
/// the Size channel of `oscillator`/`wiggle`/`noise`/`step`/`stagger`/`drive`,
/// `strobe`, …) — filling the gap with `[0,0]` would collapse every element to
/// nothing.
///
/// **The contract that binds the two ends:** whoever lowers a stream to
/// instances MUST use this same value as its fallback for an absent `size`. If
/// the renderer's fallback and the nodes' base disagree, then a node dropped at
/// its own IDENTITY silently resizes the whole scene — which is exactly what
/// happened (the shell lowered with `0.4` while every node assumed `1.0`, so a
/// `motion.scale` at `amount = 1` blew every quad up by 2.5×; doc 39).
pub const SIZE_IDENTITY: [f32; 2] = [1.0, 1.0];

/// The name of the reserved **value** column — the scalar a `value.*` node emits and a
/// consumer reads (`motion.drive`'s second input, a driven param, doc 58).
///
/// It was a `const VALUE_COL: &str = "v"` re-declared privately in every value node and
/// every consumer of one. That is a convention held together by everyone remembering it;
/// naming it here makes it a fact. (The node crates keep their local aliases — retyping the
/// letter in 30 crates is churn without a gate — but anything NEW reads it from here.)
pub const VALUE_COLUMN: &str = "v";

/// A typed column of per-element values, stored SoA.
#[derive(Clone, Debug, PartialEq)]
pub enum Column {
    Scalar(Vec<f32>),
    Vec2(Vec<[f32; 2]>),
    Vec3(Vec<[f32; 3]>),
    Vec4(Vec<[f32; 4]>),
}

impl Column {
    pub fn len(&self) -> usize {
        match self {
            Column::Scalar(v) => v.len(),
            Column::Vec2(v) => v.len(),
            Column::Vec3(v) => v.len(),
            Column::Vec4(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn dim(&self) -> Dim {
        match self {
            Column::Scalar(_) => Dim::Scalar,
            Column::Vec2(_) => Dim::Vec2,
            Column::Vec3(_) => Dim::Vec3,
            Column::Vec4(_) => Dim::Vec4,
        }
    }
}

/// A stream of `count` elements with named typed columns. An empty stream
/// (`count == 0`, no columns) is the value of an unconnected input.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stream {
    count: usize,
    attrs: BTreeMap<String, Column>,
}

impl Stream {
    pub fn new(count: usize) -> Self {
        Self {
            count,
            attrs: BTreeMap::new(),
        }
    }

    /// The empty stream (zero elements, no columns) as a `const` — the value of
    /// an unconnected input. Equal to [`Stream::default`], but `const` so a
    /// `static EMPTY_STREAM` can be borrowed for `&Stream` accessors (see
    /// [`crate::value::CookValue::as_stream`]).
    pub const fn empty() -> Self {
        Self {
            count: 0,
            attrs: BTreeMap::new(),
        }
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Attach/replace a column. The column length must equal the stream's
    /// element count (a per-element attribute). Returns the stream for
    /// chaining. Panics on length mismatch — a programming error in a node.
    pub fn with(mut self, name: impl Into<String>, col: Column) -> Self {
        self.set(name, col);
        self
    }

    pub fn set(&mut self, name: impl Into<String>, col: Column) {
        assert_eq!(
            col.len(),
            self.count,
            "column length must equal stream element count"
        );
        self.attrs.insert(name.into(), col);
    }

    pub fn get(&self, name: &str) -> Option<&Column> {
        self.attrs.get(name)
    }

    /// Deterministic iteration over `(name, column)` pairs.
    pub fn columns(&self) -> impl Iterator<Item = (&String, &Column)> {
        self.attrs.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_holds_typed_columns() {
        let s = Stream::new(3)
            .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]]))
            .with("size", Column::Scalar(vec![1.0, 1.0, 1.0]));
        assert_eq!(s.count(), 3);
        assert_eq!(s.get("P").unwrap().dim(), Dim::Vec2);
        assert_eq!(s.get("size").unwrap().len(), 3);
        assert!(s.get("missing").is_none());
    }

    #[test]
    #[should_panic(expected = "column length must equal stream element count")]
    fn mismatched_column_length_panics() {
        let mut s = Stream::new(3);
        s.set("bad", Column::Scalar(vec![1.0])); // len 1 != count 3
    }

    #[test]
    fn empty_stream_is_default() {
        assert_eq!(Stream::default(), Stream::new(0));
        assert!(Stream::default().is_empty());
    }
}
