//! Params-panel side of the Motion bridge (split out of `motion_bridge.rs` to
//! keep each file under the HR-18 LOC cap). Owns the selected node's param edits
//! and the params-panel snapshot: scalar sliders, named enum / checkbox rows,
//! and the OKLCH colour-swatch authoring (the sRGB↔linear boundary lives here).
//!
//! The whole module is gated on both Motion panels — its parent declares it with
//! the same `all(panel-motion-graph, panel-motion-params)` cfg — so the helpers
//! carry no per-fn gate. All entry points are `pub(super)` for `dispatch`.

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
        for MotionParamIntent::SetParam { node, param, value } in intents {
            let nid = NodeId(node);
            let Some(inst) = motion.doc.graph.node(nid) else {
                continue;
            };
            // A `channel` switch on a behaviour also resets its magnitude to that
            // channel's sensible default (world units vs degrees vs scale) — same
            // undo step, so Ctrl+Z restores the old values.
            let channel_switch = param == "channel"
                && (param_value(motion, nid, "channel") - value as f32).abs() > f32::EPSILON;
            let type_name = channel_switch.then(|| inst.type_name.clone());
            motion.doc.graph.set_param(nid, param, value as f32);
            if let Some(tn) = type_name {
                apply_channel_presets(motion, nid, &tn, value as f32);
            }
            motion.pump.mark_dirty();
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
        "motion.oscillator" => {
            motion
                .doc
                .graph
                .set_param(nid, "amplitude", oscillator_channel_amplitude(ch));
        }
        _ => {}
    }
}

/// Stagger `(min, max)` ramp endpoints per channel. The Rotation channel writes
/// the `rot` stream column, whose unit is **degrees** (the app's authored-angle
/// unit); Position is world units, Size a scale delta.
fn stagger_channel_range(channel: i32) -> (f32, f32) {
    match channel {
        2 => (-90.0, 90.0), // Rotation: ±90 degrees
        3 => (-0.5, 0.5),   // Size: ±0.5 scale
        _ => (-1.0, 1.0),   // Position (X/Y): ±1 world unit
    }
}

/// Oscillator peak `amplitude` per channel (same unit logic as the stagger range).
fn oscillator_channel_amplitude(channel: i32) -> f32 {
    match channel {
        2 => 30.0, // Rotation: ±30 degrees
        3 => 0.3,  // Size: ±0.3 scale
        _ => 1.0,  // Position: ±1 world unit
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

/// The colour groups declared by a node type — the 4-channel RGBA param names
/// behind each [`ParamWidget::Color`](ph2d_node_registry::ParamWidget) hint.
pub(super) fn color_groups(
    registry: &ph2d_node_registry::NodeRegistry,
    type_id: ph2d_nodegraph::node::NodeTypeId,
) -> Vec<[&'static str; 4]> {
    use ph2d_node_registry::ParamWidget;
    registry
        .param_ui(type_id)
        .into_iter()
        .flatten()
        .filter_map(|h| match h.widget {
            ParamWidget::Color { channels } => Some(channels),
            _ => None,
        })
        .collect()
}

/// The current linear-straight values of a node's 4 colour channels (per-instance
/// override, else the manifest default). Shared by the read-back change-guard and
/// the snapshot builder so the swatch and the doc agree.
pub(super) fn channel_values(
    motion: &MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    channels: [&'static str; 4],
) -> [f32; 4] {
    use ph2d_nodegraph::cook::OpResolver;
    let overrides = motion.doc.graph.node_param_overrides(nid);
    let manifest = motion
        .doc
        .graph
        .node(nid)
        .and_then(|i| motion.registry.resolve(i.type_id()))
        .map(|op| op.manifest());
    let value_of = |name: &str| -> f32 {
        if let Some(v) = overrides.and_then(|m| m.get(name)).copied() {
            return v;
        }
        manifest
            .and_then(|m| m.params.iter().find(|p| p.name == name))
            .map_or(0.0, |p| p.default)
    };
    [
        value_of(channels[0]),
        value_of(channels[1]),
        value_of(channels[2]),
        value_of(channels[3]),
    ]
}

/// Write a picked sRGB colour into a node's 4 linear-straight channel params
/// (RGB via the sRGB transfer function, alpha straight), re-cooking only when the
/// colour actually changed (the picker stays open across idle frames).
pub(super) fn apply_color_to_node(
    motion: &mut MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    channels: [&'static str; 4],
    srgb: [u8; 4],
) {
    let new = srgb8_to_linear_rgba(srgb);
    let cur = channel_values(motion, nid, channels);
    if !new
        .into_iter()
        .zip(cur)
        .any(|(n, c)| (n - c).abs() > f32::EPSILON)
    {
        return;
    }
    for (name, v) in channels.into_iter().zip(new) {
        motion.doc.graph.set_param(nid, name, v);
    }
    motion.pump.mark_dirty();
}

/// Seed each colour swatch's `widget_color` from the snapshot's display colour
/// (the OKLCH picker reads it on open + the swatch paints it). Keyed by the
/// anchor channel — the same id the panel registers.
pub(super) fn seed_color_swatches(
    store: &mut ph2d_editor::interaction::WidgetStore,
    snap: &ph2d_panel_motion_params::ParamsSnapshot,
) {
    use ph2d_panel_motion_params::{ParamRow, param_swatch_id};
    for row in &snap.rows {
        if let ParamRow::Color(c) = row {
            store.set_widget_color(param_swatch_id(c.channels[0]), c.srgb);
        }
    }
}

/// sRGB8 (straight) → linear-straight RGBA `[0,1]` (the Motion wire space): RGB
/// through the sRGB transfer function, alpha a plain `/255`.
fn srgb8_to_linear_rgba(srgb: [u8; 4]) -> [f32; 4] {
    use ph2d_color::srgb::srgb_to_linear_byte;
    [
        srgb_to_linear_byte(srgb[0]),
        srgb_to_linear_byte(srgb[1]),
        srgb_to_linear_byte(srgb[2]),
        f32::from(srgb[3]) / 255.0,
    ]
}

/// Linear-straight RGBA `[0,1]` → sRGB8 (straight) for the swatch display /
/// picker seed: RGB through the linear→sRGB transfer, alpha a plain `×255`.
pub(super) fn linear_rgba_to_srgb8(lin: [f32; 4]) -> [u8; 4] {
    use ph2d_color::srgb::linear_to_srgb_byte;
    [
        linear_to_srgb_byte(lin[0]),
        linear_to_srgb_byte(lin[1]),
        linear_to_srgb_byte(lin[2]),
        (lin[3].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
    ]
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
        AngleRow, ColorRow, EnumRow, ParamRow, ParamsSnapshot, ScalarRow, SeedRow, ToggleRow,
    };

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

    let mut rows: Vec<ParamRow> = Vec::new();
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
                    // `deg` box shows, so the row is a straight copy of the hint.
                    rows.push(ParamRow::Angle(AngleRow {
                        name: spec.name,
                        label: h.label.to_string(),
                        deg: f64::from(value_of(spec.name)),
                        min_deg: f64::from(h.min),
                        max_deg: f64::from(h.max),
                        step_deg: f64::from(h.step),
                    }));
                    continue;
                }
                ParamWidget::Seed => {
                    rows.push(ParamRow::Seed(SeedRow {
                        name: spec.name,
                        label: h.label.to_string(),
                        value: f64::from(value_of(spec.name).round()),
                        min: f64::from(h.min),
                        max: f64::from(h.max),
                    }));
                    continue;
                }
                _ => {}
            }
        }
        let value = f64::from(value_of(spec.name));
        rows.push(ParamRow::Scalar(match hint {
            Some(h) => ScalarRow {
                name: spec.name,
                label: h.label.to_string(),
                value,
                min: f64::from(h.min),
                max: f64::from(h.max),
                step: f64::from(h.step),
                integer: h.widget.is_integer(),
            },
            // No hint → a plain float slider over a neutral range.
            None => ScalarRow {
                name: spec.name,
                label: spec.name.to_string(),
                value,
                min: 0.0,
                max: (value * 4.0).max(10.0),
                step: 0.1,
                integer: false,
            },
        }));
    }
    Some(ParamsSnapshot {
        node: only,
        title,
        rows,
    })
}
