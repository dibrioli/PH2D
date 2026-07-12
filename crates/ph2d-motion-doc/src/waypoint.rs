//! **Waypoints** — the routing points a wire is dragged through (Motion Nodes F2, doc 44).
//! A sibling module of the document (the 600-LOC cap; `lib.rs` calls `emit` / `parse`).
//!
//! ## Why they live on the DOCUMENT and not on the `Edge`
//!
//! A waypoint changes **how a wire is drawn**. It changes **nothing** about what the graph
//! computes. Putting one on `ph2d_nodegraph::graph::Edge` would push decoration into the
//! substrate every cook, every fingerprint and every gate has to reason about — and the
//! `Edge` is the frozen contract's neighbour. So waypoints go exactly where the backdrops
//! went (doc 35), for exactly the same reason: **the document's UI-only section**.
//!
//! The payoff is executable: the `is_dirty` guard the backdrops brought means a waypoint
//! drag can be *proven* not to re-cook the graph. Decoration that cannot dirty the cook is
//! decoration you can drag at 60 Hz without thinking.
//!
//! ## The key is the INPUT, because that is what identifies a wire
//!
//! An input port holds **at most one** edge — the graph's own invariant, the same one
//! `GraphIntent::Disconnect { to_node, to_port }` already leans on. So "the wire landing on
//! this input" names a wire exactly, with no source in the key. Cut that wire and its
//! waypoints go with it (they mean nothing without it); re-wire the input from somewhere
//! else and the new wire starts clean.

use std::fmt::Write as _;

/// The routing points of ONE wire, in the order the wire passes through them.
///
/// `to_node` / `to_port` name the input the wire lands on — see the module note: that pair
/// IS the wire's identity.
#[derive(Clone, Debug, PartialEq)]
pub struct Waypoints {
    pub to_node: u32,
    pub to_port: u16,
    /// Graph-space points, in order from the source socket toward the target.
    pub points: Vec<(f32, f32)>,
}

/// Emit every wire's waypoints as `w` records, sorted for a stable diff. A wire with no
/// points emits nothing (an empty record would be a line that says "this wire is straight",
/// which is what its absence already says).
pub(crate) fn emit(out: &mut String, waypoints: &[Waypoints]) {
    let mut sorted: Vec<&Waypoints> = waypoints.iter().filter(|w| !w.points.is_empty()).collect();
    sorted.sort_by_key(|w| (w.to_node, w.to_port));
    for w in sorted {
        let _ = write!(out, "w {} {}", w.to_node, w.to_port);
        for (x, y) in &w.points {
            let _ = write!(out, " {x} {y}");
        }
        let _ = writeln!(out);
    }
}

/// Parse one `w` record. `None` when the line is malformed — the caller turns that into a
/// `ParseError`, so a corrupt file is rejected rather than silently losing a wire's routing.
///
/// The coordinates are checked for finiteness: a `NaN` waypoint would poison the wire's
/// polyline and make it vanish, which reads as "the editor ate my wire".
pub(crate) fn parse(line: &str) -> Option<Waypoints> {
    let mut tok = line.split_whitespace();
    if tok.next()? != "w" {
        return None;
    }
    let to_node: u32 = tok.next()?.parse().ok()?;
    let to_port: u16 = tok.next()?.parse().ok()?;

    let rest: Vec<f32> = tok.map(|s| s.parse::<f32>().ok()).collect::<Option<_>>()?;
    if rest.is_empty() || !rest.len().is_multiple_of(2) || !rest.iter().all(|v| v.is_finite()) {
        return None;
    }
    Some(Waypoints {
        to_node,
        to_port,
        points: rest.chunks_exact(2).map(|c| (c[0], c[1])).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wire_with_no_points_emits_nothing() {
        let mut out = String::new();
        emit(
            &mut out,
            &[Waypoints {
                to_node: 1,
                to_port: 0,
                points: vec![],
            }],
        );
        assert!(out.is_empty(), "a straight wire needs no record: {out:?}");
    }

    #[test]
    fn the_records_round_trip_in_order_and_sorted() {
        let mut out = String::new();
        emit(
            &mut out,
            &[
                Waypoints {
                    to_node: 7,
                    to_port: 1,
                    points: vec![(3.0, 4.0)],
                },
                Waypoints {
                    to_node: 2,
                    to_port: 0,
                    points: vec![(1.0, 2.0), (5.0, 6.0)],
                },
            ],
        );
        // Sorted by (node, port) — a stable diff, whatever order they were authored in.
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines, vec!["w 2 0 1 2 5 6", "w 7 1 3 4"]);
        let w = parse(lines[0]).expect("parses");
        assert_eq!(w.to_node, 2);
        assert_eq!(w.points, vec![(1.0, 2.0), (5.0, 6.0)], "order preserved");
    }

    /// A corrupt record is REJECTED, not silently half-read — an odd coordinate count, a
    /// `NaN`, or a missing field. (A NaN waypoint would make the wire's polyline vanish,
    /// which reads as "the editor ate my wire".)
    #[test]
    fn a_corrupt_record_is_rejected() {
        assert!(parse("w 2 0 1 2 5").is_none(), "odd coordinate count");
        assert!(parse("w 2 0 1 NaN").is_none(), "a non-finite point");
        assert!(parse("w 2 0").is_none(), "no points at all");
        assert!(parse("w 2").is_none(), "missing the port");
        assert!(parse("b 1 0 0 1 1 0 x").is_none(), "not our record");
    }
}
