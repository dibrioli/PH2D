//! Gates for [`crate::layout`]. Declared by the parent as a `#[path]` sibling for
//! the file LOC cap, so `super` is `layout`.
//!
//! ⚠️ **Three of these were rewritten in 2026-09-20 with the death of their old
//! premise visible in the diff.** They asserted a GRID (`k · DY`, columns centred
//! in a band of `(tallest−1)·DY`), and the grid is exactly what the owner's three
//! reports were about. What they assert now is the property the grid was standing
//! in for: *the drawn boxes clear each other*, and *a card sits where its
//! neighbours are*.

use super::*;
use crate::graph::Edge;

fn wire(g: &mut Graph, a: NodeId, b: NodeId) {
    g.connect(Edge {
        from: (a, 0),
        to: (b, 0),
        delayed: false,
    })
    .expect("edge");
}

/// Positions as a lookup, for asserting geometry.
fn placed(g: &Graph) -> BTreeMap<NodeId, Pos> {
    plan(g).into_iter().collect()
}

/// A card `w` wide and `h` tall, anchored at its top-left — the shape of a plain
/// node. Used by the gates that measure the SPACING, where the whole point is
/// that the cards are not all the same size.
fn boxy<K>(key: K, w: f32, h: f32) -> Item<K> {
    Item {
        key,
        extent: Extent {
            left: 0.0,
            right: w,
            top: 0.0,
            bottom: h,
        },
    }
}

/// A pill `w` wide **centred** on the historical card box, which is how a
/// zoomed-out card draws once its width follows its name (2026-09-20).
fn pill<K>(key: K, w: f32) -> Item<K> {
    let card = DX - GAP_X;
    Item {
        key,
        extent: Extent {
            left: (card - w) * 0.5,
            right: (card + w) * 0.5,
            top: 0.0,
            bottom: DY - GAP_Y,
        },
    }
}

fn link<K>(from: K, out_port: u16, to: K, in_port: u16) -> Wire<K> {
    Wire {
        from,
        out_port,
        to,
        in_port,
        delayed: false,
    }
}

// ---------------------------------------------------------------------------
// The owner's fixture (2026-09-20) — the chain of the `=120` scene.
// ---------------------------------------------------------------------------

/// `rope → scale → move → duplicator(port 1) → output`, plus `shape →
/// duplicator(port 0)`. The rope is keyed with a high bit because on that canvas
/// it is a collapsed GROUP card, which is the case the engine has to carry
/// heterogeneously.
///
/// ⚠️ This is the graph the owner photographed. Measured against the pre-2026-09-20
/// engine it gave `shape x = 60` against `duplicator x = 720` (four columns of wire
/// crossing two cards) and the rope at `y = 320` against its own `scale` at `190`.
const ROPE: u64 = 1 << 40;
const SCALE: u64 = 1;
const MOVE: u64 = 2;
const SHAPE: u64 = 3;
const DUP: u64 = 4;
const OUT: u64 = 5;

fn cena_do_dono() -> BTreeMap<u64, Pos> {
    let items = [
        Item::plain(ROPE),
        Item::plain(SCALE),
        Item::plain(MOVE),
        Item::plain(SHAPE),
        Item::plain(DUP),
        Item::plain(OUT),
    ];
    let wires = [
        link(ROPE, 0, SCALE, 0),
        link(SCALE, 0, MOVE, 0),
        // The duplicator takes the SHAPE on its upper slot and the stream of
        // positions on the lower one (ADR-0155) — both are required.
        link(MOVE, 0, DUP, 1),
        link(SHAPE, 0, DUP, 0),
        link(DUP, 0, OUT, 0),
    ];
    plan_edges(&items, &wires).into_iter().collect()
}

/// **Law 2 — a card sits next to what it FEEDS.** The `shape` has no producer, so
/// longest-path layering filed it in column 0, four columns from its only
/// consumer, and its wire crossed every card in between. FALSIFIED by deleting
/// the pull-right pass in `columns_of` (the shape goes back to `x = 60`).
#[test]
fn a_source_sits_next_to_what_it_feeds() {
    let p = cena_do_dono();
    assert!(
        (p[&SHAPE].x - p[&MOVE].x).abs() < 0.5,
        "the shape shares the duplicator's other producer's column (shape {}, move {})",
        p[&SHAPE].x,
        p[&MOVE].x
    );
    assert!(
        (p[&DUP].x - p[&SHAPE].x - DX).abs() < 0.5,
        "and that column is the one immediately before the duplicator"
    );
    // The control: pulling right must not drag the chain itself out of order.
    assert!(p[&ROPE].x < p[&SCALE].x && p[&SCALE].x < p[&MOVE].x && p[&DUP].x < p[&OUT].x);
}

/// **Law 1 — the card on the UPPER slot is placed ABOVE.** `shape` lands on the
/// duplicator's `in_port 0` and `move` on its `in_port 1`; the two share a column
/// and their barycenters TIE, so only the port can order them. FALSIFIED by
/// dropping the port from the sort key in `reorder` (the order then falls back to
/// the key order, which put `move` — the smaller key — on top).
#[test]
fn the_card_on_the_upper_slot_is_placed_above() {
    let p = cena_do_dono();
    assert!(
        p[&SHAPE].y < p[&MOVE].y,
        "port 0 draws above port 1 (shape {}, move {})",
        p[&SHAPE].y,
        p[&MOVE].y
    );
}

/// **Law 3 — «if there are two levels, the levels align at the centre».** The
/// duplicator joins the upper row (`shape`) and the lower one (`move`), and it
/// sits midway between them. FALSIFIED by putting the coordinate pass back on the
/// `k · DY` grid: the join then lands on a slot index, not between its inputs.
#[test]
fn a_join_sits_centred_between_the_rows_it_joins() {
    let p = cena_do_dono();
    let meio = (p[&SHAPE].y + p[&MOVE].y) * 0.5;
    assert!(
        (p[&DUP].y - meio).abs() < 0.5,
        "the duplicator is centred (dup {}, midpoint {})",
        p[&DUP].y,
        meio
    );
    assert!(
        p[&SHAPE].y < p[&DUP].y && p[&DUP].y < p[&MOVE].y,
        "and it is strictly between them"
    );
}

/// **Law 3, the other half — the chain BEHIND a join stays on one straight row.**
/// This is what the right→left half of the coordinate pass buys: `move` is pushed
/// off-centre by the join, and `scale` and the rope follow it instead of staying
/// on the centre line. FALSIFIED by dropping the right→left half (the rope and
/// the scale stay at the band's centre and the chain reads as a staircase — which
/// is exactly the `y = 320` against `190` the owner photographed).
#[test]
fn the_chain_behind_a_join_stays_on_one_straight_row() {
    let p = cena_do_dono();
    let ys = [p[&ROPE].y, p[&SCALE].y, p[&MOVE].y];
    let span =
        ys.iter().fold(f32::MIN, |a, &b| a.max(b)) - ys.iter().fold(f32::MAX, |a, &b| a.min(b));
    assert!(
        span < 0.5,
        "rope, scale and move share one row (span {span})"
    );
    // And the output follows the join, for the same reason.
    assert!((p[&OUT].y - p[&DUP].y).abs() < 0.5);
}

// ---------------------------------------------------------------------------
// Spacing — the drawn box, not a constant step.
// ---------------------------------------------------------------------------

/// **Two wide pills side by side do not overlap.** Before the extents this was
/// measured and NAMED as open: a pill of `232.5` against a step of `DX = 220`
/// meant two long names touched by construction. FALSIFIED by putting the step
/// back to a constant `DX`.
#[test]
fn two_wide_cards_side_by_side_do_not_overlap() {
    let largo = 340.0;
    let items = [pill(0u64, largo), pill(1u64, largo)];
    let wires = [link(0u64, 0, 1u64, 0)];
    let p: BTreeMap<u64, Pos> = plan_edges(&items, &wires).into_iter().collect();
    let esquerda = p[&0].x + items[0].extent.right;
    let direita = p[&1].x + items[1].extent.left;
    assert!(
        direita - esquerda >= GAP_X - 0.5,
        "the drawn boxes clear each other (gap {})",
        direita - esquerda
    );
}

/// **A tall card pushes the row below it.** Two siblings in one column, the first
/// three times the height of a default card: the second has to start below the
/// FIRST'S BOX, not one `DY` down. FALSIFIED by stacking on a constant `DY`.
#[test]
fn a_tall_card_pushes_the_row_below_it() {
    let alto = 660.0;
    let items = [
        boxy(0u64, 190.0, 40.0),
        boxy(1u64, 190.0, alto),
        boxy(2u64, 190.0, 40.0),
        boxy(3u64, 190.0, 40.0),
    ];
    // 0 forks into {1, 2} and they join into 3, so 1 and 2 share a column.
    let wires = [
        link(0u64, 0, 1u64, 0),
        link(0u64, 0, 2u64, 0),
        link(1u64, 0, 3u64, 0),
        link(2u64, 0, 3u64, 1),
    ];
    let p: BTreeMap<u64, Pos> = plan_edges(&items, &wires).into_iter().collect();
    let (cima, baixo) = if p[&1].y < p[&2].y {
        (1u64, 2u64)
    } else {
        (2u64, 1u64)
    };
    let fundo = p[&cima].y + if cima == 1 { alto } else { 40.0 };
    assert!(
        p[&baixo].y - fundo >= GAP_Y - 0.5,
        "the boxes clear each other (gap {})",
        p[&baixo].y - fundo
    );
}

/// **The default extent reproduces the historical step, to the bit.** The control
/// for the two gates above: a caller that does not measure must get exactly the
/// layout it got before the extents existed. FALSIFIED by any drift in `GAP_X`,
/// `GAP_Y` or `Extent::default`.
#[test]
fn the_default_extent_reproduces_the_historical_step() {
    let d = Extent::default();
    assert!((d.right - d.left + GAP_X - DX).abs() < f32::EPSILON);
    assert!((d.bottom - d.top + GAP_Y - DY).abs() < f32::EPSILON);
}

// ---------------------------------------------------------------------------
// The properties that predate 2026-09-20.
// ---------------------------------------------------------------------------

#[test]
fn a_chain_lays_out_as_one_straight_horizontal_line() {
    let mut g = Graph::new();
    let a = g.add_node("motion.grid");
    let b = g.add_node("motion.move");
    let c = g.add_node("motion.drive");
    let d = g.add_node("motion.output");
    wire(&mut g, a, b);
    wire(&mut g, b, c);
    wire(&mut g, c, d);
    let p = placed(&g);

    // One node per column ⇒ every card on the SAME y, strictly increasing x.
    let ys: BTreeSet<u32> = [a, b, c, d].iter().map(|n| p[n].y.to_bits()).collect();
    assert_eq!(ys.len(), 1, "a chain is a single straight line");
    assert!(p[&a].x < p[&b].x && p[&b].x < p[&c].x && p[&c].x < p[&d].x);
    // Columns are exactly one DX apart (no overlap; the source is leftmost).
    assert!((p[&b].x - p[&a].x - DX).abs() < 0.5);
}

/// ⚠️ **Premise replaced (2026-09-20).** This asserted `|Δy| ≥ DY`, which was a
/// statement about a GRID. What it means is that the two cards never touch, and
/// with default extents that is the same number — but now it is the number the
/// engine actually enforces.
#[test]
fn a_branch_stacks_siblings_in_one_column_at_distinct_non_overlapping_y() {
    // a → {b, c} → d : the fork and join each own a column; the siblings
    // share the middle column and must be spread so their cards never touch.
    let mut g = Graph::new();
    let a = g.add_node("motion.grid");
    let b = g.add_node("value.lfo");
    let c = g.add_node("value.noise");
    let d = g.add_node("motion.drive");
    // Fork from a's one output; join into d's TWO input ports (one edge per input).
    wire(&mut g, a, b);
    wire(&mut g, a, c);
    for (from, to_port) in [(b, 0u16), (c, 1u16)] {
        g.connect(Edge {
            from: (from, 0),
            to: (d, to_port),
            delayed: false,
        })
        .expect("edge");
    }
    let p = placed(&g);

    // b and c are in the SAME column (both one step from the source)…
    assert!((p[&b].x - p[&c].x).abs() < 0.5, "siblings share a column");
    // …and their drawn boxes clear each other.
    let d0 = Extent::default();
    assert!(
        (p[&b].y - p[&c].y).abs() >= d0.bottom - d0.top + GAP_Y - 0.5,
        "siblings are spread so their cards never overlap"
    );
    // The fork sits left of the siblings, the join to their right.
    assert!(p[&a].x < p[&b].x && p[&b].x < p[&d].x);
    // Law 1 again, through the graph wrapper: b feeds port 0, c feeds port 1.
    assert!(p[&b].y < p[&c].y, "the upper slot draws above");
}

#[test]
fn two_disconnected_chains_become_two_non_overlapping_bands() {
    let mut g = Graph::new();
    let a1 = g.add_node("motion.grid");
    let a2 = g.add_node("motion.output");
    wire(&mut g, a1, a2);
    let b1 = g.add_node("motion.grid");
    let b2 = g.add_node("motion.output");
    wire(&mut g, b1, b2);
    let p = placed(&g);

    let band_a = p[&a1].y.max(p[&a2].y) + Extent::default().bottom;
    let band_b = p[&b1].y.min(p[&b2].y);
    // The whole second component sits a clear BAND_GAP below the first BOX —
    // the two rows never share vertical space.
    assert!(
        band_b - band_a >= BAND_GAP - 1.0,
        "components stack into separate bands (gap {})",
        band_b - band_a
    );
}

#[test]
fn a_pre_edge_is_ignored_for_columns_but_keeps_the_band_together() {
    // a → b → c with a feedback c ⇢ a (delayed). Columns must ignore the
    // feedback (c is still rightmost, no cycle, no stretched layout), and
    // all three stay in ONE band.
    let mut g = Graph::new();
    let a = g.add_node("sim.zone");
    let b = g.add_node("force.wind");
    let c = g.add_node("sim.step");
    wire(&mut g, a, b);
    wire(&mut g, b, c);
    g.connect(Edge {
        from: (c, 0),
        to: (a, 0),
        delayed: true,
    })
    .expect("pre edge");
    let p = placed(&g);

    // Forward order preserved despite the back-edge; exactly three columns.
    assert!(p[&a].x < p[&b].x && p[&b].x < p[&c].x);
    assert!(
        (p[&c].x - p[&a].x - 2.0 * DX).abs() < 0.5,
        "three columns, no more"
    );
    // One weakly-connected component ⇒ one band: no BAND_GAP jump inside it.
    let span = [a, b, c].iter().map(|n| p[n].y).fold(f32::MIN, f32::max)
        - [a, b, c].iter().map(|n| p[n].y).fold(f32::MAX, f32::min);
    assert!(span < BAND_GAP, "the feedback loop stays in one band");
}

#[test]
fn arrange_moves_the_stored_positions() {
    // Positive control: `arrange` actually writes the plan into the graph.
    let mut g = Graph::new();
    let a = g.add_node("motion.grid");
    let b = g.add_node("motion.output");
    wire(&mut g, a, b);
    g.set_pos(a, Pos { x: 999.0, y: 999.0 });
    arrange(&mut g);
    assert!(
        g.pos(a).expect("pos").x < 100.0,
        "arrange overwrote the stale position"
    );
    assert!(g.pos(a).unwrap().x < g.pos(b).unwrap().x);
}

#[test]
fn plan_edges_places_a_group_card_inline_between_its_neighbours() {
    // The motion-subgraph case: a chain n0 → CARD → n2, where CARD is a
    // collapsed group standing in for hidden members. Keyed heterogeneously
    // (nodes low, the card with a high bit) — the engine must place the card
    // in its own column between the two nodes, not off to the side.
    const CARD: u64 = 1 << 40;
    let items = [Item::plain(0u64), Item::plain(CARD), Item::plain(2u64)];
    let wires = [link(0u64, 0, CARD, 0), link(CARD, 0, 2u64, 0)];
    let p: BTreeMap<u64, Pos> = plan_edges(&items, &wires).into_iter().collect();
    assert!(
        p[&0].x < p[&CARD].x && p[&CARD].x < p[&2].x,
        "card sits between its neighbours"
    );
    // A clean inline chain: one column apart, same row.
    assert!((p[&CARD].x - p[&0].x - DX).abs() < 0.5);
    assert!(
        (p[&CARD].y - p[&0].y).abs() < 0.5,
        "inline, not off to the side"
    );
}
