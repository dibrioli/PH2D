//! The **write-back** half of the params bridge (a `#[path]` child of
//! `motion_bridge_params.rs`, split off for the 600-LOC shell file cap). "What the
//! panel SENDS BACK": this frame's param edits applied to the selected node, undo
//! bracketing, and the one reader of a param's current value.
//!
//! The seam is deliberate — the sibling half (`build_params_snapshot`) answers
//! *what the panel SEES*, and it never writes. Splitting the BUILDER instead would
//! have cut a single `match` on `ParamWidget` across two files, which is cutting
//! through the middle of one thing rather than along a seam.

use crate::motion_state::MotionState;
use crate::render_loop::motion_bridge::{backdrops, color, gpu, subgraph};

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
    let sel = super::selected_motion_node().map(NodeId);
    let type_id = sel.and_then(|nid| motion.doc.graph.node(nid).map(|i| i.type_id()));
    let groups = type_id
        .map(|tid| color::color_groups(&motion.registry, tid))
        .unwrap_or_default();
    // The gradient text params (doc 85) — each stop's swatch feeds the SAME OKLCH picker.
    let grad_params = type_id
        .map(|tid| color::gradient_params(&motion.registry, tid))
        .unwrap_or_default();
    // The palette text params — each colour's swatch feeds the SAME OKLCH picker.
    let pal_params = type_id
        .map(|tid| color::palette_params(&motion.registry, tid))
        .unwrap_or_default();

    // A colour-swatch OR gradient-stop pick is an editing session (like a slider drag): its
    // live writes coalesce into ONE undo step, opened here + committed on close. Detected
    // BEFORE the bracket; the read-back writes go INSIDE it.
    let session = color::picker_session(motion, sel, &groups, &grad_params, &pal_params, store);
    let editing = any_param_editing(store) || session;
    let was = PARAM_EDITING.swap(editing, Ordering::Relaxed);
    if editing && !was {
        motion.history.begin(&motion.doc);
    }

    // Colour + gradient-stop read-back: feed the live pick into the params/string it targets
    // (sRGB→linear), re-cooking only on an actual change (the picker stays open across idle
    // frames). One door in `color.rs` (the sRGB↔linear boundary).
    color::apply_picker_readback(motion, sel, &groups, &grad_params, &pal_params, store);

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
        let card = subgraph::selected_card();
        for intent in intents {
            // A backdrop edit never touches a node and never re-cooks (decoration
            // cannot change what the graph cooks).
            if let Some(bid) = backdrop {
                backdrops::apply_param_intent(motion, bid, intent);
                continue;
            }
            if let Some(sid) = card {
                subgraph::apply_param_intent(motion, subgraph::view_id(sid), intent);
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
                    let renumbers_sim = gpu::edit_renumbers_emitter(&inst.type_name, param)
                        && (param_value(motion, nid, param) - value as f32).abs() > f32::EPSILON;
                    motion.doc.graph.set_param(nid, param, value as f32);
                    if let Some(tn) = type_name {
                        super::apply_channel_presets(motion, nid, &tn, value as f32);
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
                // Devolver um param ao default é REMOVER o override, nunca escrever o default
                // por cima (`Graph::clear_param` explica o porquê: um override que por acaso
                // vale o default fossiliza o número no dia em que o nó mudar de default).
                //
                // Os DOIS canais são limpos com o mesmo nome porque um param viaja por um só
                // — e qual deles é conhecimento que apodrece se a UI o carregar. Limpar o que
                // não existe custa uma busca falhada.
                MotionParamIntent::ResetParam { node, param } => {
                    let nid = NodeId(node);
                    if motion.doc.graph.node(nid).is_none() {
                        continue;
                    }
                    let a = motion.doc.graph.clear_param(nid, &param);
                    let b = motion.doc.graph.clear_text_param(nid, &param);
                    if a || b {
                        motion.pump.mark_dirty();
                    }
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
