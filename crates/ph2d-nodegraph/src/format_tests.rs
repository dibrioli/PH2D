//! Unit tests for [`super`] (`format.rs`) — extracted to a sibling module (`#[path]`) so the
//! textual-format source stays under the 700-LOC workspace cap, the idiom its siblings
//! (`graph_tests.rs`, `cook_tests.rs`) already use. Pure relocation — no test changed.

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

/// **A bypass round-trips through the `y` record, and only then is the file `v5`.** A muted
/// node saved and reloaded stays muted; a graph nobody muted stays byte-identical to v1. A `y`
/// on an unknown node id, or with a trailing token, is rejected at the boundary. FALSIFIED by
/// dropping the `y` emit (the save loses the mute), the parse arm (the load loses it), or the
/// version bump (the header stays v1 and a v5-only reader would reject a legit file).
#[test]
fn a_bypass_round_trips_and_only_then_is_the_file_v5() {
    let mut g = Graph::new();
    let a = g.add_node("motion.grid");
    let b = g.add_node("motion.clone");
    g.set_bypassed(b, true);

    let text = to_text(&g);
    assert!(
        text.starts_with("v5\n"),
        "a bypass bumps the header to v5:\n{text}"
    );
    assert!(
        text.contains(&format!("y {}\n", b.0)),
        "the `y` record is emitted:\n{text}"
    );
    // The `y` sits ABOVE `[layout]` — it is semantic (it changes the cook).
    assert!(text.find("\ny ").unwrap() < text.find("[layout]").unwrap());

    let back = from_text(&text).unwrap();
    assert!(back.node_bypassed(b), "the reloaded node is still muted");
    assert!(!back.node_bypassed(a), "the untouched node is not");

    // Un-muting returns the file to the version it would otherwise have had (byte-identical).
    g.set_bypassed(b, false);
    assert!(to_text(&g).starts_with("v1\n"), "no mute → back to v1");

    // A `y` on a node with no `n` record → rejected (no phantom).
    assert!(matches!(
        from_text("v5\nn 0 a\ny 7\n[layout]\n"),
        Err(ParseError::BadLine(_))
    ));
    // A trailing token on a `y` line → rejected.
    assert!(matches!(
        from_text("v5\nn 0 a\ny 0 extra\n[layout]\n"),
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

// ── Driven params (doc 58) ──────────────────────────────────────────────

/// A driven param round-trips, and it bumps the header to `v3` — **only** when there is
/// one. A graph that never drove a param must serialize byte for byte as it always did,
/// or every existing document in the repo changes on its next save.
#[test]
fn a_driven_param_round_trips_and_only_then_is_the_file_v3() {
    let mut g = Graph::new();
    let a = g.add_node("a");
    let b = g.add_node("b");
    assert!(
        to_text(&g).starts_with("v1\n"),
        "no driven param, no version bump - the old file is untouched"
    );

    g.drive_param(b, "strength", (a, 0)).unwrap();
    let text = to_text(&g);
    assert!(text.starts_with("v3\n"));
    assert!(
        text.contains("d 1 strength 0 0\n"),
        "the record names the param, not a port index: {text}"
    );
    let back = from_text(&text).unwrap();
    assert_eq!(back, g, "the document survives the round trip");

    // …and pulling the wire off takes the version back down with it.
    g.undrive_param(b, "strength");
    assert!(to_text(&g).starts_with("v1\n"));
}

/// A `d` record is an EDGE, so a hand-forged file cannot use one to smuggle in a cycle
/// (which the cook would meet as a stack overflow) or a wire to a node that is not there.
#[test]
fn a_forged_driven_param_cannot_cycle_or_point_at_nothing() {
    let cyclic = "v3\nn 0 a\nn 1 b\ne 0 0 1 0 fwd\nd 0 k 1 0\n[layout]\n";
    assert!(matches!(
        from_text(cyclic),
        Err(ParseError::Edge(EdgeError::WouldCycle))
    ));
    let phantom = "v3\nn 0 a\nd 0 k 7 0\n[layout]\n";
    assert!(matches!(from_text(phantom), Err(ParseError::BadLine(_))));
}

/// **A name with a space in it is ONE name** (doc 61) — the trailing-free-text rule the
/// `x` record already lives by. Get this wrong and "The Sea" loads as a label of "The"
/// plus a parse error, or (worse) silently truncates.
#[test]
fn a_label_round_trips_with_its_spaces_and_only_then_is_the_file_v4() {
    let mut g = Graph::new();
    let a = g.add_node("force.buoyancy");
    let b = g.add_node("motion.grid");
    assert!(
        to_text(&g).starts_with("v1\n"),
        "nobody renamed anything - the old file is untouched, byte for byte"
    );

    g.set_label(a, "The Sea");
    let text = to_text(&g);
    assert!(text.starts_with("v4\n"), "{text}");
    assert!(text.contains("t 0 The Sea\n"), "{text}");
    let back = from_text(&text).unwrap();
    assert_eq!(back.label(a), Some("The Sea"));
    assert_eq!(back, g, "the document survives the round trip");
    assert_eq!(back.label(b), None, "an unnamed node stays unnamed");

    // Un-naming it takes the version back down with it — a rename you undid leaves no
    // trace in the file.
    g.set_label(a, "");
    assert!(to_text(&g).starts_with("v1\n"));
    assert!(!to_text(&g).contains("t "));
}

/// The version cascade is a LADDER, not a switch: a labelled graph that also drives a
/// param is still v4, and every older file still loads.
#[test]
fn the_version_ladder_holds_and_old_files_still_load() {
    let mut g = Graph::new();
    let a = g.add_node("a");
    let b = g.add_node("b");
    g.drive_param(b, "k", (a, 0)).unwrap();
    assert!(to_text(&g).starts_with("v3\n"));
    g.set_label(a, "Driver");
    let text = to_text(&g);
    assert!(text.starts_with("v4\n"), "the newest record wins: {text}");
    assert!(text.contains("d 1 k 0 0\n") && text.contains("t 0 Driver\n"));
    assert_eq!(from_text(&text).unwrap(), g);

    // Every file the app has ever written still opens.
    for old in [
        "v1\nn 0 a\n[layout]\n",
        "v2\nn 0 a\nx 0 formula @P.x * 2\n[layout]\n",
        "v3\nn 0 a\nn 1 b\nd 1 k 0 0\n[layout]\n",
    ] {
        assert!(
            from_text(old).is_ok(),
            "a v-old file must still load: {old}"
        );
    }
}

/// A label on a node that is not in the file is a phantom — the same rejection every
/// other per-node record gets. And a `t` with nothing after the id is not a label.
#[test]
fn a_forged_label_is_rejected() {
    assert!(matches!(
        from_text("v4\nn 0 a\nt 7 Ghost\n[layout]\n"),
        Err(ParseError::BadLine(_))
    ));
    assert!(matches!(
        from_text("v4\nn 0 a\nt 0\n[layout]\n"),
        Err(ParseError::BadLine(_))
    ));
}
