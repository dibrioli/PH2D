//! Textual graph format — diffable, mergeable, deterministic (ADR-0032 §6).
//!
//! The canonical on-disk form is **line-oriented**, not a nested blob: one
//! node per line, one edge per line, sorted by stable id. Two agents adding
//! two nodes touch different lines, so `git` merges them without conflict —
//! the multi-agent requirement that rules out JSON/RON trees. Node **layout**
//! (editor position) lives in a trailing `[layout]` section so it never
//! appears in a semantic diff.
//!
//! Grammar (whitespace-separated; canonical type names have no spaces):
//! ```text
//! v1 | v2
//! n <id> <type_name>
//! e <from_id> <from_port> <to_id> <to_port> <fwd|pre>
//! p <id> <param_name> <value>
//! x <id> <text_param_name> <formula...>          (v2 only)
//! [layout]
//! l <id> <x> <y>
//! ```
//!
//! `p` (f32 param overrides) and `x` (**text** param overrides — an expression node's
//! formula) records are **semantic** — they sit above `[layout]` so a semantic diff
//! includes them, unlike `l` (layout) records.
//!
//! The `x` record's last field is **free text**: it is everything after the third space,
//! so a formula's interior single spaces round-trip exactly (the same trailing-free-text
//! convention as the backdrop `b` title in `ph2d-motion-doc`). Formulas are single-line;
//! leading/trailing line whitespace is trimmed on load (lines are trimmed).
//!
//! Versioning: `p` is part of the **frozen `v1` grammar** (pre-W2.T4). The `x` record is a
//! **post-freeze** record kind, so — per the versioning policy — it bumps the header to
//! `v2`: `to_text` emits `v2` **iff** the graph carries a (non-empty) text param, else
//! `v1` (byte-identical for text-param-free graphs). `from_text` accepts both `v1` and
//! `v2`. This is the isolation-preserving text-param channel (docs/Motion Nodes/32) — the
//! frozen NODE contract (`NodeManifest`/`NodeOp`/`OpResolver`) is untouched; only the
//! serialization grammar gains an additive, versioned record.

use crate::graph::{Edge, EdgeError, Graph, NodeId, Pos};
use std::fmt::Write as _;

/// Serialize a graph to its canonical textual form. Deterministic: nodes,
/// edges and layout are each emitted in sorted order, so the same graph always
/// produces byte-identical output.
pub fn to_text(graph: &Graph) -> String {
    // A (non-empty) text param bumps the header to v2 (a post-freeze record kind); a
    // text-param-free graph stays byte-identical v1.
    let has_text = graph
        .node_text_params()
        .values()
        .any(|m| m.values().any(|v| !v.is_empty()));
    let mut out = String::from(if has_text { "v2\n" } else { "v1\n" });

    let mut nodes: Vec<_> = graph.nodes().iter().collect();
    nodes.sort_by_key(|n| n.id.0);
    for n in nodes {
        let _ = writeln!(out, "n {} {}", n.id.0, n.type_name);
    }

    let mut edges: Vec<_> = graph.edges().iter().collect();
    edges.sort_by_key(|e| (e.from.0.0, e.from.1, e.to.0.0, e.to.1, e.delayed));
    for e in edges {
        let kind = if e.delayed { "pre" } else { "fwd" };
        let _ = writeln!(
            out,
            "e {} {} {} {} {}",
            e.from.0.0, e.from.1, e.to.0.0, e.to.1, kind
        );
    }

    // Per-instance param overrides (semantic). `node_params()` is a nested
    // `BTreeMap`, so node ids and param names are both already sorted.
    for (id, params) in graph.node_params() {
        for (name, value) in params {
            let _ = writeln!(out, "p {} {} {}", id.0, name, value);
        }
    }

    // Per-node text params (semantic). The formula is the trailing free-text field
    // (interior spaces preserved). Empty formulas == unset, so skip them.
    for (id, params) in graph.node_text_params() {
        for (name, value) in params {
            if !value.is_empty() {
                let _ = writeln!(out, "x {} {} {}", id.0, name, value);
            }
        }
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
        Some("v1") | Some("v2") => {}
        _ => return Err(ParseError::BadHeader),
    }

    // Collect records first (two-pass), so order in the file is irrelevant.
    let mut node_recs: Vec<(NodeId, String)> = Vec::new();
    let mut edge_recs: Vec<Edge> = Vec::new();
    let mut param_recs: Vec<(NodeId, String, f32)> = Vec::new();
    let mut text_param_recs: Vec<(NodeId, String, String)> = Vec::new();
    let mut layout_recs: Vec<(NodeId, Pos)> = Vec::new();
    let mut seen_ids = std::collections::BTreeSet::new();

    for line in lines {
        // The `x` (text param) record's last field is free text (a formula with interior
        // spaces), so split into at most 4 fields — the trailing one keeps its spaces
        // (the same convention as the backdrop `b` title). Handled before the
        // whitespace-collapsing tokenizer the other records use.
        if line.starts_with("x ") {
            let parts: Vec<&str> = line.splitn(4, ' ').collect();
            if parts.len() < 4 {
                return Err(ParseError::BadLine(line.into()));
            }
            let id = NodeId(
                parts[1]
                    .parse()
                    .map_err(|_| ParseError::BadLine(line.into()))?,
            );
            text_param_recs.push((id, parts[2].to_string(), parts[3].to_string()));
            continue;
        }
        let mut tok = line.split_whitespace();
        match tok.next() {
            Some("n") => {
                let id = NodeId(parse(&mut tok, line)?);
                if !seen_ids.insert(id) {
                    return Err(ParseError::BadLine(line.into())); // duplicate id
                }
                let name = tok.next().ok_or_else(|| ParseError::BadLine(line.into()))?;
                if tok.next().is_some() {
                    // Extra tokens: e.g. a corrupt file with a whitespaced name
                    // ("n 0 motion clone") that would otherwise load lossy.
                    return Err(ParseError::BadLine(line.into()));
                }
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
            Some("p") => {
                let id = NodeId(parse(&mut tok, line)?);
                let name = tok.next().ok_or_else(|| ParseError::BadLine(line.into()))?;
                let value: f32 = parse(&mut tok, line)?;
                // Reject non-finite overrides (`"nan"`/`"inf"` parse fine as f32
                // but are never legitimate authored params) and any trailing
                // token (a whitespaced param name would split, loading lossy).
                if !value.is_finite() || tok.next().is_some() {
                    return Err(ParseError::BadLine(line.into()));
                }
                param_recs.push((id, name.to_string(), value));
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

    // A `p` record must reference a node declared by an `n` record — otherwise
    // the override is a dead entry on a non-existent node (`set_param` is lenient
    // for the in-code path, mirroring `set_pos`, so the file boundary rejects it
    // here rather than storing an unvalidatable phantom). Order-independent: the
    // check runs after the two-pass collect, so a `p` may precede its `n`.
    if let Some((id, _, _)) = param_recs.iter().find(|(id, _, _)| !seen_ids.contains(id)) {
        return Err(ParseError::BadLine(format!(
            "p record for unknown node id {}",
            id.0
        )));
    }
    // Same for `x` (text param) records — no phantom on a non-existent node.
    if let Some((id, _, _)) = text_param_recs
        .iter()
        .find(|(id, _, _)| !seen_ids.contains(id))
    {
        return Err(ParseError::BadLine(format!(
            "x record for unknown node id {}",
            id.0
        )));
    }

    let mut graph = Graph::new();
    for (id, name) in node_recs {
        graph.insert_raw(id, name);
    }
    for e in edge_recs {
        graph.connect(e).map_err(ParseError::Edge)?;
    }
    for (id, name, value) in param_recs {
        graph.set_param(id, name, value);
    }
    for (id, name, value) in text_param_recs {
        graph.set_text_param(id, name, value);
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
        g.connect(Edge {
            from: (grid, 0),
            to: (clone, 0),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (clone, 0),
            to: (clone, 1),
            delayed: true,
        })
        .unwrap();
        g.set_pos(grid, Pos { x: 10.0, y: 20.0 });
        g.set_pos(clone, Pos { x: 120.0, y: 20.0 });
        // Per-instance param overrides — exercises the `p` record round-trip.
        g.set_param(grid, "rows", 4.0);
        g.set_param(grid, "cols", 5.0);
        g.set_param(clone, "count", 2.0);
        g
    }

    #[test]
    fn round_trip_preserves_semantics_and_layout() {
        let g = sample();
        let text = to_text(&g);
        let back = from_text(&text).unwrap();
        assert_eq!(g.nodes(), back.nodes());
        assert_eq!(g.edges(), back.edges());
        assert_eq!(g.node_params(), back.node_params());
        assert_eq!(g.layout(), back.layout());
    }

    /// A **text** param (an expression formula, with interior spaces and operators)
    /// round-trips through the `x` record, and the header bumps to `v2`. FALSIFIED if the
    /// formula's spaces split it lossy, or if it were dropped (the old data-loss bug).
    #[test]
    fn text_params_round_trip_with_spaces() {
        let mut g = Graph::new();
        let e = g.add_node("motion.expression");
        g.set_text_param(e, "expr", "sin(f * a + t) * 4 > b && c");
        let text = to_text(&g);
        assert!(
            text.starts_with("v2\n"),
            "a text param bumps to v2:\n{text}"
        );
        let back = from_text(&text).unwrap();
        assert_eq!(
            g.node_text_params(),
            back.node_text_params(),
            "the formula (spaces + operators) survives"
        );
    }

    /// A text-param-free graph stays `v1` (byte-identical to before the feature).
    #[test]
    fn text_param_free_graph_stays_v1() {
        assert!(to_text(&sample()).starts_with("v1\n"));
    }

    /// Both `v1` (no `x`) and `v2` (with `x`) files load; an `x` on an unknown node id or a
    /// malformed `x` (too few fields) is rejected at the boundary.
    #[test]
    fn x_record_parsing_and_rejections() {
        // A hand-written v2 file with an `x` record loads.
        let g = from_text("v2\nn 0 motion.expression\nx 0 expr i * 2\n[layout]\n").unwrap();
        assert_eq!(
            g.node_text_param_overrides(NodeId(0)).unwrap()["expr"],
            "i * 2"
        );
        // `x` for a node with no `n` record → rejected (no phantom).
        assert!(matches!(
            from_text("v2\nn 0 a\nx 7 expr i\n[layout]\n"),
            Err(ParseError::BadLine(_))
        ));
        // Malformed `x` (missing the formula field) → rejected.
        assert!(matches!(
            from_text("v2\nn 0 a\nx 0 expr\n[layout]\n"),
            Err(ParseError::BadLine(_))
        ));
    }

    #[test]
    fn params_are_in_the_semantic_section_above_layout() {
        // A `p` record must sit before `[layout]` so a semantic diff includes it
        // (params change the cook; layout does not).
        let text = to_text(&sample());
        let p_pos = text.find("\np ").expect("a param record");
        let layout_pos = text.find("[layout]").expect("layout section");
        assert!(p_pos < layout_pos);
    }

    #[test]
    fn non_finite_or_malformed_param_is_rejected() {
        // "nan"/"inf" parse as f32 but are never legitimate authored params.
        assert!(matches!(
            from_text("v1\nn 0 a\np 0 k nan\n[layout]\n"),
            Err(ParseError::BadLine(_))
        ));
        assert!(matches!(
            from_text("v1\nn 0 a\np 0 k inf\n[layout]\n"),
            Err(ParseError::BadLine(_))
        ));
        // A trailing token (a whitespaced param name would split lossy).
        assert!(matches!(
            from_text("v1\nn 0 a\np 0 k 1.0 extra\n[layout]\n"),
            Err(ParseError::BadLine(_))
        ));
    }

    #[test]
    fn param_for_unknown_node_id_is_rejected() {
        // A `p` referencing a node id with no `n` record is a dead override —
        // rejected at the file boundary rather than stored as a phantom.
        assert!(matches!(
            from_text("v1\nn 0 a\np 7 k 1.0\n[layout]\n"),
            Err(ParseError::BadLine(_))
        ));
        // Order-independent: a `p` *before* its `n` still loads.
        assert!(from_text("v1\np 0 k 1.0\nn 0 a\n[layout]\n").is_ok());
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
    fn n_line_with_extra_tokens_is_rejected() {
        // A type name with a space splits into extra tokens; reject rather than
        // load a truncated name (the round-trip would be silently lossy).
        assert!(matches!(
            from_text("v1\nn 0 motion clone\n[layout]\n"),
            Err(ParseError::BadLine(_))
        ));
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
