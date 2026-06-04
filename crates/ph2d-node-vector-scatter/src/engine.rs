//! The scatter engine: place `count` translated copies of the input at
//! deterministic pseudo-random positions within a `spread × spread` area, merged
//! into one network. Translation only (a relative tangent is unaffected), so the
//! copies are rigid duplicates.
//!
//! ## Determinism (cross-OS golden)
//!
//! Positions come from a pure integer hash of `(seed, copy_index, axis)` mapped
//! to `[-spread, spread]` — bit-identical on every target (no `rand`, no global
//! state). Snapped to Q16.16 when the input is `deterministic`.

use std::collections::BTreeMap;

use glam::Vec2;
use ph2d_vector_doc::deterministic::snap;
use ph2d_vector_doc::{Region, RegionId, Segment, SegmentId, VectorNetwork, Vertex, VertexId};

/// Allocation ceiling for `count` — bounds the untrusted `f32 → count` against
/// an OOM value (each copy duplicates the whole input network).
pub const MAX_COUNT: usize = 1 << 14;

/// Scatter `count` copies of `input` within `±spread` of the origin, seeded by
/// `seed`.
#[must_use]
pub fn scatter(input: &VectorNetwork, count: usize, spread: f32, seed: u32) -> VectorNetwork {
    let n = count.min(MAX_COUNT);
    if n == 0 || input.vertices.is_empty() {
        return {
            let mut empty = VectorNetwork::empty();
            empty.deterministic = input.deterministic;
            empty
        };
    }
    let mut b = Builder::new();
    for i in 0..n {
        let off = Vec2::new(
            spread * noise(seed, i as u32, 0),
            spread * noise(seed, i as u32, 1),
        );
        b.append(input, off);
    }
    if input.deterministic {
        snap_network(&mut b.net);
    }
    b.net.deterministic = input.deterministic;
    b.net
}

struct Builder {
    net: VectorNetwork,
    next_v: VertexId,
    next_s: SegmentId,
    next_r: RegionId,
}

impl Builder {
    fn new() -> Self {
        Self {
            net: VectorNetwork::empty(),
            next_v: 0,
            next_s: 0,
            next_r: 0,
        }
    }

    /// Append one copy of `src` translated by `offset` (positions move; relative
    /// tangents are unchanged). Fresh ids keep the merged network valid.
    fn append(&mut self, src: &VectorNetwork, offset: Vec2) {
        let mut vmap: BTreeMap<VertexId, VertexId> = BTreeMap::new();
        for v in &src.vertices {
            let nid = self.next_v;
            self.next_v += 1;
            vmap.insert(v.id, nid);
            self.net.vertices.push(Vertex::auto(nid, v.pos + offset));
        }
        let mut smap: BTreeMap<SegmentId, SegmentId> = BTreeMap::new();
        for s in &src.segments {
            let (Some(&start), Some(&end)) = (vmap.get(&s.start), vmap.get(&s.end)) else {
                continue;
            };
            let nid = self.next_s;
            self.next_s += 1;
            smap.insert(s.id, nid);
            let mut ns = Segment::straight(nid, start, end);
            ns.out_at_start = s.out_at_start;
            ns.in_at_end = s.in_at_end;
            ns.style_ref = s.style_ref;
            self.net.segments.push(ns);
        }
        for r in &src.regions {
            let nid = self.next_r;
            self.next_r += 1;
            let mut nr = Region::new(nid, r.winding);
            nr.segments = r
                .segments
                .iter()
                .filter_map(|(sid, fwd)| smap.get(sid).map(|&ns| (ns, *fwd)))
                .collect();
            nr.fill = r.fill;
            nr.z = r.z;
            if !nr.segments.is_empty() {
                self.net.regions.push(nr);
            }
        }
    }
}

/// Deterministic signed noise in `[-1, 1)` from `(seed, index, axis)`.
fn noise(seed: u32, index: u32, axis: u32) -> f32 {
    let mut h = seed ^ 0x9e37_79b9;
    h = (h.wrapping_mul(0x85eb_ca6b)) ^ index.rotate_left(15);
    h = (h.wrapping_mul(0xc2b2_ae35)) ^ (axis.wrapping_add(1)).rotate_left(13);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb_352d);
    h ^= h >> 15;
    (h as f32 / u32::MAX as f32) * 2.0 - 1.0
}

fn snap_network(net: &mut VectorNetwork) {
    for v in net.vertices.iter_mut() {
        v.pos = Vec2::new(snap(v.pos.x), snap(v.pos.y));
    }
    for s in net.segments.iter_mut() {
        s.out_at_start = Vec2::new(snap(s.out_at_start.x), snap(s.out_at_start.y));
        s.in_at_end = Vec2::new(snap(s.in_at_end.x), snap(s.in_at_end.y));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vector_doc::primitives;

    fn dot() -> VectorNetwork {
        let mut net = primitives::rect(Vec2::new(-1.0, -1.0), Vec2::new(1.0, 1.0));
        net.deterministic = true;
        net
    }

    #[test]
    fn scatter_makes_count_copies() {
        let out = scatter(&dot(), 10, 100.0, 0);
        assert!(out.validate().is_ok());
        assert_eq!(out.regions.len(), 10);
        assert_eq!(out.vertices.len(), 40, "10 × 4 corners");
    }

    #[test]
    fn copies_land_within_spread() {
        let spread = 100.0;
        let out = scatter(&dot(), 50, spread, 7);
        // Each corner is within [-spread-1, spread+1] (the dot is ±1).
        for v in &out.vertices {
            assert!(v.pos.x.abs() <= spread + 1.0 && v.pos.y.abs() <= spread + 1.0);
        }
    }

    #[test]
    fn seed_changes_placement() {
        assert_ne!(scatter(&dot(), 10, 100.0, 1), scatter(&dot(), 10, 100.0, 2));
    }

    #[test]
    fn zero_count_is_empty() {
        let out = scatter(&dot(), 0, 100.0, 0);
        assert!(out.validate().is_ok());
        assert!(out.regions.is_empty());
    }

    #[test]
    fn deterministic_and_reproducible() {
        let out = scatter(&dot(), 25, 100.0, 42);
        assert!(out.deterministic);
        assert_eq!(out, scatter(&dot(), 25, 100.0, 42), "byte-stable per seed");
    }
}
