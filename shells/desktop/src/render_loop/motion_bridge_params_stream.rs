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

/// The scalar columns the stream feeding `node`'s input port 0 carries RIGHT NOW
/// (from the cook memo, `Cook::peek`) — the live options for the `value.attribute`
/// Custom picker (the roadmap's *dropdown populated at runtime*). Curated `covered`
/// columns and the value/sim/mask transients an artist never reads are excluded; the
/// current pick `keep` is always kept so its chip stays put between cooks.
pub(super) fn upstream_scalar_columns(
    motion: &MotionState,
    node: ph2d_nodegraph::graph::NodeId,
    covered: &std::collections::BTreeSet<&str>,
    keep: &str,
) -> Vec<String> {
    use ph2d_nodegraph::attr::Column;
    // The edge feeding input port 0 (the stream in); a delayed feedback edge is not it.
    let src = motion
        .doc
        .graph
        .edges()
        .iter()
        .find(|e| e.to == (node, 0) && !e.delayed)
        .map(|e| e.from);
    let stream = src.and_then(|(sn, sp)| {
        motion
            .pump
            .cook
            .peek(sn)
            .and_then(|o| o.get(sp as usize))
            .map(|v| v.as_stream())
    });
    let mut names: Vec<&str> = Vec::new();
    if let Some(stream) = stream {
        for (name, col) in stream.columns() {
            if matches!(col, Column::Scalar(_)) {
                names.push(name.as_str());
            }
        }
    }
    keep_extra_columns(names.into_iter(), covered, keep)
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
}
