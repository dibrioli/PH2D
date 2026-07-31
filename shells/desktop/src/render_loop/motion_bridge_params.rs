//! Params-panel side of the Motion bridge (split out of `motion_bridge.rs` to
//! keep each file under the HR-18 LOC cap). Owns the selected node's param edits
//! and the params-panel snapshot: scalar sliders, named enum / checkbox rows,
//! and the OKLCH colour-swatch authoring (the sRGB↔linear boundary lives here).
//!
//! The whole module is gated on both Motion panels — its parent declares it with
//! the same `all(panel-motion-graph, panel-motion-params)` cfg — so the helpers
//! carry no per-fn gate. All entry points are `pub(super)` for `dispatch`.

use super::color::{color_groups, linear_rgba_to_srgb8};
use crate::motion_state::MotionState;

/// The peek/stream-reading helpers (the live number a wire drives, the live columns
/// the Custom picker offers) — a child so `motion_bridge.rs` stays under the cap.
#[path = "motion_bridge_params_stream.rs"]
mod params_stream;

/// The channel-aware magnitude presets (a sibling child, shell LOC cap).
#[path = "motion_bridge_params_channel.rs"]
mod params_channel;
/// Re-exported under its old path so the params tests (a sibling module) and this
/// module keep naming the presets `params::apply_channel_presets` after the split.
pub(super) use params_channel::apply_channel_presets;

/// **Publish the params panel this frame** (M1.P1) — the dispatch's one call: apply the
/// selected node's edits, rebuild its snapshot, seed the colour swatches from it, hand it
/// to the panel. Needs BOTH motion panels (selection from the graph, rows to params); it
/// lives here rather than inline in `dispatch` so the shell dispatch stays under the LOC
/// cap and the params logic sits together.
pub(super) fn publish(
    motion: &mut MotionState,
    store: &mut ph2d_editor::interaction::WidgetStore,
    motion_active: bool,
) {
    if !motion_active {
        ph2d_panel_motion_params::set_current_params(None);
        return;
    }
    // Apply this frame's edits (colour picks + scalar sliders) BEFORE rebuilding, so the
    // panel reflects them; then seed each colour swatch's picker from the fresh snapshot.
    apply_param_edits(motion, store);
    let snap = build_params_snapshot(motion);
    if let Some(s) = &snap {
        super::color::seed_color_swatches(store, s);
    }
    ph2d_panel_motion_params::set_current_params(snap);
}

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
    use ph2d_panel_motion_params::{MotionParamIntent, any_param_editing};
    use std::sync::atomic::{AtomicBool, Ordering};
    static PARAM_EDITING: AtomicBool = AtomicBool::new(false);

    // The selected node + its colour groups (each = 4 RGBA channel params driven
    // by one swatch → OKLCH picker).
    let sel = selected_motion_node().map(NodeId);
    let type_id = sel.and_then(|nid| motion.doc.graph.node(nid).map(|i| i.type_id()));
    let groups = type_id
        .map(|tid| color_groups(&motion.registry, tid))
        .unwrap_or_default();
    // The gradient text params (doc 85) — each stop's swatch feeds the SAME OKLCH picker.
    let grad_params = type_id
        .map(|tid| super::color::gradient_params(&motion.registry, tid))
        .unwrap_or_default();

    // A colour-swatch OR gradient-stop pick is an editing session (like a slider drag): its
    // live writes coalesce into ONE undo step, opened here + committed on close. Detected
    // BEFORE the bracket; the read-back writes go INSIDE it.
    let session = super::color::picker_session(motion, sel, &groups, &grad_params, store);
    let editing = any_param_editing(store) || session;
    let was = PARAM_EDITING.swap(editing, Ordering::Relaxed);
    if editing && !was {
        motion.history.begin(&motion.doc);
    }

    // Colour + gradient-stop read-back: feed the live pick into the params/string it targets
    // (sRGB→linear), re-cooking only on an actual change (the picker stays open across idle
    // frames). One door in `color.rs` (the sRGB↔linear boundary).
    super::color::apply_picker_readback(motion, sel, &groups, &grad_params, store);

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
                    // ADR-0130 D7: an edit that re-numbers the emitter's ids
                    // (rate/life/max) moves the id↔particle map, so the GPU sim's
                    // paired state would mispair the new window against the old.
                    // Decided BEFORE `set_param` consumes `param`, and only when
                    // the value actually MOVES — a slider re-emits its intent every
                    // frame of a gesture, and restarting the sim on a value that
                    // did not change would keep it pinned at the seed.
                    let renumbers_sim = super::gpu::edit_renumbers_emitter(&inst.type_name, param)
                        && (param_value(motion, nid, param) - value as f32).abs() > f32::EPSILON;
                    motion.doc.graph.set_param(nid, param, value as f32);
                    if let Some(tn) = type_name {
                        apply_channel_presets(motion, nid, &tn, value as f32);
                    }
                    motion.pump.mark_dirty();
                    if renumbers_sim {
                        // RESTART from the tick on screen — never `forget_state`,
                        // whose re-derive is O(current tick) and freezes a drag.
                        motion.gpu_cook.reseed_from_next_tick();
                    }
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
pub(crate) fn param_value(
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
    if channel != params_channel::CHANNEL_ROTATION {
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
pub(crate) fn selected_motion_node() -> Option<u32> {
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
        AngleRow, ChannelsRow, ColorRow, CurveRow, EnumRow, GradientRow, ParamRow, ParamsSnapshot,
        ScalarRow, SeedRow, SourceRow, TextRow, ToggleRow,
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

    // Channels folded into a colour swatch (or into a `Channels` picker's `mode`) —
    // suppress their standalone rows.
    let mut consumed: Vec<&'static str> = color_groups(&motion.registry, type_id)
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

    // Text params (a `motion.expression` formula, a `field.remap` Curve) are NOT
    // `ParamSpec`s (f32-only), so they never appear in the manifest loop below — surface
    // each as a row FIRST, reading the graph's text channel (docs/Motion Nodes/32-33).
    // `Curve` rides the SAME text channel as `Text` but paints an interactive curve editor
    // (A1-ui): draggable handles over the serialized curve, never the string.
    for h in hints.into_iter().flatten() {
        // A named-channel picker (plan §1.1): the artist-facing face of a stream-column
        // TEXT param. It reads the live column (a text param) + its `mode` (an f32
        // param), matches them to a channel, and folds `mode` in (consumed, so it gets
        // no row of its own). Custom = no match → the raw text field is offered.
        if let ParamWidget::Channels {
            mode_param,
            channels,
        } = h.widget
        {
            let attr = motion
                .doc
                .graph
                .node_text_param_overrides(nid)
                .and_then(|m| m.get(h.param))
                .cloned()
                .unwrap_or_default();
            let mode = value_of(mode_param).round() as i32;
            let selected = channels
                .iter()
                .position(|c| c.column == attr && c.mode == mode)
                .unwrap_or(channels.len());
            // The live upstream columns the Custom picker offers (roadmap: dropdown
            // populated at runtime) — everything the curated channels already cover
            // is excluded, so the chips are the ADVANCED columns, not duplicates.
            let covered: std::collections::BTreeSet<&str> =
                channels.iter().map(|c| c.column).collect();
            let extra = params_stream::upstream_scalar_columns(motion, nid, &covered, &attr);
            rows.push(ParamRow::Channels(ChannelsRow {
                label: h.label.to_string(),
                text_param: h.param,
                mode_param,
                // Resolve to primitives so the panel needs no registry dependency.
                channels: channels
                    .iter()
                    .map(|c| (c.label, c.column, c.mode))
                    .collect(),
                selected,
                custom: attr,
                extra,
            }));
            consumed.push(mode_param);
            continue;
        }
        // A source picker (doc 65): a TEXT param that names a value the app published
        // into the external channel. The options are the LIVE published names — the same
        // `Cook::externals` the node reads from — so the artist picks a shape they drew
        // by name. `motion.pump.cook` holds the externals whether the graph cooked on the
        // CPU or the GPU (the shell republishes them every frame, ADR-0126-independent).
        if h.widget == ParamWidget::Source {
            let current = motion
                .doc
                .graph
                .node_text_param_overrides(nid)
                .and_then(|m| m.get(h.param))
                .cloned()
                .unwrap_or_default();
            let options = params_stream::source_options(motion);
            rows.push(ParamRow::Source(SourceRow {
                label: h.label.to_string(),
                param: h.param,
                options,
                current,
            }));
            continue;
        }
        // A Gradient editor (doc 85) — a `ColorRamp` in a text param (`serialize_gradient`),
        // the colour sibling of the Curve. The panel draws the bar + draggable stops from the
        // string; a stop's COLOUR is read back through the OKLCH picker below.
        if h.widget == ParamWidget::Gradient {
            let value = motion
                .doc
                .graph
                .node_text_param_overrides(nid)
                .and_then(|m| m.get(h.param))
                .cloned()
                .unwrap_or_default();
            rows.push(ParamRow::Gradient(GradientRow {
                name: h.param,
                label: h.label.to_string(),
                value,
            }));
            continue;
        }
        if h.widget == ParamWidget::Text || h.widget == ParamWidget::Curve {
            let value = motion
                .doc
                .graph
                .node_text_param_overrides(nid)
                .and_then(|m| m.get(h.param))
                .cloned()
                .unwrap_or_default();
            let name = h.param;
            let label = h.label.to_string();
            rows.push(if h.widget == ParamWidget::Curve {
                ParamRow::Curve(CurveRow { name, label, value })
            } else {
                ParamRow::Text(TextRow { name, label, value })
            });
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
        // **A driven param shows the number the WIRE is putting in** (doc 58), not the
        // override the wire is overriding — the resolution order the cook uses, made visible.
        // Read from the cook's memo (`peek`), never by evaluating anything a second time.
        let driven = params_stream::driven_value(motion, nid, spec.name);
        let value = f64::from(driven.unwrap_or_else(|| value_of(spec.name)));
        let driven = driven.is_some();
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
                // The typed ceiling, when the node declared one wider than the
                // drag range. `max` is the floor of it: a hard limit that sat
                // BELOW the slider would silently un-type values the slider can
                // still reach.
                let hard_max = motion
                    .registry
                    .param_hard_max(type_id, spec.name)
                    .unwrap_or(max)
                    .max(max);
                ScalarRow {
                    name: spec.name,
                    label: h.label.to_string(),
                    value,
                    min: f64::from(min),
                    max: f64::from(max),
                    hard_max: f64::from(hard_max),
                    step: f64::from(step),
                    integer: h.widget.is_integer(),
                    driven,
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
                    hard_max: f64::from(max),
                    step: 0.1,
                    integer: false,
                    driven,
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
