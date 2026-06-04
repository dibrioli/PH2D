//! The mirror engine: reflect the input network across the X and/or Y axis
//! (through the network-local origin) and combine the copies into one network.
//!
//! A reflection is a **linear** map (no translation), so a vertex position and a
//! segment's relative tangent vectors transform by the *same* formula — we apply
//! one closure to both. Reflection flips a loop's orientation, but a simple
//! filled region renders the same under NonZero either way, so we copy the
//! regions verbatim (no re-ordering). Overlapping copies are kept as distinct
//! regions (not boolean-merged) — faithful to "N copies"; a boolean merge is a
//! later refinement if a use-case needs it.

use std::collections::BTreeMap;

use glam::Vec2;
use ph2d_vector_doc::deterministic::snap;
use ph2d_vector_doc::{Region, RegionId, Segment, SegmentId, VectorNetwork, Vertex, VertexId};

/// Which axis (axes) to mirror across — the `axis` discriminant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MirrorAxis {
    /// Reflect across the X axis (negate y): original + 1 vertical copy.
    X,
    /// Reflect across the Y axis (negate x): original + 1 horizontal copy.
    Y,
    /// Both axes: original + 3 copies (4-up).
    Both,
}

/// Reflect `input` per `axis`, returning original + reflected copies combined.
#[must_use]
pub fn mirror(input: &VectorNetwork, axis: MirrorAxis) -> VectorNetwork {
    let mut b = Builder::new();
    b.append(input, |p| p); // the original, verbatim
    match axis {
        MirrorAxis::X => b.append(input, |p| Vec2::new(p.x, -p.y)),
        MirrorAxis::Y => b.append(input, |p| Vec2::new(-p.x, p.y)),
        MirrorAxis::Both => {
            b.append(input, |p| Vec2::new(-p.x, p.y));
            b.append(input, |p| Vec2::new(p.x, -p.y));
            b.append(input, |p| Vec2::new(-p.x, -p.y));
        }
    }
    if input.deterministic {
        snap_network(&mut b.net);
    }
    b.net.deterministic = input.deterministic;
    b.net
}

/// Accumulates transformed copies of a source network into one result, assigning
/// fresh sequential ids per copy so the merged network is `validate`-clean
/// regardless of the source's id scheme.
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

    /// Append one copy of `src` with `map` applied to every position **and**
    /// tangent (valid because `map` is a linear/affine-without-translation
    /// transform). Old→new id maps keep segment/region refs consistent.
    fn append(&mut self, src: &VectorNetwork, map: impl Fn(Vec2) -> Vec2) {
        let mut vmap: BTreeMap<VertexId, VertexId> = BTreeMap::new();
        for v in &src.vertices {
            let nid = self.next_v;
            self.next_v += 1;
            vmap.insert(v.id, nid);
            self.net.vertices.push(Vertex::auto(nid, map(v.pos)));
        }
        let mut smap: BTreeMap<SegmentId, SegmentId> = BTreeMap::new();
        for s in &src.segments {
            // A segment referencing a missing vertex is dropped (strict; keeps
            // the merged network valid).
            let (Some(&start), Some(&end)) = (vmap.get(&s.start), vmap.get(&s.end)) else {
                continue;
            };
            let nid = self.next_s;
            self.next_s += 1;
            smap.insert(s.id, nid);
            let mut ns = Segment::straight(nid, start, end);
            ns.out_at_start = map(s.out_at_start);
            ns.in_at_end = map(s.in_at_end);
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
            // Skip a region that lost all its segments to the strict drop above.
            if !nr.segments.is_empty() {
                self.net.regions.push(nr);
            }
        }
    }
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

    /// A unit square in the +x+y quadrant (so reflections land in distinct
    /// quadrants and don't overlap).
    fn quad() -> VectorNetwork {
        let mut net = primitives::rect(Vec2::new(1.0, 1.0), Vec2::new(3.0, 3.0));
        net.deterministic = true;
        net
    }

    fn bbox(net: &VectorNetwork) -> (Vec2, Vec2) {
        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);
        for v in &net.vertices {
            min = min.min(v.pos);
            max = max.max(v.pos);
        }
        (min, max)
    }

    #[test]
    fn mirror_both_makes_four_copies() {
        // The Day-5 smoke: Quad → 4 copies, one per quadrant, spanning [-3,3]².
        let out = mirror(&quad(), MirrorAxis::Both);
        assert!(out.validate().is_ok());
        assert_eq!(out.regions.len(), 4, "4-up");
        assert_eq!(out.vertices.len(), 16, "4 × 4 corners");
        let (min, max) = bbox(&out);
        assert_eq!(min, Vec2::new(-3.0, -3.0));
        assert_eq!(max, Vec2::new(3.0, 3.0));
    }

    #[test]
    fn mirror_x_makes_two_copies_reflected_in_y() {
        let out = mirror(&quad(), MirrorAxis::X);
        assert!(out.validate().is_ok());
        assert_eq!(out.regions.len(), 2);
        let (min, max) = bbox(&out);
        // Original y∈[1,3], reflected y∈[-3,-1]; x unchanged [1,3].
        assert_eq!(min, Vec2::new(1.0, -3.0));
        assert_eq!(max, Vec2::new(3.0, 3.0));
    }

    #[test]
    fn mirror_y_makes_two_copies_reflected_in_x() {
        let out = mirror(&quad(), MirrorAxis::Y);
        assert!(out.validate().is_ok());
        assert_eq!(out.regions.len(), 2);
        let (min, max) = bbox(&out);
        assert_eq!(min, Vec2::new(-3.0, 1.0));
        assert_eq!(max, Vec2::new(3.0, 3.0));
    }

    #[test]
    fn deterministic_and_reproducible() {
        let out = mirror(&quad(), MirrorAxis::Both);
        assert!(out.deterministic);
        assert_eq!(out, mirror(&quad(), MirrorAxis::Both), "byte-stable");
    }

    #[test]
    fn empty_input_is_empty() {
        let out = mirror(&VectorNetwork::empty(), MirrorAxis::Both);
        assert!(out.validate().is_ok());
        assert!(out.regions.is_empty());
    }
}
