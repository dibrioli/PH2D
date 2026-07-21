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
use rayon::prelude::*;
use std::collections::BTreeMap;

/// Above this element count, [`par_build`] spreads the per-element map across
/// cores; below it stays serial. The threshold exists because rayon's
/// fork/join has a fixed cost that a tiny stream (the boot demo's handful of
/// instances) would pay for nothing — and, more importantly, the small-N path
/// must **not** touch the thread pool or allocate, so the paused/idle no-alloc
/// guarantee (`ph2d-eval-motion/tests/paused_no_alloc.rs`) holds. Correctness is
/// identical on both sides; this is purely a performance floor.
///
/// GPU/M5 Fase 0. Tunable; ~8k is well above any editor-chrome stream and well
/// below where an O(N) node starts to hurt on one core.
pub const PAR_THRESHOLD: usize = 8192;

/// Build a per-element `Vec<T>` by index, in parallel above [`PAR_THRESHOLD`].
///
/// **Determinism contract (the whole reason this is one audited function):**
/// `f(i)` MUST be a pure function of `i` plus shared *immutable* data — no
/// cross-element read whose result depends on iteration order, and **no float
/// reduction** (a sum/average across elements reorders under threads and IEEE
/// addition is not associative). Nodes that reduce or read neighbours
/// (`voronoi`, `boids`, the sims) keep their serial fixed-order pass — see the
/// split in `motion.voronoi`. For a pure map there is nothing to reorder:
/// rayon's indexed `collect` writes element `i` to slot `i`, so the result is
/// **bit-identical** to the serial `(0..n).map(f).collect()`, and the ECS
/// `transform_determinism` / replay-hash gates are unaffected.
pub fn par_build<T, F>(n: usize, f: F) -> Vec<T>
where
    T: Send,
    F: Fn(usize) -> T + Sync + Send,
{
    if n >= PAR_THRESHOLD {
        (0..n).into_par_iter().map(f).collect()
    } else {
        (0..n).map(f).collect()
    }
}

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
    /// The heap bytes this column's data occupies — the honest input to a
    /// byte-budgeted cache (ADR-0137: a checkpoint ring capped by COUNT is a
    /// multiplier, the ADR-0117 class). An estimate by design: capacity slack
    /// and allocator overhead are not counted, which errs on the small side —
    /// the direction a budget must NOT err — so callers pad their budgets, not
    /// this number.
    pub fn approx_bytes(&self) -> usize {
        match self {
            Column::Scalar(v) => v.len() * 4,
            Column::Vec2(v) => v.len() * 8,
            Column::Vec3(v) => v.len() * 12,
            Column::Vec4(v) => v.len() * 16,
        }
    }

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
/// **Cloning a `Stream` is a refcount, not a copy** (ADR-0138): the columns
/// live behind `Arc`, so the sim loop's per-tick snapshots (`Cook::checkpoint`,
/// `advance_tick`'s prev, the pump's boundary hand-off) share data instead of
/// re-copying the whole field. Sound because nothing mutates a `Column` in
/// place — the API has no `get_mut`, and every writer builds a fresh column and
/// [`Stream::set`]s it (the same immutability the GPU side's buffers rely on).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stream {
    count: usize,
    attrs: BTreeMap<String, std::sync::Arc<Column>>,
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
        self.attrs.insert(name.into(), std::sync::Arc::new(col));
    }

    pub fn get(&self, name: &str) -> Option<&Column> {
        self.attrs.get(name).map(std::sync::Arc::as_ref)
    }

    /// Deterministic iteration over `(name, column)` pairs.
    /// The stream's column data in bytes — see [`Column::approx_bytes`].
    pub fn approx_bytes(&self) -> usize {
        self.columns().map(|(_, c)| c.approx_bytes()).sum()
    }

    pub fn columns(&self) -> impl Iterator<Item = (&String, &Column)> {
        self.attrs.iter().map(|(n, c)| (n, c.as_ref()))
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

    /// **Cloning shares, writing replaces** (ADR-0138): the clone's columns are
    /// the SAME allocations (a refcount, not a copy — what makes the sim loop's
    /// per-tick snapshots cheap), and a `set` on the clone swaps in a fresh
    /// column without touching the original. A `Stream` whose clone deep-copied
    /// would fail the pointer identity here — the exact regression this pins.
    #[test]
    fn cloning_a_stream_shares_columns_and_writing_replaces_them() {
        let a = Stream::new(2).with("P", Column::Vec2(vec![[1.0, 2.0], [3.0, 4.0]]));
        let mut b = a.clone();
        assert!(
            std::ptr::eq(a.get("P").unwrap(), b.get("P").unwrap()),
            "a cloned column must be the same allocation"
        );
        b.set("P", Column::Vec2(vec![[9.0, 9.0], [8.0, 8.0]]));
        assert!(
            !std::ptr::eq(a.get("P").unwrap(), b.get("P").unwrap()),
            "writing un-shares"
        );
        assert_eq!(
            a.get("P").unwrap(),
            &Column::Vec2(vec![[1.0, 2.0], [3.0, 4.0]]),
            "the original is untouched"
        );
    }

    #[test]
    fn empty_stream_is_default() {
        assert_eq!(Stream::default(), Stream::new(0));
        assert!(Stream::default().is_empty());
    }

    #[test]
    fn par_build_is_bit_identical_to_serial_both_sides_of_the_threshold() {
        // A closure with the shape a node uses: a per-element f32 map that would
        // ULP-drift if the arithmetic were reordered (mul + add of a fractional
        // index). Above the threshold `par_build` runs on the thread pool; the
        // result MUST equal the explicit serial map byte-for-byte, or a
        // parallelized node would silently diverge from the CPU canon.
        let f = |i: usize| (i as f32) * 0.1 + (i as f32).sqrt() * 1.3;
        for n in [
            0usize,
            1,
            100,
            PAR_THRESHOLD - 1,
            PAR_THRESHOLD,
            PAR_THRESHOLD * 4 + 7,
        ] {
            let serial: Vec<f32> = (0..n).map(f).collect();
            let built = par_build(n, f);
            assert_eq!(built, serial, "par_build diverged from serial at n = {n}");
        }
    }

    #[test]
    fn par_build_preserves_index_order() {
        // The identity map: element i must land at slot i (rayon's indexed
        // collect), not merely contain the same multiset — a set-preserving but
        // order-shuffling collect would pass a hash-of-sorted check yet corrupt
        // the P/tint/size correspondence across columns.
        let n = PAR_THRESHOLD * 2;
        let built = par_build(n, |i| i as u32);
        assert!(built.iter().copied().eq(0..n as u32));
    }
}
