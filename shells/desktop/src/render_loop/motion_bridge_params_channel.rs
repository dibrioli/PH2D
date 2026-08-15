//! Channel-aware presets of the params bridge (a `#[path]` child of
//! `motion_bridge_params.rs`, split off for the 600-LOC shell file cap). "What a
//! behaviour's magnitude MEANS on the channel it drives" — the reset presets and the
//! Rotation option they hinge on. The widget-range WIDENING (`contain` /
//! `channel_range_override`) stays with `build_params_snapshot`, its only caller.

use crate::motion_state::MotionState;
use ph2d_node_registry::ParamUnit;

/// Reset a behaviour node's magnitude params to a sensible default for the newly
/// selected channel (#10 consistency). Switching what a stagger/oscillator drives
/// — X/Y position (world units) vs Rotation (degrees) vs Size (scale delta) —
/// rewrites the range so a `±1` meant for position doesn't read as a barely-there
/// ±1° / ±huge-scale on the other channels. Editor UX (not node math): it
/// runs on the channel switch inside the same undo step, so Ctrl+Z restores the
/// artist's previous values. Non-behaviour node types are a no-op.
pub(in crate::render_loop::motion_bridge) fn apply_channel_presets(
    motion: &mut MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    type_name: &str,
    channel: f32,
) {
    let ch = channel.round() as i32;
    match type_name {
        "motion.stagger" => {
            let (min, max) = stagger_channel_range(ch);
            motion.doc.graph.set_param(nid, "min", min);
            motion.doc.graph.set_param(nid, "max", max);
        }
        // Both wave behaviours carry the same `channel` enum + `amplitude`.
        "motion.oscillator" | "motion.wiggle" => {
            motion
                .doc
                .graph
                .set_param(nid, "amplitude", wave_channel_amplitude(ch));
            // The oscillator's DC `offset` shares the amplitude's unit, so its
            // range is channel-dependent too (`channel_range_override`). Any param
            // whose RANGE moves with the channel must have its VALUE reset with it
            // — otherwise a 300° offset dialled on Rotation survives into a
            // ±10-world-unit position channel, outside the range it will be shown
            // in. Zero is the neutral offset and is legal on every channel.
            if type_name == "motion.oscillator" {
                motion.doc.graph.set_param(nid, "offset", 0.0);
            }
        }
        _ => {}
    }
    // ⚠️ **E depois a regra DERIVADA, que é a que não apodrece.** Todo param cuja
    // FAIXA segue o canal (a declaração do nó) tem de ter o VALOR trazido para
    // dentro da faixa do canal NOVO — senão um `scale` de 360 graus dialado em
    // Rotation sobrevive para o canal X, onde ele é 360 unidades de mundo e joga
    // as instâncias para fora do quadro, mostrado num slider que não o alcança.
    //
    // Os presets acima escolhem um valor BONITO para os três nós que os têm; isto
    // só garante o mínimo para TODOS — e para aqueles três é no-op, porque o valor
    // que eles escrevem já está dentro (gate).
    clamp_channel_ranged_params(motion, nid, type_name, ch);
}

/// Traz cada param de faixa-por-canal para dentro da faixa que o canal NOVO
/// implica: a declarada, se o canal é angular; a do hint, caso contrário — pela
/// MESMA porta que a row do painel usa, senão a faixa em que o valor é guardado e
/// a faixa em que ele é mostrado divergiriam.
fn clamp_channel_ranged_params(
    motion: &mut MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    type_name: &str,
    ch: i32,
) {
    let id = ph2d_nodegraph::node::NodeTypeId::of(type_name);
    let Some(decls) = motion
        .registry
        .channel_ranged_types()
        .find(|(k, _)| *k == id)
        .map(|(_, v)| v)
    else {
        return;
    };
    let hints = motion.registry.param_ui(id).unwrap_or(&[]);
    for d in decls {
        let (lo, hi) = if channel_unit(ch) == ParamUnit::Angle {
            (d.min, d.max)
        } else if let Some(h) = hints.iter().find(|h| h.param == d.param) {
            (h.min, h.max)
        } else {
            continue;
        };
        let v = motion
            .doc
            .graph
            .node_param_overrides(nid)
            .and_then(|m| m.get(d.param).copied())
            .unwrap_or_else(|| {
                motion
                    .registry
                    .manifests()
                    .find(|m| m.id == id)
                    .and_then(|m| m.params.iter().find(|p| p.name == d.param))
                    .map_or(0.0, |p| p.default)
            });
        let c = v.clamp(lo, hi);
        if c != v {
            motion.doc.graph.set_param(nid, d.param, c);
        }
    }
}

/// The `channel` enum's Rotation option (see the behaviours' `channel` hint:
/// `0` X, `1` Y, `2` Rotation, `3` Size).
pub(super) const CHANNEL_ROTATION: i32 = 2;

/// **What a magnitude MEANS on the channel it drives** (doc 88) — the resolution
/// of [`ParamUnit::FromChannel`], living next to the ranges and presets that
/// already answer the same question for their own halves.
///
/// This is the whole reason the `FromChannel` variant exists: a stagger's `min`
/// is metres on Position, DEGREES on Rotation and a bare scale factor on Size,
/// and a boundary that converted all three by `pixels_per_meter` would turn the
/// `±90` preset into `±9000`. The three answers were already written down here
/// twice (once as a range, once as a preset); this is the third face of the one
/// fact, deliberately in the same file so they cannot drift.
///
/// An unknown index falls back to [`ParamUnit::None`] — a visible gap is worth
/// more than a wrong scale.
pub(super) fn channel_unit(channel: i32) -> ParamUnit {
    match channel {
        0 | 1 => ParamUnit::Length,           // Position X / Y: world metres
        CHANNEL_ROTATION => ParamUnit::Angle, // the `rot` column: degrees
        3 => ParamUnit::Ratio,                // Size: a scale delta, dimensionless
        _ => ParamUnit::None,
    }
}

/// Stagger `(min, max)` ramp endpoints per channel. The Rotation channel writes
/// the `rot` stream column, whose unit is **degrees** (the app's authored-angle
/// unit); Position is world units, Size a scale delta.
fn stagger_channel_range(channel: i32) -> (f32, f32) {
    match channel {
        CHANNEL_ROTATION => (-90.0, 90.0), // ±90 degrees
        3 => (-0.5, 0.5),                  // Size: ±0.5 scale
        _ => (-1.0, 1.0),                  // Position (X/Y): ±1 world unit
    }
}

/// Peak `amplitude` per channel for the wave behaviours (oscillator / wiggle) —
/// same unit logic as the stagger range.
fn wave_channel_amplitude(channel: i32) -> f32 {
    match channel {
        CHANNEL_ROTATION => 30.0, // ±30 degrees
        3 => 0.3,                 // Size: ±0.3 scale
        _ => 1.0,                 // Position: ±1 world unit
    }
}
