//! The peek/stream-reading helpers of the params bridge (a `#[path]` child of
//! `motion_bridge_params.rs`, split off for the 600-LOC shell file cap). "What the
//! wires carry": the live number a wire drives into a param, and the live columns
//! the upstream stream offers the `value.attribute` Custom picker.

use crate::motion_state::MotionState;

/// **The live number a wire is putting into `param`** (doc 58), or `None` if nothing
/// drives it — read from the cook's MEMO (`Cook::peek`) on a CPU frame, and from the
/// GPU **tap** on a device frame (the memo is empty then). Both are a lookup, never a
/// second evaluation, and both are one frame behind — the memo because `peek` runs
/// before this frame's cook, the tap by construction — so they agree
/// ([[feedback_derived_coordinate_seed_must_match_sample]]).
///
/// It is the SAME reduction the cook does (the first `v` scalar): the memo path is
/// `param_source::driven_value`, the tap path is `readout::reading_of` — the one
/// already-unified "what a wire is worth" the readout row and the probe share, so a
/// third copy is never minted ([[feedback_two_doors_to_the_same_question_diverge]]).
pub(super) fn driven_value(
    motion: &MotionState,
    node: ph2d_nodegraph::graph::NodeId,
    param: &str,
) -> Option<f32> {
    let (src, port) = *motion.doc.graph.param_sources(node)?.get(param)?;
    // CPU-cooked frame: the memo holds the real thing.
    if let Some(cooked) = motion.pump.cook.peek(src) {
        return ph2d_nodegraph::param_source::driven_value(cooked.get(port as usize)?);
    }
    // GPU-cooked frame: the memo is empty. The tap carries one subsampled stream per
    // STAGED node — port 0 only, which is what a value driver has — so a driver off a
    // higher output port is honestly unavailable here (rare; the row shows the
    // override instead). Row 0 of the tap IS element 0, so `v[0]` matches the memo.
    let stream = motion.gpu_tap.as_ref()?.get(&src).filter(|_| port == 0)?;
    match super::super::readout::reading_of(stream, stream.count() as u32) {
        super::super::readout::Reading::Value(v) => Some(v),
        super::super::readout::Reading::Instances(_) => None,
    }
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
/// The pump's memo (`Cook::peek`) on a CPU frame — a zero-cost lookup. But the graph
/// cooks on the **GPU by default** (`PH2D_GPU_COOK=1`), and then the memo is EMPTY:
/// `motion_bridge::cook_gpu` returns `Handled` and the sink loop that fills the memo is
/// skipped. On a device frame, read the SAME tap the graph panel's readouts read
/// (`motion.gpu_tap`) — the one door, not a private second cook
/// ([[feedback_two_doors_to_the_same_question_diverge]]). The tap is a 48-row subsample
/// per STAGED node, and column MEMBERSHIP is what the picker needs — a subsample carries
/// the same columns as the full stream, so 48 rows discover the names as well as four
/// million. Port 0 only (the tap stages one output per node); a source off a higher port
/// is honestly empty here (rare). Without this the Custom picker showed NO "From stream"
/// chips in the default (GPU) env, even though the columns were right there upstream —
/// the bug the Enio reported.
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
    match motion.gpu_tap.as_ref().and_then(|t| t.get(&sn)) {
        Some(stream) if sp == 0 => scalar_names(stream),
        _ => Vec::new(),
    }
}

/// **The names the app has published into the graph's external channel** (doc 65) — the
/// options a [`ParamWidget::Source`](ph2d_node_registry::ParamWidget::Source) picker
/// offers (a `motion.path` picks a drawn shape by name). They are the `Cook::externals`
/// keys, held on `motion.pump.cook` and republished every frame by the shell whether the
/// graph cooked on the CPU or the GPU — so the picker is never blind on a device frame.
/// Sorted (the map is a `BTreeMap`), so the chips are stable frame to frame.
pub(super) fn source_options(motion: &MotionState) -> Vec<String> {
    motion.pump.cook.externals().keys().cloned().collect()
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

    /// **The bug the Enio reported** (`esse env não tem Index e count`), by its ROOT
    /// cause. On a GPU-cooked frame (the default, `PH2D_GPU_COOK=1`) the CPU pump never
    /// runs, so `pump.cook` is empty; the picker must read the columns from the SAME tap
    /// the graph readouts read (`motion.gpu_tap`). Here the memo is empty and the tap
    /// carries the upstream node's subsample — the picker still offers `Index`/`Count`.
    /// RED-first: neuter the tap branch of [`upstream_columns`] and both asserts fail.
    #[test]
    fn the_picker_offers_columns_from_the_gpu_tap() {
        use ph2d_nodegraph::attr::{Column, Stream};
        use ph2d_nodegraph::graph::NodeId;
        use std::collections::BTreeMap;
        let mut motion = crate::motion_state::MotionState::new();
        motion.doc = ph2d_motion_doc::MotionDoc::new();
        let attr = crate::picker_smoke::build_picker_scene(&mut motion.doc.graph);
        // The node feeding attr's input port 0 (tint) — the one the tap must carry.
        let src: NodeId = motion
            .doc
            .graph
            .edges()
            .iter()
            .find(|e| e.to == (attr, 0))
            .map(|e| e.from.0)
            .expect("attr has an input");
        // A GPU frame: no pump (empty memo) + a tap holding a 48-row subsample with the
        // SAME columns the full stream carries.
        let sub = Stream::new(2)
            .with("Index", Column::Scalar(vec![0.0, 1.0]))
            .with("Count", Column::Scalar(vec![2.0, 2.0]))
            .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 1.0]]));
        motion.gpu_tap = Some(BTreeMap::from([(src, sub)]));
        let covered: BTreeSet<&str> = BTreeSet::new();
        let cols = super::upstream_scalar_columns(&motion, attr, &covered, "");
        assert!(cols.iter().any(|c| c == "Index"), "offers Index: {cols:?}");
        assert!(cols.iter().any(|c| c == "Count"), "offers Count: {cols:?}");
    }

    /// **The `motion.path` Source picker offers the DRAWN SHAPES by name** (doc 65) — the
    /// options are the names the app published into the external channel, so the artist
    /// picks the shape they drew instead of typing its id. RED-first: make `source_options`
    /// read from nothing (return empty) and the chips vanish.
    #[test]
    fn the_source_picker_offers_the_published_shape_names() {
        use ph2d_nodegraph::attr::{Column, Stream};
        let mut motion = crate::motion_state::MotionState::new();
        motion.doc = ph2d_motion_doc::MotionDoc::new();
        // The app published two drawn shapes into the graph's external channel.
        let seg = || Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 1.0]]));
        motion.pump.cook.set_external("Track", seg());
        motion.pump.cook.set_external("Ring", seg());
        let opts = super::source_options(&motion);
        assert!(opts.iter().any(|o| o == "Track"), "offers Track: {opts:?}");
        assert!(opts.iter().any(|o| o == "Ring"), "offers Ring: {opts:?}");
    }

    /// The other route: when the CPU pump DID cook the graph, the picker reads the same
    /// columns straight from its memo (`peek`) — so the tap did not become the only
    /// working path (the two are not redundant; each covers a different cook).
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

    /// **The driven-value readout (doc 58) works under GPU too.** A param driven by a
    /// wire shows the number the WIRE puts in; on a GPU frame that number lives only in
    /// the tap, and `driven_value` must read it there (the memo is empty). `b.strength`
    /// is driven by `a`, whose tapped `v` is 42. RED-first: neuter the tap branch → the
    /// row falls back to the override and this returns `None`.
    #[test]
    fn a_driven_param_reads_its_value_from_the_gpu_tap() {
        use ph2d_nodegraph::attr::{Column, Stream};
        use std::collections::BTreeMap;
        let mut motion = crate::motion_state::MotionState::new();
        motion.doc = ph2d_motion_doc::MotionDoc::new();
        let a = motion.doc.graph.add_node("value.gain");
        let b = motion.doc.graph.add_node("value.gain");
        motion
            .doc
            .graph
            .drive_param(b, "strength", (a, 0))
            .expect("drive strength from a");
        motion.gpu_tap = Some(BTreeMap::from([(
            a,
            Stream::new(1).with("v", Column::Scalar(vec![42.0])),
        )]));
        assert_eq!(
            super::driven_value(&motion, b, "strength"),
            Some(42.0),
            "the driven value comes from the tap when the memo is empty"
        );
    }
}
