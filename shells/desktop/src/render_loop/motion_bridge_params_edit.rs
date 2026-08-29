//! The **write-back** half of the params bridge (a `#[path]` child of
//! `motion_bridge_params.rs`, split off for the 600-LOC shell file cap). "What the
//! panel SENDS BACK": this frame's param edits applied to the selected node, undo
//! bracketing, and the one reader of a param's current value.
//!
//! The seam is deliberate — the sibling half (`build_params_snapshot`) answers
//! *what the panel SEES*, and it never writes. Splitting the BUILDER instead would
//! have cut a single `match` on `ParamWidget` across two files, which is cutting
//! through the middle of one thing rather than along a seam.

use super::params_file;
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
///
/// ⚠️ **`toasts` porque uma edição pode SOLTAR um fio** (`drop_hidden_drivers`): trocar a
/// espécie de uma forma esconde os knobs que ela não lê, e um fio que os conduzia tem de cair
/// — em voz alta, nunca em silêncio.
pub(super) fn apply_param_edits(
    motion: &mut MotionState,
    store: &ph2d_editor::interaction::WidgetStore,
    toasts: &mut ph2d_editor::ToastQueue,
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
            // ⚠️ **O nó que esta intenção TOCOU**, para a reparação a seguir. Os braços que
            // saem por `continue` não tocaram nada (nó inexistente, backdrop, card), e é isso
            // que os mantém fora da reparação sem uma condição extra.
            let touched = match intent {
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
                    nid
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
                    nid
                }
                // Devolver um param ao default é REMOVER o override, nunca escrever o default
                // por cima (`Graph::clear_param` explica o porquê: um override que por acaso
                // vale o default fossiliza o número no dia em que o nó mudar de default).
                //
                // Os DOIS canais são limpos com o mesmo nome porque um param viaja por um só
                // — e qual deles é conhecimento que apodrece se a UI o carregar. Limpar o que
                // não existe custa uma busca falhada.
                // **Escolher um ficheiro** — o painel PEDE, a shell abre. O diálogo congela o
                // loop, e por isso passa pela porta que o declara (`modal::pick_file`, via
                // `params_file::pick`); o filtro sai da espécie que o `ParamUiHint` declara,
                // nunca de uma lista escrita aqui.
                //
                // ⚠️ Cancelar não escreve nada e não toca o nó: sai por `continue`, então nem
                // a reparação de fios órfãos corre. *Um gesto abandonado não é uma edição.*
                MotionParamIntent::PickFile { node, param } => {
                    let nid = NodeId(node);
                    if motion.doc.graph.node(nid).is_none() {
                        continue;
                    }
                    let Some(path) = params_file::pick(motion, nid, param) else {
                        continue;
                    };
                    motion.doc.graph.set_text_param(nid, param, path);
                    motion.pump.mark_dirty();
                    nid
                }
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
                    nid
                }
            };
            // **A REPARAÇÃO**, uma vez por intenção que tocou um nó — nunca três cópias, uma
            // por braço: um braço novo sem ela seria uma quarta porta pela qual o fio órfão
            // volta, e ninguém o veria porque o defeito é silencioso por natureza.
            drop_hidden_drivers(motion, touched, toasts);
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

/// **UM PARAM QUE DESAPARECEU NÃO FICA CONDUZIDO** — o segundo pedido do report do Enio de
/// 2026-08-27: *"se o usuário trocar de shape e a nova shape não tiver um parâmetro linkado,
/// o link deve ser quebrado"*.
///
/// Trocar a espécie de um `source.shape` esconde os knobs que aquela espécie não lê. Um fio
/// que os conduzia continuava ligado: o socket ficava no card, o número continuava a ser
/// cozido, e **nada o lia** — um fio visível a alimentar um param inexistente, que é pior que
/// o knob morto que o `ParamGate` existe para não ter (doc 90). *Um controle escondido o
/// artista deixa de ver; um fio pendurado ele continua a ver, e conclui que age.*
///
/// ⚠️ **A lei é um INVARIANTE, não um diff:** *nenhum param escondido fica conduzido*. Não se
/// compara "antes" com "depois" — comparar exigiria capturar a visibilidade antes de cada
/// escrita, e um caminho de escrita novo (um preset, um paste, uma migração) escaparia à
/// captura. Perguntar o estado FINAL não tem esse buraco, e é idempotente: correr duas vezes
/// não solta nada a mais.
///
/// ⚠️⚠️ **E a régua NÃO é a do menu de largar, de propósito — a assimetria é a lei.** O menu
/// filtra pelas **três** famílias de gate (um controle que não se vê não se pode usar, logo não
/// se oferece); esta solta pela **discreta apenas** (`mode_hidden`). Soltar um fio DESTRÓI
/// trabalho do artista, e isso só se justifica quando o controle deixou de EXISTIR — não
/// quando ele está momentaneamente inerte. ⚠️ Medido no próprio `source.shape`: **seis** params
/// (as duas cores, o tracejado e as duas pontas do Trim) pendem do *Stroke Width* passar por
/// zero, que é um arrasto — a 1.ª redacção desta função usava as três famílias e apagava a
/// ligação do artista no meio de um gesto reversível.
///
/// ⚠️ **E é a mesma família da lei do CANAL que já vivia ao lado** (`apply_channel_presets`:
/// *"todo param cuja FAIXA segue o canal tem de ter o VALOR trazido para a faixa nova"*) — ali
/// o que fica inconsistente é um número, aqui é uma aresta.
///
/// ⚠️ **Nunca em silêncio.** O fio é trabalho do artista; soltá-lo sem dizer é o app a desfazer
/// uma edição às escondidas. O toast NOMEIA os params soltos, pelo rótulo que o painel mostra.
fn drop_hidden_drivers(
    motion: &mut MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    toasts: &mut ph2d_editor::ToastQueue,
) {
    // Os nomes primeiro, com o `motion` emprestado só para leitura; a escrita vem depois.
    let hidden: Vec<String> = {
        let gone = super::params_visible::mode_hidden(motion, nid);
        motion
            .doc
            .graph
            .param_sources(nid)
            .into_iter()
            .flatten()
            .map(|(name, _)| name.clone())
            .filter(|name| gone(name))
            .collect()
    };
    if hidden.is_empty() {
        return;
    }
    // O rótulo que o painel mostra ("Tooth Depth"), nunca a chave canónica: o artista procura
    // no painel o que o toast lhe disser.
    let hints = motion
        .doc
        .graph
        .node(nid)
        .and_then(|i| motion.registry.param_ui(i.type_id()));
    let labels: Vec<&str> = hidden
        .iter()
        .map(|name| {
            hints
                .and_then(|hs| hs.iter().find(|h| h.param == *name))
                .map_or(name.as_str(), |h| h.label)
        })
        .collect();
    toasts.push(ph2d_editor::Toast::info(if labels.len() == 1 {
        format!("Unlinked {} - this shape has no such control", labels[0])
    } else {
        format!(
            "Unlinked {} - this shape has no such controls",
            labels.join(", ")
        )
    }));
    for name in &hidden {
        motion.doc.graph.undrive_param(nid, name);
    }
    motion.pump.mark_dirty();
}
