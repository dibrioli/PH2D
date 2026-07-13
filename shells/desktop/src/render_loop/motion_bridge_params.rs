//! Params-panel side of the Motion bridge (split out of `motion_bridge.rs` to
//! keep each file under the HR-18 LOC cap). Owns the selected node's param edits
//! and the params-panel snapshot: scalar sliders, named enum / checkbox rows,
//! and the OKLCH colour-swatch authoring (the sRGB↔linear boundary lives here).
//!
//! The whole module is gated on both Motion panels — its parent declares it with
//! the same `all(panel-motion-graph, panel-motion-params)` cfg — so the helpers
//! carry no per-fn gate. All entry points are `pub(super)` for `dispatch`.

use super::color::{apply_color_to_node, color_groups, linear_rgba_to_srgb8};
use crate::motion_state::MotionState;

/// Apply this frame's params-panel edits to the selected node, bracketed into
/// undo steps (M1.P1 + colour authoring). Two edit sources, ONE session model:
///
/// - **Scalar** slider / chip edits arrive as queued
///   [`SetParam`](ph2d_panel_motion_params::MotionParamIntent)s.
/// - **Colour** edits arrive continuously while a swatch's OKLCH picker is open:
///   the live pick is read back (sRGB→linear) into the group's 4 channel params.
///
/// A whole gesture is ONE undo step: the bracket opens on the false→true edge of
/// an *editing session* (`any_param_editing` OR a colour picker targeting one of
/// the node's swatches) and commits on release; a discrete typed commit (no
/// session) is wrapped in its own step. Each applied edit re-cooks
/// (`mark_dirty`). Stale intents whose node no longer exists are dropped.
pub(super) fn apply_param_edits(
    motion: &mut MotionState,
    store: &ph2d_editor::interaction::WidgetStore,
) {
    use ph2d_nodegraph::graph::NodeId;
    use ph2d_panel_motion_params::{MotionParamIntent, any_param_editing, param_swatch_id};
    use std::sync::atomic::{AtomicBool, Ordering};
    static PARAM_EDITING: AtomicBool = AtomicBool::new(false);

    // The selected node + its colour groups (each = 4 RGBA channel params driven
    // by one swatch → OKLCH picker).
    let sel = selected_motion_node().map(NodeId);
    let groups = sel
        .and_then(|nid| motion.doc.graph.node(nid).map(|i| i.type_id()))
        .map(|tid| color_groups(&motion.registry, tid))
        .unwrap_or_default();

    // A colour pick is an editing session (like a slider drag): its live param
    // writes coalesce into ONE undo step, opened here + committed on close.
    let color_session = groups
        .iter()
        .any(|ch| store.picker_target() == Some(param_swatch_id(ch[0])));
    let editing = any_param_editing(store) || color_session;
    let was = PARAM_EDITING.swap(editing, Ordering::Relaxed);
    if editing && !was {
        motion.history.begin(&motion.doc);
    }

    // Colour read-back: while a swatch's picker is open, feed the live pick into
    // its 4 channel params (sRGB→linear), re-cooking only on an actual change.
    if let Some(nid) = sel {
        for ch in &groups {
            if store.picker_target() == Some(param_swatch_id(ch[0]))
                && let Some((value, _, _, _)) =
                    store.blender_picker(ph2d_editor::ids::INSP_BLENDER_PICKER)
            {
                apply_color_to_node(motion, nid, *ch, value.rgba);
            }
        }
    }

    // Scalar slider / chip + enum edits.
    let intents = ph2d_panel_motion_params::drain_param_intents();
    if !intents.is_empty() {
        // A discrete (typed) commit arrives with no bracket open → its own step.
        let discrete = !editing && !was;
        if discrete {
            motion.history.begin(&motion.doc);
        }
        // When the subject is a BACKDROP, the rows are its own (title / colour) and
        // the edits go to the document's decoration, never to a node. Routed by the
        // live SELECTION rather than by the intent's `node` field: an intent left
        // over from a previous frame carries an id from the old subject, and a node
        // id and a backdrop id are both just `u32` — reusing it would let a stale
        // rename land on whatever node happened to share the number.
        let backdrop = ph2d_panel_motion_graph::current_graph_backdrop_selection();
        // Same rule for a selected CARD (doc 57): a subgraph has no manifest, so its
        // one row (the name) is routed to the document's nesting, never to a node.
        let card = super::subgraph::selected_card();
        for intent in intents {
            // A backdrop edit never touches a node and never re-cooks (decoration
            // cannot change what the graph cooks).
            if let Some(bid) = backdrop {
                super::backdrops::apply_param_intent(motion, bid, intent);
                continue;
            }
            if let Some(sid) = card {
                super::subgraph::apply_param_intent(motion, super::subgraph::view_id(sid), intent);
                continue;
            }
            match intent {
                MotionParamIntent::SetParam { node, param, value } => {
                    let nid = NodeId(node);
                    let Some(inst) = motion.doc.graph.node(nid) else {
                        continue;
                    };
                    // A `channel` switch on a behaviour also resets its magnitude to that
                    // channel's sensible default (world units vs degrees vs scale) — same
                    // undo step, so Ctrl+Z restores the old values.
                    let channel_switch = param == "channel"
                        && (param_value(motion, nid, "channel") - value as f32).abs()
                            > f32::EPSILON;
                    let type_name = channel_switch.then(|| inst.type_name.clone());
                    motion.doc.graph.set_param(nid, param, value as f32);
                    if let Some(tn) = type_name {
                        apply_channel_presets(motion, nid, &tn, value as f32);
                    }
                    motion.pump.mark_dirty();
                }
                // A formula edit (a `motion.expression` text param) — the additive text
                // channel (docs/Motion Nodes/32-33).
                MotionParamIntent::SetTextParam { node, param, value } => {
                    let nid = NodeId(node);
                    if motion.doc.graph.node(nid).is_none() {
                        continue;
                    }
                    motion.doc.graph.set_text_param(nid, param, value);
                    motion.pump.mark_dirty();
                }
            }
        }
        if discrete {
            motion.history.commit_if_changed(&motion.doc);
        }
    }

    // Close the session bracket on the true→false edge (one step for the gesture).
    if !editing && was {
        motion.history.commit_if_changed(&motion.doc);
    }
}

/// The current value of one param on a node (per-instance override, else the
/// manifest default; unknown param → `0`).
pub(super) fn param_value(
    motion: &MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    name: &str,
) -> f32 {
    use ph2d_nodegraph::cook::OpResolver;
    let overrides = motion.doc.graph.node_param_overrides(nid);
    if let Some(v) = overrides.and_then(|m| m.get(name)).copied() {
        return v;
    }
    motion
        .doc
        .graph
        .node(nid)
        .and_then(|i| motion.registry.resolve(i.type_id()))
        .and_then(|op| op.manifest().params.iter().find(|p| p.name == name))
        .map_or(0.0, |p| p.default)
}

/// Reset a behaviour node's magnitude params to a sensible default for the newly
/// selected channel (#10 consistency). Switching what a stagger/oscillator drives
/// — X/Y position (world units) vs Rotation (degrees) vs Size (scale delta) —
/// rewrites the range so a `±1` meant for position doesn't read as a barely-there
/// ±1° / ±huge-scale on the other channels. Editor UX (not node math): it
/// runs on the channel switch inside the same undo step, so Ctrl+Z restores the
/// artist's previous values. Non-behaviour node types are a no-op.
pub(super) fn apply_channel_presets(
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
}

/// The `channel` enum's Rotation option (see the behaviours' `channel` hint:
/// `0` X, `1` Y, `2` Rotation, `3` Size).
const CHANNEL_ROTATION: i32 = 2;

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

/// A behaviour's magnitude param needs a **channel-aware widget range**, not just
/// a channel-aware value: a `ParamUiHint`'s range is static, and the behaviours'
/// were authored for position (`±10` world units). On the Rotation channel the
/// same param means DEGREES, where `±10` is a barely-visible tilt — and, worse, a
/// range that cannot even represent the `±90` preset: the slider would saturate,
/// display `-10`, and overwrite the doc with `-10` on the first touch.
///
/// Widen `[min, max]` so it **contains** `value` — the invariant every row must
/// satisfy before it reaches the panel.
///
/// A `ParamUiHint`'s range is a suggestion, not a constraint: `Graph::set_param`
/// never clamps, so a preset, an undo, or a loaded document can hold a value
/// outside it. A row whose range does not contain its value is a *lying widget* —
/// `normalized_track` clamps it to the track end, the panel PAINTS the clamped
/// number instead of the real one, and the first touch emits that clamped number
/// straight back into the doc, silently destroying the authored value. (That is
/// exactly the bug the Enio caught with Stagger on the Rotation channel.)
///
/// Containing the value costs a degraded slider span in the pathological case and
/// self-heals the moment the value returns inside the hint's range — a cheap
/// price for a widget that can never lie or destroy.
fn contain(min: f32, max: f32, value: f32) -> (f32, f32) {
    (min.min(value), max.max(value))
}

/// Returns `(min, max, step)` to use instead of the hint's, or `None` to keep it.
/// Only Rotation needs widening (Position / Size are already world-unit-scaled).
fn channel_range_override(type_name: &str, param: &str, channel: i32) -> Option<(f32, f32, f32)> {
    if channel != CHANNEL_ROTATION {
        return None;
    }
    // A full turn each way, dialled in whole degrees.
    const TURN: f32 = 360.0;
    match (type_name, param) {
        ("motion.stagger", "min" | "max") => Some((-TURN, TURN, 1.0)),
        ("motion.oscillator", "offset") => Some((-TURN, TURN, 1.0)),
        ("motion.oscillator" | "motion.wiggle", "amplitude") => Some((0.0, TURN, 1.0)),
        _ => None,
    }
}

/// The single selected Motion node's `NodeId.0`, or `None` unless exactly one
/// node is selected (params edit a single node; multi-select is a later step).
pub(super) fn selected_motion_node() -> Option<u32> {
    match ph2d_panel_motion_graph::current_graph_selection()[..] {
        [only] => Some(only),
        _ => None,
    }
}

/// Build the selected node's [`ParamsSnapshot`](ph2d_panel_motion_params::ParamsSnapshot)
/// (M1.P1): the display title + one row per declared `ParamSpec`, pairing each
/// with its `ParamUiHint` (range / widget / label) and its current value (the
/// per-instance override, else the manifest default). `None` unless exactly one
/// node is selected and resolvable.
pub(super) fn build_params_snapshot(
    motion: &MotionState,
) -> Option<ph2d_panel_motion_params::ParamsSnapshot> {
    use ph2d_node_registry::ParamWidget;
    use ph2d_nodegraph::cook::OpResolver;
    use ph2d_nodegraph::graph::NodeId;
    use ph2d_panel_motion_params::{
        AngleRow, ColorRow, EnumRow, ParamRow, ParamsSnapshot, ScalarRow, SeedRow, TextRow,
        ToggleRow,
    };

    // The params panel shows the properties of whatever ONE subject is selected.
    // A backdrop is not a node (no manifest, never cooks), so its rows are built by
    // the module that owns backdrops shell-side.
    if let Some(snap) = super::backdrops::params_snapshot(motion) {
        return Some(snap);
    }
    // A collapsed subgraph is not a node either (doc 57) — its rows are its own.
    if let Some(snap) = super::subgraph::params_snapshot(motion) {
        return Some(snap);
    }

    let only = selected_motion_node()?;
    let nid = NodeId(only);
    let inst = motion.doc.graph.node(nid)?;
    let type_id = inst.type_id();
    let manifest = motion.registry.resolve(type_id)?.manifest();
    let title = motion
        .registry
        .ui_manifest(type_id)
        .map(|u| u.display_name.to_string())
        .unwrap_or_else(|| inst.type_name.clone());
    let hints = motion.registry.param_ui(type_id);
    let overrides = motion.doc.graph.node_param_overrides(nid);
    let value_of = |name: &str| -> f32 {
        if let Some(v) = overrides.and_then(|m| m.get(name)).copied() {
            return v;
        }
        manifest
            .params
            .iter()
            .find(|p| p.name == name)
            .map_or(0.0, |p| p.default)
    };

    // Channels folded into a colour swatch — suppress their standalone rows.
    let consumed: Vec<&'static str> = color_groups(&motion.registry, type_id)
        .into_iter()
        .flatten()
        .collect();

    // The behaviour channel this node currently drives (`None` for a node that
    // declares no `channel` param) — it selects the magnitude rows' unit, and so
    // their widget range (see `channel_range_override`).
    let channel = manifest
        .params
        .iter()
        .any(|p| p.name == "channel")
        .then(|| value_of("channel").round() as i32);

    let mut rows: Vec<ParamRow> = Vec::new();

    // Text params (a `motion.expression` formula) are NOT `ParamSpec`s (f32-only), so they
    // never appear in the manifest loop below — surface each `ParamWidget::Text` hint as a
    // Text row first (the formula is the node's primary control), reading the graph's text
    // channel (docs/Motion Nodes/32-33).
    for h in hints.into_iter().flatten() {
        if h.widget == ParamWidget::Text {
            let value = motion
                .doc
                .graph
                .node_text_param_overrides(nid)
                .and_then(|m| m.get(h.param))
                .cloned()
                .unwrap_or_default();
            rows.push(ParamRow::Text(TextRow {
                name: h.param,
                label: h.label.to_string(),
                value,
            }));
        }
    }

    for spec in manifest.params {
        let hint = hints.and_then(|hs| hs.iter().find(|h| h.param == spec.name));
        // A `Color`-anchored param emits ONE swatch row for its 4 channels.
        if let Some(h) = hint
            && let ParamWidget::Color { channels } = h.widget
        {
            let lin = [
                value_of(channels[0]),
                value_of(channels[1]),
                value_of(channels[2]),
                value_of(channels[3]),
            ];
            rows.push(ParamRow::Color(ColorRow {
                label: h.label.to_string(),
                channels,
                srgb: linear_rgba_to_srgb8(lin),
            }));
            continue;
        }
        // A non-anchor colour channel is folded into its swatch — no scalar row.
        if consumed.contains(&spec.name) {
            continue;
        }
        // A boolean → a real checkbox; an enum → a named segmented selector; an
        // angle → a `deg` number box; a seed → a number box + re-roll button.
        if let Some(h) = hint {
            match h.widget {
                ParamWidget::Toggle => {
                    rows.push(ParamRow::Toggle(ToggleRow {
                        name: spec.name,
                        label: h.label.to_string(),
                        on: value_of(spec.name) >= 0.5,
                    }));
                    continue;
                }
                ParamWidget::Enum { labels } => {
                    let selected = (value_of(spec.name).round().max(0.0) as usize)
                        .min(labels.len().max(1) - 1);
                    rows.push(ParamRow::Enum(EnumRow {
                        name: spec.name,
                        label: h.label.to_string(),
                        selected,
                        labels,
                    }));
                    continue;
                }
                ParamWidget::Angle => {
                    // Degrees end to end — the param already stores what the
                    // `deg` box shows, so the row is a straight copy of the hint
                    // (widened to contain the value: a drag-scrub clamps to the
                    // registered range, which would eat an out-of-range angle).
                    let deg = value_of(spec.name);
                    let (min, max) = contain(h.min, h.max, deg);
                    rows.push(ParamRow::Angle(AngleRow {
                        name: spec.name,
                        label: h.label.to_string(),
                        deg: f64::from(deg),
                        min_deg: f64::from(min),
                        max_deg: f64::from(max),
                        step_deg: f64::from(h.step),
                    }));
                    continue;
                }
                ParamWidget::Seed => {
                    let seed = value_of(spec.name).round();
                    let (min, max) = contain(h.min, h.max, seed);
                    rows.push(ParamRow::Seed(SeedRow {
                        name: spec.name,
                        label: h.label.to_string(),
                        value: f64::from(seed),
                        min: f64::from(min),
                        max: f64::from(max),
                    }));
                    continue;
                }
                _ => {}
            }
        }
        let value = f64::from(value_of(spec.name));
        rows.push(ParamRow::Scalar(match hint {
            Some(h) => {
                // A behaviour's magnitude range depends on the channel it drives
                // (degrees on Rotation vs world units on X/Y) — the static hint
                // can only describe one, so widen it for ergonomics …
                let (min, max, step) = channel
                    .and_then(|ch| channel_range_override(&inst.type_name, spec.name, ch))
                    .unwrap_or((h.min, h.max, h.step));
                // … then widen it again, unconditionally, so it CONTAINS the doc
                // value. That is the correctness half: no clamp, no lie, no
                // destroy-on-touch, whatever put the value there.
                let (min, max) = contain(min, max, value_of(spec.name));
                ScalarRow {
                    name: spec.name,
                    label: h.label.to_string(),
                    value,
                    min: f64::from(min),
                    max: f64::from(max),
                    step: f64::from(step),
                    integer: h.widget.is_integer(),
                }
            }
            // No hint → a neutral range around the param's manifest DEFAULT.
            //
            // **A range must never be a function of the value it ranges over.** A slider's
            // range is the SCALE the value is measured against, and a scale that grows with
            // what it measures is a positive feedback loop. This branch used to read
            // `max = value * 4`, whose fixed point sits at a quarter of the track: drag above
            // it and the value multiplied every frame — to billions in about a second — and
            // drag below it and it collapsed to zero. Neither is a drag; both are a runaway.
            // (Enio, smoke 2026-07-12: *"sliders chegam a bilhões e não arrastam linearmente"*.)
            //
            // The DEFAULT is a manifest constant, so the scale holds still while the knob
            // moves across it. `contain` may still widen the range to hold an out-of-range doc
            // value, which is idempotent — widening for a value the range already holds is a
            // no-op, so a drag inside the range never moves it.
            //
            // Every registered param is hinted (`every_scalar_row_comes_from_a_declared_hint`
            // is the gate), so this branch is the backstop for a node type that forgets one —
            // and a backstop must be INERT, not armed.
            None => {
                let neutral = (spec.default.abs() * 4.0).max(10.0);
                let (min, max) = contain(0.0, neutral, value_of(spec.name));
                ScalarRow {
                    name: spec.name,
                    label: spec.name.to_string(),
                    value,
                    min: f64::from(min),
                    max: f64::from(max),
                    step: 0.1,
                    integer: false,
                }
            }
        }));
    }
    Some(ParamsSnapshot {
        node: only,
        title,
        rows,
    })
}
