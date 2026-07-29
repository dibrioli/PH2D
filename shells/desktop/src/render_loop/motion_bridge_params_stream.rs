//! The peek/stream-reading helpers of the params bridge (a `#[path]` child of
//! `motion_bridge_params.rs`, split off for the 600-LOC shell file cap). "What the
//! wires carry": the live number a wire drives into a param, and the live columns
//! the upstream stream offers the `value.attribute` Custom picker.

use crate::motion_state::MotionState;

/// **The live number a wire is putting into `param`** (doc 58), or `None` if nothing
/// drives it — read from the cook's MEMO (`Cook::peek`), so showing it costs a lookup
/// and never a second evaluation.
///
/// It is the SAME reduction the cook itself does (`driven_value`: the first value of
/// the `"v"` column) — a second one here would be a number that agrees with the wire
/// on most frames and disagrees on the frame that matters
/// ([[feedback_derived_coordinate_seed_must_match_sample]]).
pub(super) fn driven_value(
    motion: &MotionState,
    node: ph2d_nodegraph::graph::NodeId,
    param: &str,
) -> Option<f32> {
    let (src, port) = *motion.doc.graph.param_sources(node)?.get(param)?;
    let cooked = motion.pump.cook.peek(src)?;
    ph2d_nodegraph::param_source::driven_value(cooked.get(port as usize)?)
}

/// The scalar columns the stream feeding `node`'s input port 0 carries — the live
/// options for the `value.attribute` Custom picker (the roadmap's *dropdown populated
/// at runtime*). Curated `covered` columns and the value/sim/mask transients an artist
/// never reads are excluded; the current pick `keep` is always kept so its chip stays
/// put between cooks.
pub(super) fn upstream_scalar_columns(
    motion: &MotionState,
    node: ph2d_nodegraph::graph::NodeId,
    covered: &std::collections::BTreeSet<&str>,
    keep: &str,
) -> Vec<String> {
    // The edge feeding input port 0 (the stream in); a delayed feedback edge is not it.
    let Some((sn, sp)) = motion
        .doc
        .graph
        .edges()
        .iter()
        .find(|e| e.to == (node, 0) && !e.delayed)
        .map(|e| e.from)
    else {
        // `attr` is unwired — nothing upstream to read; the chip for the current pick
        // (if any) is all the picker can offer.
        return keep_extra_columns(std::iter::empty(), covered, keep);
    };
    let names = upstream_columns(motion, sn, sp);
    keep_extra_columns(names.iter().map(String::as_str), covered, keep)
}

/// The scalar-column NAMES the stream at `(sn, sp)` carries, **owned** so they outlive
/// either source.
///
/// Preferred from the pump's memo (`Cook::peek`) — a zero-cost lookup that is populated
/// whenever the graph cooks on the CPU. But the graph cooks on the **GPU by default**
/// (`PH2D_GPU_COOK=1`), and then the CPU memo is EMPTY: `motion_bridge::cook_gpu`
/// returns `Handled` and the sink loop that fills the memo is skipped. When the memo
/// misses, DISCOVER the columns with a fresh single-node cook. Column membership is
/// **structural** — which columns a node emits is a fact about the graph, not the tick
/// (a `motion.grid` carries `Index`/`Count` at every tick) — so `playhead = 0` is
/// enough, and it is exactly what the reference gate cooks.
///
/// Without the fallback the Custom picker showed NO "From stream" chips in the default
/// (GPU) env, even though the columns were right there upstream — the bug the Enio
/// reported. The cost is one CPU cook of the selected node's upstream, per frame, only
/// while a stream-column picker is on screen (an interactive, single-node situation).
fn upstream_columns(
    motion: &MotionState,
    sn: ph2d_nodegraph::graph::NodeId,
    sp: u16,
) -> Vec<String> {
    if let Some(stream) = motion
        .pump
        .cook
        .peek(sn)
        .and_then(|o| o.get(sp as usize))
        .map(|v| v.as_stream())
    {
        return scalar_names(stream);
    }
    let mut scratch = ph2d_nodegraph::cook::Cook::new();
    match scratch.cook(&motion.doc.graph, &motion.registry, sn, 0.0) {
        Ok(out) => out
            .get(sp as usize)
            .map(|v| scalar_names(v.as_stream()))
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

/// The names of the `Scalar` columns of `stream`, owned.
fn scalar_names(stream: &ph2d_nodegraph::attr::Stream) -> Vec<String> {
    use ph2d_nodegraph::attr::Column;
    stream
        .columns()
        .filter(|(_, c)| matches!(c, Column::Scalar(_)))
        .map(|(n, _)| n.to_string())
        .collect()
}

/// The PURE filter behind the picker: drop the curated columns and the internal /
/// transient ones, sort + dedup what remains, and lead with the current pick (so its
/// chip is stable even on a frame the stream did not cook). Extracted so the rule is
/// tested without a live cook.
fn keep_extra_columns<'a>(
    names: impl Iterator<Item = &'a str>,
    covered: &std::collections::BTreeSet<&str>,
    keep: &str,
) -> Vec<String> {
    // The value domain's own column + the sim/mask transients an artist never reads.
    const INTERNAL: &[&str] = &["v", "falloff", "accel", "sim_d", "sim_t", "weight"];
    let mut out: Vec<String> = names
        .filter(|n| !covered.contains(*n) && !INTERNAL.contains(n))
        .map(|n| n.to_string())
        .collect();
    out.sort();
    out.dedup();
    if !keep.is_empty() && !covered.contains(keep) && !out.iter().any(|c| c == keep) {
        out.insert(0, keep.to_string());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::keep_extra_columns;
    use std::collections::BTreeSet;

    /// Curated + internal columns are dropped, the rest sorted, and the current pick
    /// leads. A stream carrying `age`/`vel`/`opacity` (curated), `v` (internal), and
    /// `id`/`Index`/`inv_mass` (advanced) offers only the last three + the pick.
    #[test]
    fn keep_extra_drops_curated_and_internal_and_leads_with_the_pick() {
        let covered: BTreeSet<&str> = ["age", "vel", "opacity"].into_iter().collect();
        let got = keep_extra_columns(
            ["age", "v", "id", "Index", "inv_mass"].into_iter(),
            &covered,
            "my_attr",
        );
        // `age`/`v` gone; the rest byte-sorted (`I` < `i`); the pick inserted first.
        assert_eq!(got, vec!["my_attr", "Index", "id", "inv_mass"]);
    }

    /// A pick that IS a curated column is NOT added (Custom would not even be shown),
    /// and a pick already among the live columns is not duplicated.
    #[test]
    fn a_curated_or_present_pick_is_not_re_added() {
        let covered: BTreeSet<&str> = ["age"].into_iter().collect();
        assert_eq!(
            keep_extra_columns(["id"].into_iter(), &covered, "age"),
            vec!["id"]
        );
        assert_eq!(
            keep_extra_columns(["id"].into_iter(), &covered, "id"),
            vec!["id"]
        );
    }

    /// **The bug the Enio reported** (`esse env não tem Index e count`). With the graph
    /// cooked on the GPU (the default, `PH2D_GPU_COOK=1`) the CPU pump never runs, so its
    /// cook memo is empty and the old `peek`-only picker offered NO "From stream" chips.
    /// An UNPUMPED `MotionState` reproduces exactly that empty memo; the picker must still
    /// discover `Index`/`Count` via the fresh-cook fallback in [`upstream_columns`].
    /// RED-first: `peek`-only returns `[]` and both asserts fail.
    #[test]
    fn the_picker_offers_columns_when_the_cpu_memo_is_empty() {
        let mut motion = crate::motion_state::MotionState::new();
        motion.doc = ph2d_motion_doc::MotionDoc::new();
        let attr = crate::picker_smoke::build_picker_scene(&mut motion.doc.graph);
        // No pump: the CPU cook memo is empty, exactly as when the graph cooks on the GPU.
        let covered: BTreeSet<&str> = BTreeSet::new();
        let cols = super::upstream_scalar_columns(&motion, attr, &covered, "");
        assert!(cols.iter().any(|c| c == "Index"), "offers Index: {cols:?}");
        assert!(cols.iter().any(|c| c == "Count"), "offers Count: {cols:?}");
    }

    /// The other route: when the CPU pump DID cook the graph, the picker reads the same
    /// columns straight from its memo (`peek`) — so the fallback did not become the only
    /// working path.
    #[test]
    fn the_picker_offers_columns_from_the_cpu_pump_memo() {
        use ph2d_nodegraph::graph::NodeId;
        let mut motion = crate::motion_state::MotionState::new();
        motion.doc = ph2d_motion_doc::MotionDoc::new();
        let attr = crate::picker_smoke::build_picker_scene(&mut motion.doc.graph);
        let sinks: Vec<NodeId> = motion
            .doc
            .graph
            .nodes()
            .iter()
            .filter(|n| n.type_name == "motion.output")
            .map(|n| n.id)
            .collect();
        let (uv, size) = (motion.default_uv_rect, motion.default_size);
        motion
            .pump
            .pump(&motion.doc.graph, &motion.registry, &sinks, 0, 0.0, uv, size);
        let covered: BTreeSet<&str> = BTreeSet::new();
        let cols = super::upstream_scalar_columns(&motion, attr, &covered, "");
        assert!(cols.iter().any(|c| c == "Index"), "offers Index: {cols:?}");
        assert!(cols.iter().any(|c| c == "Count"), "offers Count: {cols:?}");
    }
}
