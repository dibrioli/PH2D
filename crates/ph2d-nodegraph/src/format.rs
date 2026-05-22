//! Textual graph format — diffable, mergeable, deterministic (ADR-0032 §6).
//!
//! The canonical on-disk form is **line-oriented**, not a nested blob: one
//! node per line, one edge per line, sorted by stable id. Two agridts adding
//! two nodes touch different lines, so `git` merges them without conflict —
//! the multi-agridt requirement that rules out JSON/RON trees. Node **layout**
//! (editor position) lives in a trailing `[layout]` section so it never
//! appears in a semantic diff.
//!
//! Grammar (whitespace-separated; canonical type names have no spaces):
//! ```text
//! v1
//! n <id> <type_name>
//! e <from_id> <from_port> <to_id> <to_port> <fwd|pre>
//! [layout]
//! l <id> <x> <y>
//! ```

use crate::graph::{Edge, EdgeError, Graph, NodeId, Pos};
use std::fmt::Write as _;

/// Serialize a graph to its canonical textual form. Deterministic: nodes,
/// edges and layout are each emitted in sorted order, so the same graph always
/// produces byte-identical output.
pub fn to_text(graph: &Graph) -> String {
    let mut out = String::from("v1\n");

    let mut nodes: Vec<_> = graph.nodes().iter().collect();
    nodes.sort_by_key(|n| n.id.0);
    for n in nodes {
        let _ = writeln!(out, "n {} {}", n.id.0, n.type_name);
    }

    let mut edges: Vec<_> = graph.edges().iter().collect();
    edges.sort_by_key(|e| (e.from.0 .0, e.from.1, e.to.0 .0, e.to.1, e.delayed));
    for e in edges {
        let kind = if e.delayed { "pre" } else { "fwd" };
        let _ = writeln!(
            out,
            "e {} {} {} {} {}",
            e.from.0 .0, e.from.1, e.to.0 .0, e.to.1, kind
        );
    }

    out.push_str("[layout]\n");
    // `layout()` is a BTreeMap, already sorted by id.
    for (id, pos) in graph.layout() {
        let _ = writeln!(out, "l {} {} {}", id.0, pos.x, pos.y);
    }

    out
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    /// Missing or wrong `v1` header.
    BadHeader,
    /// A line did not match any record grammar.
    BadLine(String),
    /// An edge record was structurally invalid for the graph.
    Edge(EdgeError),
}

/// Parse the canonical textual form back into a graph. Two-pass: all nodes are
/// inserted before any edge, so record order in the file does not matter and a
/// hand-edited file still loads. Edges are re-validated (a corrupt cyclic file
/// is rejected via the acyclicity invariant).
pub fn from_text(text: &str) -> Result<Graph, ParseError> {
    let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());

    match lines.next() {
        Some("v1") => {}
        _ => return Err(ParseError::BadHeader),
    }

    // Collect records first (two-pass), so order in the file is irrelevant.
    let mut node_recs: Vec<(NodeId, String)> = Vec::new();
    let mut edge_recs: Vec<Edge> = Vec::new();
    let mut layout_recs: Vec<(NodeId, Pos)> = Vec::new();

    for line in lines {
        let mut tok = line.split_whitespace();
        match tok.next() {
            Some("n") => {
                let id = NodeId(parse(&mut tok, line)?);
                let name = tok.next().ok_or_else(|| ParseError::BadLine(line.into()))?;
                node_recs.push((id, name.to_string()));
            }
            Some("e") => {
                let from = NodeId(parse(&mut tok, line)?);
                let from_port: u16 = parse(&mut tok, line)?;
                let to = NodeId(parse(&mut tok, line)?);
                let to_port: u16 = parse(&mut tok, line)?;
                let delayed = match tok.next() {
                    Some("pre") => true,
                    Some("fwd") => false,
                    _ => return Err(ParseError::BadLine(line.into())),
                };
                edge_recs.push(Edge {
                    from: (from, from_port),
                    to: (to, to_port),
                    delayed,
                });
            }
            Some("l") => {
                let id = NodeId(parse(&mut tok, line)?);
                let x: f32 = parse(&mut tok, line)?;
                let y: f32 = parse(&mut tok, line)?;
                layout_recs.push((id, Pos { x, y }));
            }
            Some("[layout]") => {}
            _ => return Err(ParseError::BadLine(line.into())),
        }
    }

    let mut graph = Graph::new();
    for (id, name) in node_recs {
        graph.insert_raw(id, name);
    }
    for e in edge_recs {
        graph.connect(e).map_err(ParseError::Edge)?;
    }
    for (id, pos) in layout_recs {
        graph.set_pos(id, pos);
    }
    Ok(graph)
}

fn parse<T: std::str::FromStr>(
    tok: &mut std::str::SplitWhitespace<'_>,
    line: &str,
) -> Result<T, ParseError> {
    tok.next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| ParseError::BadLine(line.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Graph {
        let mut g = Graph::new();
        let grid = g.add_node("motion.grid");
        let clone = g.add_node("motion.clone");
        g.connect(Edge { from: (grid, 0), to: (clone, 0), delayed: false }).unwrap();
        g.connect(Edge { from: (clone, 0), to: (clone, 1), delayed: true }).unwrap();
        g.set_pos(grid, Pos { x: 10.0, y: 20.0 });
        g.set_pos(clone, Pos { x: 120.0, y: 20.0 });
        g
    }

    #[test]
    fn round_trip_preserves_semantics_and_layout() {
        let g = sample();
        let text = to_text(&g);
        let back = from_text(&text).unwrap();
        assert_eq!(g.nodes(), back.nodes());
        assert_eq!(g.edges(), back.edges());
        assert_eq!(g.layout(), back.layout());
    }

    #[test]
    fn serialization_is_deterministic() {
        let g = sample();
        // Re-parsing then re-serializing yields byte-identical output.
        assert_eq!(to_text(&g), to_text(&from_text(&to_text(&g)).unwrap()));
    }

    #[test]
    fn layout_is_segregated_from_semantics() {
        // Two graphs identical but for node position serialize to the same
        // text up to the [layout] section — a semantic diff ignores layout.
        let mut a = sample();
        let text_a = to_text(&a);
        a.set_pos(NodeId(0), Pos { x: 999.0, y: 999.0 });
        let text_b = to_text(&a);
        let sem = |t: &str| t.split("[layout]").next().unwrap().to_string();
        assert_eq!(sem(&text_a), sem(&text_b));
        assert_ne!(text_a, text_b); // but the layout section did change
    }

    #[test]
    fn bad_header_is_rejected() {
        assert!(matches!(from_text("oops\n"), Err(ParseError::BadHeader)));
    }

    #[test]
    fn cyclic_file_is_rejected() {
        // A hand-forged file with a forward cycle must not load.
        let text = "v1\nn 0 a\nn 1 b\ne 0 0 1 0 fwd\ne 1 0 0 0 fwd\n[layout]\n";
        assert!(matches!(
            from_text(text),
            Err(ParseError::Edge(EdgeError::WouldCycle))
        ));
    }
}
