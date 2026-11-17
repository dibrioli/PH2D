//! **The intent funnel** (Motion Nodes M1.E10) — every edit the panel asks for, applied
//! to the shell-owned document. Sibling of `motion_bridge` (shell LOC cap); `super` is
//! `render_loop::motion_bridge`, so the apply helpers, the sub-modules and `reconcile`
//! all resolve through it.
//!
//! **The id a panel intent carries is a VIEW id** (doc 57): it names a node, or it names
//! a collapsed subgraph CARD. This is where that is decoded (`subgraph::target` /
//! `subgraph::resolve_port`) — so that everything downstream, from the connect authority
//! to the plumbing to the toasts, goes on speaking about real nodes and never learns that
//! a fold exists.

use super::{
    MotionState, apply_delete_selection, apply_disconnect, backdrops, connect, edit, reconcile,
    rewire, subgraph,
};
use crate::motion_state::FormaEscolhida;
use ph2d_editor_core::ToastQueue;
use ph2d_editor_core::screens::layout::CenterSplit;

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
pub(crate) fn apply_graph_intents(
    motion: &mut MotionState,
    playhead: &mut ph2d_core::Playhead,
    toasts: &mut ToastQueue,
    split: &mut CenterSplit,
) {
    let intents = ph2d_panel_motion_graph::drain_intents();
    aplica_e_grava(motion, playhead, toasts, split, intents);
    publica_no_painel(motion);
}

/// **Um lote do painel, aplicado e gravado** — o dreno do editor e a gravação por script de uma cena
/// (`macro_track::smoke`), que não passa pela fila global nem publica no painel.
pub(crate) fn aplica_e_grava(
    motion: &mut MotionState,
    playhead: &mut ph2d_core::Playhead,
    toasts: &mut ToastQueue,
    split: &mut CenterSplit,
    intents: Vec<ph2d_panel_motion_graph::GraphIntent>,
) {
    motion.macro_lote_de_grafo(&intents);
    apply_graph_batch(motion, playhead, toasts, split, intents);
    motion.macro_fecha();
}

/// **Publica no painel o que as portas lhe pediram** ([`crate::motion_state::PedidosAoPainel`]) — só
/// a porta do editor: o palco de uma macro corre o [`apply_graph_batch`] e nunca isto.
pub(crate) fn publica_no_painel(motion: &mut MotionState) {
    if let Some(nos) = motion.ao_painel.selecao.take() {
        ph2d_panel_motion_graph::request_graph_selection(nos);
    }
    for intent in motion.ao_painel.params.drain(..) {
        crate::push_param_intent(intent);
    }
}

/// **Um lote de intenções do grafo**, aplicado pela ordem — a porta do dreno acima e a do tocador
/// de uma macro ([`crate::macro_track::tocar`]), que lhe entrega os lotes gravados.
pub(crate) fn apply_graph_batch(
    motion: &mut MotionState,
    playhead: &mut ph2d_core::Playhead,
    toasts: &mut ToastQueue,
    split: &mut CenterSplit,
    intents: Vec<ph2d_panel_motion_graph::GraphIntent>,
) {
    use ph2d_nodegraph::graph::{NodeId, Pos};
    use ph2d_panel_motion_graph::GraphIntent;
    let heal_ok = intents.iter().any(super::heal::is_constructive)
        && !intents.iter().any(super::heal::is_destructive);
    for intent in intents {
        match intent {
            GraphIntent::BeginDrag => motion.history.begin(&motion.doc),
            GraphIntent::MoveNodes { nodes, dx, dy } => {
                for id in nodes {
                    let alvo = subgraph::target(id);
                    if dx != 0.0 || dy != 0.0 {
                        marca_arrumado_a_mao(motion, alvo);
                    }
                    match alvo {
                        subgraph::Target::Card(sid) => subgraph::translate(motion, sid, dx, dy),
                        subgraph::Target::Node(nid) => {
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
                }
            }
            GraphIntent::GroupSelection { nodes } => subgraph::group(motion, nodes),
            GraphIntent::Ungroup { id } => subgraph::ungroup(motion, id),
            GraphIntent::EnterSubgraph { id } => subgraph::set_level(motion, Some(id)),
            GraphIntent::GoToLevel { level } => subgraph::set_level(motion, level),
            GraphIntent::EndDrag => motion.history.commit_if_changed(&motion.doc),
            GraphIntent::Connect {
                from_node,
                from_port,
                to_node,
                to_port,
            } => {
                if let Some((f, fp)) = subgraph::resolve_port(motion, from_node, from_port, false)
                    && let Some((t, tp)) = subgraph::resolve_port(motion, to_node, to_port, true)
                {
                    if subgraph::saida_morta(motion, (f, fp)) {
                        subgraph::avisa_saida_morta(toasts);
                    } else {
                        connect::apply_connect(motion, toasts, f.0, fp, t.0, tp);
                    }
                }
            }
            GraphIntent::DriveParam {
                from_node,
                from_port,
                to_node,
                param,
            } => {
                if let Some((f, fp)) = subgraph::resolve_port(motion, from_node, from_port, false)
                    && let Some(t) = match subgraph::target(to_node) {
                        subgraph::Target::Node(n) => Some(n),
                        subgraph::Target::Card(_) => None,
                    }
                {
                    subgraph::drive(motion, toasts, (f, fp), t, param);
                }
            }
            GraphIntent::DropRefused {
                from_node,
                from_port,
                to_node,
            } => {
                if let Some(from) = subgraph::resolve_port(motion, from_node, from_port, false) {
                    toasts.push(ph2d_editor_core::Toast::warning(super::recusa::no_corpo(
                        motion, from, to_node,
                    )));
                }
            }
            GraphIntent::ToggleParamSection { node, section } => {
                if let subgraph::Target::Node(n) = subgraph::target(node) {
                    let aberta = motion
                        .card_sections
                        .get(&(n.0, section))
                        .copied()
                        .unwrap_or_else(|| {
                            motion
                                .doc
                                .graph
                                .node(n)
                                .map(|i| i.type_id())
                                .is_none_or(|t| {
                                    !motion.registry.param_groups_folded(t).contains(&section)
                                })
                        });
                    motion.card_sections.insert((n.0, section), !aberta);
                }
            }
            GraphIntent::SetParam { node, param, value } => {
                if let subgraph::Target::Node(n) = subgraph::target(node) {
                    motion.encaminha_param(crate::MotionParamIntent::SetParam {
                        node: n.0,
                        param,
                        value: f64::from(value),
                    });
                }
            }
            GraphIntent::PickSelection { node, param } => {
                let subgraph::Target::Node(n) = subgraph::target(node) else {
                    continue;
                };
                match motion.selected_shape.clone() {
                    FormaEscolhida::Nome(nome) => {
                        motion.doc.graph.set_text_param(n, param, nome.clone());
                        motion.pump.mark_dirty();
                        toasts.push(ph2d_editor_core::Toast::info(ph2d_i18n::tr_with(
                            "app.motion.motion_bridge_intents.path_set_to",
                            &[("nome", &nome)],
                        )));
                    }
                    FormaEscolhida::Nada => {
                        toasts.push(ph2d_editor_core::Toast::info(
                            ph2d_i18n::tr("app.motion.motion_bridge_intents.select_a_drawing_first_on_the_canvas_or_in_the_h")
                                .to_string(),
                        ));
                    }
                    FormaEscolhida::NaoEDesenho => {
                        toasts.push(ph2d_editor_core::Toast::info(
                            ph2d_i18n::tr("app.motion.motion_bridge_intents.the_selected_object_is_not_a_drawing_pick_a_path").to_string(),
                        ));
                    }
                    FormaEscolhida::SemNome => {
                        toasts.push(ph2d_editor_core::Toast::info(
                            ph2d_i18n::tr("app.motion.motion_bridge_intents.give_this_drawing_a_name_in_the_hierarchy_first").to_string(),
                        ));
                    }
                    FormaEscolhida::SemArco => {
                        toasts.push(ph2d_editor_core::Toast::info(
                            ph2d_i18n::tr("app.motion.motion_bridge_intents.this_drawing_has_fewer_than_two_points_there_is")
                                .to_string(),
                        ));
                    }
                }
            }
            GraphIntent::PickFile { node, param } => {
                if let subgraph::Target::Node(n) = subgraph::target(node) {
                    motion.encaminha_param(crate::MotionParamIntent::PickFile { node: n.0, param });
                }
            }
            GraphIntent::StepChoice { node, param, delta } => {
                if let subgraph::Target::Node(n) = subgraph::target(node) {
                    super::params::step_choice(motion, n, param, delta);
                }
            }
            GraphIntent::PickChoice { node, param, index } => {
                if let subgraph::Target::Node(n) = subgraph::target(node) {
                    super::params::pick_choice(motion, n, param, index);
                }
            }
            GraphIntent::SetTextParam { node, param, value } => {
                if let subgraph::Target::Node(n) = subgraph::target(node) {
                    motion.encaminha_param(crate::MotionParamIntent::SetTextParam {
                        node: n.0,
                        param,
                        value,
                    });
                }
            }
            GraphIntent::Disconnect { to_node, to_port } => {
                if let Some((t, tp)) = subgraph::resolve_port(motion, to_node, to_port, true) {
                    apply_disconnect(motion, toasts, t.0, tp);
                }
            }
            GraphIntent::OpenLibrary {
                x,
                y,
                connect_from,
                connect_to,
                splice,
                compatible,
            } => {
                motion.open_library = Some(crate::motion_state::LibraryOpen {
                    spawn: (x, y),
                    connect_from,
                    connect_to,
                    splice,
                    compatible,
                });
            }
            GraphIntent::AddNode { type_name, x, y } => {
                let pre = motion.doc.clone();
                let id = motion.doc.graph.add_node(type_name);
                motion.doc.graph.set_pos(id, Pos { x, y });
                reconcile(motion, &pre.graph);
                motion.history.push_undo(pre);
                motion.pump.mark_dirty();
            }
            GraphIntent::DeleteSelection { nodes } => {
                apply_delete_selection(motion, nodes, toasts);
            }
            GraphIntent::DuplicateSelection { nodes } => {
                let cards: Vec<u32> = nodes
                    .iter()
                    .filter_map(|v| subgraph::subgraph_of(*v))
                    .collect();
                let mut sources: Vec<u32> = nodes
                    .iter()
                    .filter(|v| subgraph::subgraph_of(**v).is_none())
                    .copied()
                    .collect();
                for sid in &cards {
                    sources.extend(
                        ph2d_motion_doc::subgraph::member_nodes_deep(
                            &motion.doc.subgraphs,
                            &motion.doc.members,
                            *sid,
                        )
                        .iter()
                        .map(|n| n.0),
                    );
                }
                let copy_of = edit::duplicate(motion, sources);
                if !copy_of.is_empty() {
                    let selection = subgraph::duplicate_nesting(motion, &cards, &copy_of);
                    motion.pede_selecao(selection);
                }
            }
            GraphIntent::CopySelection { nodes } => {
                let cards: Vec<u32> = nodes
                    .iter()
                    .filter_map(|v| subgraph::subgraph_of(*v))
                    .collect();
                let mut sources: Vec<u32> = nodes
                    .iter()
                    .filter(|v| subgraph::subgraph_of(**v).is_none())
                    .copied()
                    .collect();
                for sid in &cards {
                    sources.extend(
                        ph2d_motion_doc::subgraph::member_nodes_deep(
                            &motion.doc.subgraphs,
                            &motion.doc.members,
                            *sid,
                        )
                        .iter()
                        .map(|n| n.0),
                    );
                }
                edit::copy_selection(motion, sources, cards);
            }
            GraphIntent::Paste => edit::paste(motion),
            GraphIntent::CutWires { targets } => {
                let targets: Vec<(u32, u16)> = targets
                    .into_iter()
                    .filter_map(|(n, p)| {
                        subgraph::resolve_port(motion, n, p, true).map(|(n, p)| (n.0, p))
                    })
                    .collect();
                edit::cut_wires(motion, toasts, targets);
            }
            GraphIntent::SpliceReroute {
                to_node,
                to_port,
                x,
                y,
            } => {
                if let Some((t, tp)) = subgraph::resolve_port(motion, to_node, to_port, true) {
                    rewire::splice_reroute(motion, toasts, t.0, tp, x, y);
                }
            }
            GraphIntent::SpliceNode {
                to_node,
                to_port,
                to_type,
                x,
                y,
            } => {
                if let Some((t, tp)) = subgraph::resolve_port(motion, to_node, to_port, true) {
                    rewire::splice_node(motion, toasts, t.0, tp, to_type, x, y);
                }
            }
            GraphIntent::MoveWireEnd {
                from_node,
                from_port,
                old_to_node,
                old_to_port,
                new_to,
            } => {
                let src = subgraph::resolve_port(motion, from_node, from_port, false);
                let old = subgraph::resolve_port(motion, old_to_node, old_to_port, true);
                let land = match new_to {
                    Some((n, p)) => {
                        subgraph::resolve_port(motion, n, p, true).map(|(n, p)| Some((n.0, p)))
                    }
                    None => Some(None),
                };
                if let (Some((f, fp)), Some((o, op)), Some(land)) = (src, old, land) {
                    rewire::move_wire_end(motion, toasts, f.0, fp, o.0, op, land);
                }
            }
            GraphIntent::SmartConnect {
                from_node,
                from_port,
                to_type,
                x,
                y,
            } => {
                if let Some((f, fp)) = subgraph::resolve_port(motion, from_node, from_port, false) {
                    edit::smart_connect(motion, toasts, f.0, fp, to_type, x, y);
                }
            }
            GraphIntent::SwapInChain {
                a,
                b,
                back_dx,
                back_dy,
            } => {
                if !ph2d_panel_motion_graph::is_subgraph_view(a)
                    && !ph2d_panel_motion_graph::is_subgraph_view(b)
                {
                    rewire::swap_in_chain(motion, toasts, a, b, back_dx, back_dy);
                }
            }
            GraphIntent::SpliceExistingIntoWire {
                node,
                to_node,
                to_port,
            } => {
                if !ph2d_panel_motion_graph::is_subgraph_view(node)
                    && let Some((t, tp)) = subgraph::resolve_port(motion, to_node, to_port, true)
                {
                    rewire::splice_existing_into_wire(motion, toasts, node, t.0, tp);
                }
            }
            GraphIntent::SmartConnectBack {
                to_node,
                to_port,
                to_type,
                x,
                y,
            } => {
                if let Some((t, tp)) = subgraph::resolve_port(motion, to_node, to_port, true) {
                    edit::smart_connect_back(motion, toasts, t.0, tp, to_type, x, y);
                }
            }
            GraphIntent::SetProbe { node } => {
                motion.probe = node.and_then(|v| subgraph::probe_target(motion, v));
                motion.probe_ring.clear();
            }
            GraphIntent::FixInert { node } => {
                super::heal::heal_one(motion, toasts, ph2d_nodegraph::graph::NodeId(node));
            }
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
            GraphIntent::TogglePlay => {
                playhead.toggle_play();
            }
            GraphIntent::SetNodeHelp(on) => {
                motion.node_help_enabled = on;
            }
            GraphIntent::AddBackdrop { x, y, w, h } => backdrops::add(motion, x, y, w, h),
            GraphIntent::MoveBackdrop { id, dx, dy } => backdrops::translate(motion, id, dx, dy),
            GraphIntent::ResizeBackdrop { id, left, dx, dy } => {
                backdrops::resize(motion, id, left, dx, dy)
            }
            GraphIntent::DeleteBackdrop { id } => backdrops::delete(motion, id),
            GraphIntent::Rename { target, name } => rename(motion, target, name),
            GraphIntent::SetBackdropColor { id, color } => {
                let pre = motion.doc.clone();
                backdrops::set_color(motion, id, color);
                if motion.doc != pre {
                    motion.history.push_undo(pre);
                }
            }
            GraphIntent::ArrangeLayout => {
                let pre = motion.doc.clone();
                crate::motion_arrumar::arrumar(motion);
                if motion.doc != pre {
                    motion.history.push_undo(pre);
                }
            }
            GraphIntent::SetBypass { nodes, on } => {
                let pre = motion.doc.clone();
                for id in nodes {
                    match subgraph::subgraph_of(id) {
                        Some(sid)
                            if ph2d_motion_doc::subgraph::find(&motion.doc.subgraphs, sid)
                                .is_some() =>
                        {
                            motion.doc.set_subgraph_bypassed(sid, on)
                        }
                        Some(_) => {}
                        None => {
                            let nid = NodeId(id);
                            if motion.doc.graph.node(nid).is_some() {
                                motion.doc.graph.set_bypassed(nid, on);
                            }
                        }
                    }
                }
                if motion.doc != pre {
                    motion.history.push_undo(pre);
                    motion.pump.mark_dirty();
                }
            }
        }
    }
    if heal_ok {
        super::heal::heal_setup(motion, toasts);
    }
}

/// Marca o dono de cada nó que `alvo` arrasta (um cartão de grupo arrasta todos os de dentro).
fn marca_arrumado_a_mao(motion: &mut MotionState, alvo: subgraph::Target) {
    let nos = match alvo {
        subgraph::Target::Card(sid) => ph2d_motion_doc::subgraph::member_nodes_deep(
            &motion.doc.subgraphs,
            &motion.doc.members,
            sid,
        ),
        subgraph::Target::Node(nid) => vec![nid],
    };
    for n in nos {
        if let Some(o) = motion.doc.owner_of(n) {
            motion.doc.arrumados_a_mao.insert(o);
        }
    }
}

/// **Name a thing** (doc 61) — the one place the three targets' three storage places meet.
///
/// A name is **not semantic**: no cook reads it, so this deliberately does NOT `mark_dirty`. It is
/// still document state, so it is undoable — and the undo step is pushed only if the name actually
/// CHANGED, because pressing Enter on a box you did not edit is not an edit, and an undo queue
/// that fills with those is one the artist cannot use.
///
/// The trim/empty rule is the graph's (`set_label`: an empty name is not a name, it is *"call it
/// what it is"*), and the two `title` fields follow it — an emptied group card goes back to
/// saying "Group", not to saying nothing.
pub(super) fn rename(
    motion: &mut MotionState,
    target: ph2d_panel_motion_graph::RenameTarget,
    name: String,
) {
    use ph2d_nodegraph::graph::NodeId;
    use ph2d_panel_motion_graph::RenameTarget;
    let pre = motion.doc.clone();
    let name = name.trim().to_string();
    match target {
        RenameTarget::Node(id) => motion.doc.graph.set_label(NodeId(id), name),
        RenameTarget::Subgraph(id) => {
            if let Some(s) = motion.doc.subgraphs.iter_mut().find(|s| s.id == id) {
                s.title = name;
            }
        }
        RenameTarget::Backdrop(id) => backdrops::set_title(motion, id, name),
    }
    if motion.doc != pre {
        motion.history.push_undo(pre);
    }
}
