//! Motion Nodes tool ⟷ shell bridge (Motion Nodes M0.T10). Replaces the retired
//! `motion_smoke` debug path with the production per-frame cook.
//!
//! Per-frame jobs (mirror of `vector_bridge`), all no-ops unless the `motion`
//! tool is active:
//!
//! 1. **Panel visibility** — show the docked graph + params panels; hide the
//!    real Inspector (edge-triggered) so they don't both claim the slot.
//! 2. **Center split** — split the center into scene ⟂ graph on activate
//!    (remembered orientation, default `Horizontal { t: 0.55 }`), restore to
//!    `None` on deactivate.
//! 3. **Per-frame cook** — advance the transport by this frame's fixed steps,
//!    then cook the graph's sink into the reused `MotionState.instances` buffer.
//!    The render loop injects that slice via `SpriteRenderer::render_with_extra`
//!    (`present.rs`) — the cooked stream draws without being spawned into the
//!    ECS `PresentWorld` (stream ≠ ECS, ADR-0035).
//!
//! **Zero concrete-tool downcast:** unlike `vector_bridge`, the document lives in
//! `MotionState` (shell), not the tool, so the central render loop stays
//! downcast-free without this bridge reaching into a concrete tool at all.

use crate::motion_state::MotionState;
use ph2d_editor::screens::layout::CenterSplit;
use ph2d_editor::{HeroScreen, ToastQueue, ToolId, ToolRegistry};

/// Per-frame Motion-tool plumbing. Safe to call every frame; a no-op when the
/// Motion tool is inactive (beyond flipping panel visibility / the split off).
///
/// - `frame_ticks`: fixed steps this frame (`FixedStepReport::ticks`) — advances
///   the transport when playing.
/// - `fixed_dt`: the fixed timestep in seconds (playhead = `tick × fixed_dt`).
/// - `cursor`: the latest pointer position (screen px) — drives the cursor-gated
///   graph keyboard focus (Blender-style F acts on the hovered area).
/// - `toasts`: the shell toast queue — the connect authority raises a refusal
///   toast here when a dragged edge is rejected (cycle / occupied / typing /
///   membrane).
#[cfg_attr(not(feature = "panel-motion-graph"), allow(unused_variables))]
pub(super) fn dispatch(
    hero: &mut HeroScreen,
    tools: &ToolRegistry,
    motion: &mut MotionState,
    frame_ticks: u32,
    fixed_dt: f64,
    cursor: (f32, f32),
    toasts: &mut ToastQueue,
) {
    let motion_active = tools
        .active()
        .is_some_and(|t| t.id() == ToolId::new("motion"));

    // ── 1. Panel visibility (mirror of the Vector dock takeover) ──────────
    hero.panel_visibility.insert("motion_graph", motion_active);
    hero.panel_visibility.insert("motion_params", motion_active);

    // Graph keyboard focus follows the cursor, re-evaluated EVERY frame (not just
    // on move) so a cursor that stopped over the graph before the panel published
    // its rect still gets focus by the time a key is pressed. `panel_rect` is from
    // last frame's paint (stable); `None` off the graph → the scene owns keys.
    let over_graph = motion_active
        && hero
            .store
            .panel_rect(ph2d_editor::ids::MOTION_GRAPH_PANEL)
            .is_some_and(|r| r.contains(cursor.0, cursor.1));
    hero.store
        .set_graph_focused(over_graph.then_some(ph2d_editor::ids::MOTION_GRAPH_PANEL));

    // ── 2. Center split + Inspector takeover — edge-triggered on activation ──
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        static LAST_ACTIVE: AtomicBool = AtomicBool::new(false);
        let was = LAST_ACTIVE.swap(motion_active, Ordering::Relaxed);
        if was != motion_active {
            hero.panel_visibility.insert("inspector", !motion_active);
            if motion_active {
                // Split into scene ⟂ graph. Keep any orientation the user already
                // chose (SplitH/SplitV chips); default to Cavalry-style horizontal.
                if !hero.view.center_split.is_split() {
                    hero.view.center_split = CenterSplit::Horizontal {
                        t: CenterSplit::T_DEFAULT,
                    };
                }
                // Auto-play on entry so time-driven behaviours animate live the
                // moment the tool opens (Cavalry/AE preview semantics). Space
                // toggles pause; nothing moves until a `Temporal` node is wired.
                motion.transport.play();
            } else {
                hero.view.center_split = CenterSplit::None;
            }
        }
    }

    // ── Apply the panel's edits, then publish the fresh view (M1.E10) ──────
    // The panel pushed `GraphIntent`s during last frame's paint; apply them to
    // the doc (each a single undo step) BEFORE rebuilding the snapshot so the
    // change shows this frame. Rebuilt each active frame (Phase 1a); a dirty
    // gate lands later. `None` while inactive → no allocation off the editor.
    #[cfg(feature = "panel-motion-graph")]
    {
        if motion_active {
            apply_graph_intents(motion, toasts, &mut hero.view.center_split);
            // Publish the addable-node catalog for the add-menu. Rebuilt each
            // active frame (cheap: ~dozens of `Copy` entries) alongside the
            // snapshot; memoizing it is a follow-up like the snapshot's own
            // dirty gate.
            ph2d_panel_motion_graph::set_current_node_catalog(build_catalog(&motion.registry));
        } else {
            ph2d_panel_motion_graph::set_current_node_catalog(Vec::new());
        }
        ph2d_panel_motion_graph::set_current_motion_graph(
            motion_active.then(|| {
                ph2d_panel_motion_graph::snapshot_from(&motion.doc.graph, &motion.registry)
            }),
        );
    }

    // ── Params panel (M1.P1): apply the selected node's param edits, then
    // publish its params snapshot ────────────────────────────────────────────
    // Needs BOTH panels: the selection comes from the graph panel, the rows go
    // to the params panel. Apply BEFORE the cook so a param change shows this
    // frame; publish the freshly-mutated doc so the panel reflects it.
    #[cfg(all(feature = "panel-motion-graph", feature = "panel-motion-params"))]
    {
        if motion_active {
            // Apply this frame's edits (colour picks + scalar sliders) BEFORE
            // rebuilding, so the panel reflects them; then seed each colour
            // swatch's picker colour from the freshly-mutated snapshot.
            apply_param_edits(motion, &hero.store);
            let snap = build_params_snapshot(motion);
            if let Some(s) = &snap {
                seed_color_swatches(&mut hero.store, s);
            }
            ph2d_panel_motion_params::set_current_params(snap);
        } else {
            ph2d_panel_motion_params::set_current_params(None);
        }
    }

    if !motion_active {
        return;
    }

    // ── 3. Advance transport + cook the sink (skipped when paused/unchanged) ──
    // `advance` is a no-op while paused, so the tick only moves when playing or
    // scrubbing → the pump skips a static paused frame (zero-alloc, M0.T12). No
    // framing node in M0, so the pump samples one opaque atlas tile (set in
    // init.rs) at a sub-spacing size → clean, distinct dots.
    motion.transport.advance(frame_ticks as u64);
    let playhead = motion.playhead(fixed_dt);
    let tick = motion.transport.tick;
    motion.pump.pump(
        &motion.doc.graph,
        &motion.registry,
        motion.sink,
        tick,
        playhead,
        motion.default_uv_rect,
        motion.default_size,
    );
}

/// Apply the panel's queued [`GraphIntent`]s to the shell-owned document (M1.E10).
///
/// - **Drag** (`BeginDrag`/`MoveNodes`/`EndDrag`) is a live sequence: the bracket
///   opens the undo step, each incremental delta applies immediately (so the node
///   tracks the cursor with no end-jump), and the release commits one step.
///   Positions are UI-only (they never touch the cook) → no `mark_dirty`.
/// - **Structural** edits (`Connect`/`Disconnect`/`AddNode`/`DeleteSelection`)
///   each are one atomic undo step and change the cook → `mark_dirty`. Connect is
///   validated here (the shell is the authority): a trial clone runs
///   `Graph::connect` (cycle / occupied-input) then `Graph::validate` (typing /
///   membrane), and the edit is kept only when the new edge is legal — else a
///   refusal toast is raised and the document is untouched.
#[cfg(feature = "panel-motion-graph")]
fn apply_graph_intents(motion: &mut MotionState, toasts: &mut ToastQueue, split: &mut CenterSplit) {
    use ph2d_editor::Toast;
    use ph2d_nodegraph::graph::{Edge, NodeId, Pos};
    use ph2d_panel_motion_graph::GraphIntent;
    for intent in ph2d_panel_motion_graph::drain_intents() {
        match intent {
            GraphIntent::BeginDrag => motion.history.begin(&motion.doc),
            GraphIntent::MoveNodes { nodes, dx, dy } => {
                for id in nodes {
                    let nid = NodeId(id);
                    if let Some(p) = motion.doc.graph.pos(nid) {
                        motion.doc.graph.set_pos(
                            nid,
                            Pos {
                                x: p.x + dx,
                                y: p.y + dy,
                            },
                        );
                    }
                }
            }
            GraphIntent::EndDrag => motion.history.commit_if_changed(&motion.doc),
            GraphIntent::Connect {
                from_node,
                from_port,
                to_node,
                to_port,
            } => {
                let edge = Edge {
                    from: (NodeId(from_node), from_port),
                    to: (NodeId(to_node), to_port),
                    delayed: false,
                };
                let mut trial = motion.doc.graph.clone();
                match trial.connect(edge) {
                    Err(e) => {
                        toasts.push(Toast::warning(connect_err_msg(e)));
                    }
                    Ok(()) => {
                        let rejected = match trial.validate(&motion.registry) {
                            Ok(()) => false,
                            Err(viols) => viols.iter().any(|v| violation_blocks_edge(v, edge)),
                        };
                        if rejected {
                            toasts.push(Toast::warning("Can't connect: incompatible ports"));
                        } else {
                            let pre = motion.doc.clone();
                            motion.doc.graph = trial;
                            motion.history.push_undo(pre);
                            motion.pump.mark_dirty();
                        }
                    }
                }
            }
            GraphIntent::Disconnect { to_node, to_port } => {
                let pre = motion.doc.clone();
                if motion
                    .doc
                    .graph
                    .disconnect(NodeId(to_node), to_port)
                    .is_some()
                {
                    motion.history.push_undo(pre);
                    motion.pump.mark_dirty();
                }
            }
            GraphIntent::AddNode { type_name, x, y } => {
                let pre = motion.doc.clone();
                let id = motion.doc.graph.add_node(type_name);
                motion.doc.graph.set_pos(id, Pos { x, y });
                motion.history.push_undo(pre);
                motion.pump.mark_dirty();
            }
            GraphIntent::DeleteSelection { nodes } => {
                let pre = motion.doc.clone();
                let mut changed = false;
                for id in nodes {
                    changed |= motion.doc.graph.remove_node(NodeId(id));
                }
                if changed {
                    // Deleting the sink stops the cook cleanly rather than
                    // pointing it at a phantom node.
                    if let Some(s) = motion.sink
                        && motion.doc.graph.node(s).is_none()
                    {
                        motion.sink = None;
                    }
                    motion.history.push_undo(pre);
                    motion.pump.mark_dirty();
                }
            }
            // Split chrome (E9) — UI-only (no cook / undo). `with_t` clamps the
            // fraction; orientation flips preserve it.
            GraphIntent::SetSplit { t } => {
                if split.is_split() {
                    *split = split.with_t(t);
                }
            }
            GraphIntent::SetSplitVertical { vertical } => {
                *split = if vertical {
                    split.to_vertical()
                } else {
                    split.to_horizontal()
                };
            }
            // Transport play/pause (Space) — the shell owns the transport; no
            // doc edit / undo. The per-frame cook advances only while playing.
            GraphIntent::TogglePlay => {
                motion.transport.toggle();
            }
            // Set the render output (O) — the node whose cooked stream is lowered
            // to instances. An output choice, not a doc edit (no undo); re-cooks.
            GraphIntent::SetSink { node } => {
                let nid = NodeId(node);
                if motion.doc.graph.node(nid).is_some() {
                    motion.sink = Some(nid);
                    motion.pump.mark_dirty();
                }
            }
        }
    }
}

/// Human-readable reason a structural `connect` was rejected (add-menu toast).
#[cfg(feature = "panel-motion-graph")]
fn connect_err_msg(e: ph2d_nodegraph::graph::EdgeError) -> &'static str {
    use ph2d_nodegraph::graph::EdgeError;
    match e {
        EdgeError::WouldCycle => "Can't connect: would create a cycle",
        EdgeError::InputAlreadyConnected => "Can't connect: input already wired",
        EdgeError::UnknownNode => "Can't connect: unknown node",
    }
}

/// Does a validation violation reject *this* just-added edge (as opposed to a
/// pre-existing problem elsewhere)? Only a type mismatch or a membrane crossing
/// on the same endpoints blocks the connect.
#[cfg(feature = "panel-motion-graph")]
fn violation_blocks_edge(
    v: &ph2d_nodegraph::graph::Violation,
    edge: ph2d_nodegraph::graph::Edge,
) -> bool {
    use ph2d_nodegraph::graph::Violation;
    match v {
        Violation::TypeMismatch { from, to } | Violation::Membrane { from, to } => {
            *from == edge.from && *to == edge.to
        }
        _ => false,
    }
}

/// Build the addable-node catalog from the registry (canonical name + English
/// display label + category), sorted by category then label so the menu groups
/// by color (the palette teaches the library map, plan §2.4).
#[cfg(feature = "panel-motion-graph")]
fn build_catalog(
    registry: &ph2d_node_registry::NodeRegistry,
) -> Vec<ph2d_panel_motion_graph::NodeChoice> {
    use ph2d_node_registry::NodeUiCategory;
    use ph2d_panel_motion_graph::NodeChoice;
    let mut v: Vec<NodeChoice> = registry
        .manifests()
        .map(|m| {
            let ui = registry.ui_manifest(m.id);
            NodeChoice {
                type_name: m.name,
                display: ui.map(|u| u.display_name).unwrap_or(m.name),
                category: ui.map(|u| u.category).unwrap_or(NodeUiCategory::Utility),
            }
        })
        .collect();
    v.sort_by(|a, b| (a.category as u8, a.display).cmp(&(b.category as u8, b.display)));
    v
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
#[cfg(all(feature = "panel-motion-graph", feature = "panel-motion-params"))]
fn apply_param_edits(motion: &mut MotionState, store: &ph2d_editor::interaction::WidgetStore) {
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

    // Scalar slider / chip edits.
    let intents = ph2d_panel_motion_params::drain_param_intents();
    if !intents.is_empty() {
        // A discrete (typed) commit arrives with no bracket open → its own step.
        let discrete = !editing && !was;
        if discrete {
            motion.history.begin(&motion.doc);
        }
        for MotionParamIntent::SetParam { node, param, value } in intents {
            let nid = NodeId(node);
            if motion.doc.graph.node(nid).is_some() {
                motion.doc.graph.set_param(nid, param, value as f32);
                motion.pump.mark_dirty();
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

/// The single selected Motion node's `NodeId.0`, or `None` unless exactly one
/// node is selected (params edit a single node; multi-select is a later step).
#[cfg(all(feature = "panel-motion-graph", feature = "panel-motion-params"))]
fn selected_motion_node() -> Option<u32> {
    match ph2d_panel_motion_graph::current_graph_selection()[..] {
        [only] => Some(only),
        _ => None,
    }
}

/// The colour groups declared by a node type — the 4-channel RGBA param names
/// behind each [`ParamWidget::Color`](ph2d_node_registry::ParamWidget) hint.
#[cfg(all(feature = "panel-motion-graph", feature = "panel-motion-params"))]
fn color_groups(
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
#[cfg(all(feature = "panel-motion-graph", feature = "panel-motion-params"))]
fn channel_values(
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
#[cfg(all(feature = "panel-motion-graph", feature = "panel-motion-params"))]
fn apply_color_to_node(
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
#[cfg(all(feature = "panel-motion-graph", feature = "panel-motion-params"))]
fn seed_color_swatches(
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
#[cfg(all(feature = "panel-motion-graph", feature = "panel-motion-params"))]
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
#[cfg(all(feature = "panel-motion-graph", feature = "panel-motion-params"))]
fn linear_rgba_to_srgb8(lin: [f32; 4]) -> [u8; 4] {
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
#[cfg(all(feature = "panel-motion-graph", feature = "panel-motion-params"))]
fn build_params_snapshot(motion: &MotionState) -> Option<ph2d_panel_motion_params::ParamsSnapshot> {
    use ph2d_node_registry::ParamWidget;
    use ph2d_nodegraph::cook::OpResolver;
    use ph2d_nodegraph::graph::NodeId;
    use ph2d_panel_motion_params::{
        ColorRow, EnumRow, ParamRow, ParamsSnapshot, ScalarRow, ToggleRow,
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
        // A boolean → a real checkbox; an enum → a named segmented selector.
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

#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
mod tests {
    use super::*;
    use crate::motion_state::MotionState;
    use ph2d_panel_motion_params::ParamRow;

    /// The reported-bug + colour-authoring seam, end to end and headless: a
    /// selected `motion.tint` node resolves to exactly ONE colour swatch row
    /// (not four raw channel sliders), its channels are the RGBA params, and its
    /// display colour is **opaque white** — the identity default that killed the
    /// red dominance. This proves the tint's `ParamWidget::Color` hint flows all
    /// the way to a paintable `ColorRow` (registry → snapshot builder), the exact
    /// seam a compile-green change would leave unverified.
    #[test]
    fn selected_tint_node_yields_one_white_color_row() {
        let mut motion = MotionState::new();
        let tint = motion.doc.graph.add_node("motion.tint");
        ph2d_panel_motion_graph::set_graph_selection(vec![tint.0]);

        let snap = build_params_snapshot(&motion).expect("tint node is resolvable");
        assert_eq!(snap.rows.len(), 1, "tint authors colour as ONE swatch row");
        let ParamRow::Color(c) = &snap.rows[0] else {
            panic!("tint's row must be a colour swatch, not scalar sliders");
        };
        assert_eq!(c.channels, ["r", "g", "b", "a"]);
        // Default params are linear white → sRGB opaque white (the no-op tint).
        assert_eq!(c.srgb, [255, 255, 255, 255]);

        ph2d_panel_motion_graph::set_graph_selection(Vec::new());
    }

    /// The colour read-back is the inverse of the swatch display: writing a
    /// picked sRGB colour lands linear-straight channel values on the node, and
    /// re-reading them rebuilds the same sRGB swatch (round-trip stable). Guards
    /// the sRGB↔linear boundary the bridge owns (the Motion wire is linear).
    #[test]
    fn color_pick_writes_linear_and_round_trips_to_srgb() {
        let mut motion = MotionState::new();
        let tint = motion.doc.graph.add_node("motion.tint");
        let picked = [40, 160, 220, 128]; // a saturated sRGB blue, half alpha

        apply_color_to_node(&mut motion, tint, ["r", "g", "b", "a"], picked);

        // The stored channels are linear-straight (RGB gamma-decoded, alpha /255).
        let lin = channel_values(&motion, tint, ["r", "g", "b", "a"]);
        assert!(lin[0] < lin[2], "blue channel dominates in linear too");
        assert!((lin[3] - 128.0 / 255.0).abs() < 1e-6, "alpha is straight");
        // Re-encoding the stored linear colour reproduces the pick (±1 LSB).
        let srgb = linear_rgba_to_srgb8(lin);
        for (got, want) in srgb.into_iter().zip(picked) {
            assert!(
                got.abs_diff(want) <= 1,
                "round-trip {srgb:?} ≈ {picked:?} within 1 LSB"
            );
        }
    }

    /// The Behaviours seam, cooked through the REAL registry (not a unit-test
    /// stub): `grid → stagger → oscillator` is well-typed (validate passes — the
    /// nodes are registered and their ports match) and cooks end to end, and the
    /// behaviours actually displace the grid. This is the "isolamento órfão"
    /// antidote — a node can be unit-green yet unregistered / mistyped in the
    /// live pipeline; this proves it is wired.
    #[test]
    fn grid_stagger_oscillator_cook_through_the_real_registry() {
        use ph2d_nodegraph::attr::Column;
        use ph2d_nodegraph::cook::Cook;
        use ph2d_nodegraph::graph::{Edge, Graph};

        let motion = MotionState::new(); // registry = register_all_nodes
        let cook_p = |g: &Graph, target| {
            let mut cook = Cook::new();
            let out = cook.cook(g, &motion.registry, target, 0.25).unwrap();
            match out[0].as_stream().get("P").unwrap() {
                Column::Vec2(v) => v.clone(),
                _ => panic!("P"),
            }
        };

        // Bare grid (baseline) vs grid → stagger(Y) → oscillator(Y).
        let mut g = Graph::new();
        let grid = g.add_node("motion.grid");
        let stagger = g.add_node("motion.stagger");
        let osc = g.add_node("motion.oscillator");
        g.connect(Edge {
            from: (grid, 0),
            to: (stagger, 0),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (stagger, 0),
            to: (osc, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(stagger, "channel", 1.0);
        g.set_param(stagger, "min", 0.0);
        g.set_param(stagger, "max", 2.0);
        g.set_param(osc, "channel", 1.0);
        g.set_param(osc, "amplitude", 1.0);
        g.set_param(osc, "phase_stagger", 0.0); // uniform bob → +amplitude at t=¼

        // The whole chain type-checks against the real registry.
        g.validate(&motion.registry)
            .expect("grid → stagger → oscillator is well-typed");

        let base = cook_p(&g, grid);
        let out = cook_p(&g, osc);
        assert_eq!(out.len(), base.len(), "count preserved through behaviours");
        assert!(base.len() >= 4, "grid emits its default cells");
        let n = base.len();
        for (i, (b, o)) in base.iter().zip(&out).enumerate() {
            // X untouched; Y = base + stagger ramp (i/(n-1)·2) + oscillator (+1).
            let ramp = 2.0 * i as f32 / (n as f32 - 1.0);
            assert!((o[0] - b[0]).abs() < 1e-4, "X untouched at {i}");
            assert!(
                (o[1] - (b[1] + ramp + 1.0)).abs() < 1e-4,
                "Y = base + ramp + osc at {i}"
            );
        }
    }

    /// A behaviour's enum / boolean params resolve to NAMED widget rows, not
    /// number sliders: the selected stagger node yields an `Enum` Channel row
    /// (X/Y/Rot/Size), an `Enum` Easing row, and a `Toggle` Reverse row — the
    /// exact fix the Enio asked for (no memorising slider steps).
    #[test]
    fn stagger_params_are_named_enums_and_a_checkbox() {
        use ph2d_panel_motion_params::ParamRow;
        let mut motion = MotionState::new();
        let st = motion.doc.graph.add_node("motion.stagger");
        ph2d_panel_motion_graph::set_graph_selection(vec![st.0]);

        let snap = build_params_snapshot(&motion).expect("stagger resolvable");
        let channel = snap
            .rows
            .iter()
            .find_map(|r| match r {
                ParamRow::Enum(e) if e.name == "channel" => Some(e),
                _ => None,
            })
            .expect("channel is a named Enum row, not a slider");
        assert_eq!(channel.labels, ["X", "Y", "Rot", "Size"]);
        assert!(
            snap.rows
                .iter()
                .any(|r| matches!(r, ParamRow::Enum(e) if e.name == "easing")),
            "easing is a named Enum row"
        );
        assert!(
            snap.rows
                .iter()
                .any(|r| matches!(r, ParamRow::Toggle(t) if t.name == "reverse")),
            "reverse is a checkbox (Toggle) row, not a 0/1 slider"
        );
        ph2d_panel_motion_graph::set_graph_selection(Vec::new());
    }

    /// The animation enabler (ask "when do we see animation?"): playing advances
    /// the playhead — so any `Temporal` behaviour moves — and pausing freezes it.
    /// The default transport is paused, which is why nothing moved before.
    #[test]
    fn transport_play_advances_time_and_pause_freezes_it() {
        let mut motion = MotionState::new();
        let dt = 1.0 / 60.0;
        assert_eq!(motion.playhead(dt), 0.0, "starts paused at t=0");
        motion.transport.play();
        motion.transport.advance(30);
        let t = motion.playhead(dt);
        assert!(
            t > 0.0,
            "playing advances the playhead → behaviours animate"
        );
        motion.transport.toggle(); // → paused
        motion.transport.advance(30);
        assert_eq!(motion.playhead(dt), t, "paused freezes the playhead");
    }

    /// Set as Output: the pump renders whatever node `motion.sink` names, so
    /// pointing it at a node makes that node the visible output — and a `None`
    /// sink renders nothing (the mechanism the O-key intent drives).
    #[test]
    fn sink_drives_what_the_pump_renders() {
        let mut motion = MotionState::new();
        let grid = motion.doc.graph.add_node("motion.grid");
        let (uv, size) = (motion.default_uv_rect, motion.default_size);

        motion.pump.mark_dirty();
        motion.pump.pump(
            &motion.doc.graph,
            &motion.registry,
            Some(grid),
            0,
            0.0,
            uv,
            size,
        );
        assert!(
            motion.pump.instances.len() >= 4,
            "the chosen output node renders its cells"
        );

        motion.pump.mark_dirty();
        motion
            .pump
            .pump(&motion.doc.graph, &motion.registry, None, 0, 0.0, uv, size);
        assert_eq!(motion.pump.instances.len(), 0, "no sink → nothing rendered");
    }
}
