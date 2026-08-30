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
//! 3. **Per-frame cook** — cook the graph's sink, at the tick the editor's ONE
//!    clock (`ph2d_core::Playhead`, W4.T7) is standing on, into the reused
//!    `MotionState.instances` buffer. Motion keeps NO transport of its own.
//!    The render loop injects that slice via `SpriteRenderer::render_with_extra`
//!    (`present.rs`) — the cooked stream draws without being spawned into the
//!    ECS `PresentWorld` (stream ≠ ECS, ADR-0035).
//!
//! **Zero concrete-tool downcast:** unlike `vector_bridge`, the document lives in
//! `MotionState` (shell), not the tool, so the central render loop stays
//! downcast-free without this bridge reaching into a concrete tool at all.
//!
//! The params-panel side (scalar/enum/checkbox rows + OKLCH colour authoring)
//! lives in the sibling [`params`] module, and the headless tests in
//! `motion_bridge_tests.rs` (+ `motion_bridge_param_tests.rs` and
//! `motion_bridge_plumbing_tests.rs`), all split out for the HR-18 LOC cap.

use crate::motion_state::MotionState;
use ph2d_editor::screens::layout::CenterSplit;
use ph2d_editor::{HeroScreen, ToastQueue, ToolId, ToolRegistry};

// GPU-resident cook routing (F1.1 fully-GPU + F1.2 hybrid). Unconditional — the
// GPU path does not depend on the graph-panel feature.
// ⚠️ `pub(crate)` para que uma CENA possa afirmar a **rota** que vai tomar, e não a declaração
// de que ela tem dois sinks. A `=107` precisa disso: `sinks.len() == 2` é o *proxy*, e o gate que
// o media ficava verde sobre uma cena que a rota tivesse mandado para o device na mesma
// (auditoria de 2026-08-27). `gpu_route` é função **pura** — é o oráculo certo e já existia.
#[path = "motion_bridge_gpu.rs"]
pub(crate) mod gpu;

#[cfg(all(feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_params.rs"]
pub(crate) mod params;

#[cfg(all(feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_color.rs"]
mod color;

#[cfg(feature = "panel-motion-graph")]
#[path = "motion_bridge_plumbing.rs"]
mod plumbing;

#[cfg(feature = "panel-motion-graph")]
#[path = "motion_bridge_backdrops.rs"]
mod backdrops;

#[path = "motion_bridge_shapes.rs"]
mod shapes;

// doc 86 §2: the OBJECT membrane (sibling of `shapes`) — every named sprite
// becomes an external the graph can source (`source.object`). Unconditional
// like `shapes`: a graph cooks in the background whatever tool is active.
#[path = "motion_bridge_objects.rs"]
mod objects;
// The membrane's object-bake wrappers, re-exported so `motion_bridge::publish_objects`
// / `bake_objects` / `bake_flip_objects` resolve unchanged for the render loop.
pub(super) use objects::{bake_flip_objects, bake_objects, publish_objects};
// A porta que constrói a aparência de um objecto — ver `objects::streams`. Só o gate das
// FOLHAS a chama, e ela existe para ele não copiar a forma do stream.
#[cfg(test)]
pub(super) use objects::streams::appearance_tile;
#[cfg(test)]
pub(super) use objects::streams::appearance_vector;
// ⚠️ **E o conversor de tile**, que o `motion_glow_layer` usa para a metade vetorial
// viva chegar ao bright-pass (bug do Enio, 2026-08-20). A MESMA função que a
// partição de LOD usa — duas vistas da mesma conversão nunca podem divergir.
pub(crate) use objects::vector_instance_as_tile;
// The cursor half of the same table (): the editor value a
// document cannot hold. Re-exported beside the object publishes it must follow.

// The named-group membership predicate, re-exported pub(crate) so the object/flip bakes'
// `select_present` (top-level shell modules) reach it. It is the SAME tree relation
// `objects::group_externals` descends, and the object-bake gate pins that they agree.
pub(crate) use objects::{Appearance, entity_is_in_a_named_group, sprite_appearance};

/// The LOD count knee (the const lives in the private `objects` module) — for the
/// `=6` smoke diagnostic, so the printed threshold and the one the partition uses are
/// the same number.
pub(crate) fn objects_lod_count() -> usize {
    objects::LOD_COUNT
}

#[cfg(feature = "panel-motion-graph")]
#[path = "motion_bridge_edit.rs"]
mod edit;

#[cfg(feature = "panel-motion-graph")]
#[path = "motion_bridge_readout.rs"]
mod readout;

#[cfg(feature = "panel-motion-graph")]
#[path = "motion_bridge_rewire.rs"]
mod rewire;

#[cfg(feature = "panel-motion-graph")]
#[path = "motion_bridge_connect.rs"]
mod connect;

#[cfg(feature = "panel-motion-graph")]
#[path = "motion_bridge_library.rs"]
mod library;

#[cfg(feature = "panel-motion-graph")]
#[path = "motion_bridge_adapt.rs"]
mod adapt;

// **Que colunas a stream carrega aqui?** — a porta única, com o memo do cook E a
// tomada de GPU atrás dela. Sob `panel-motion-graph` (e não sob a cfg mais estreita
// do painel de params) porque os DOIS consumidores dela vivem aqui: o column picker
// e o diagnóstico do nome que não resolve.
#[cfg(feature = "panel-motion-graph")]
#[path = "motion_bridge_columns.rs"]
mod columns;

// ADR-0155 W2 — the setup auto-heal. Sibling of `adapt` (heal-on-gesture, not
// heal-on-refusal): `apply_graph_intents` runs it after a constructive batch.
#[cfg(feature = "panel-motion-graph")]
#[path = "motion_bridge_heal.rs"]
mod heal;

#[cfg(feature = "panel-motion-graph")]
#[path = "motion_bridge_fold.rs"]
mod fold;

#[cfg(feature = "panel-motion-graph")]
#[path = "motion_bridge_subgraph.rs"]
mod subgraph;

#[cfg(feature = "panel-motion-graph")]
#[path = "motion_bridge_group_bypass.rs"]
mod group_bypass;
// Without the graph panel there is no way to mute a group, so the cook always reads the
// document graph — a shim keeps the per-frame cook path feature-agnostic (it uses `fold`/
// `subgraph`, which are panel-gated).
#[cfg(not(feature = "panel-motion-graph"))]
mod group_bypass {
    pub(super) fn cook_graph(_m: &super::MotionState) -> Option<ph2d_nodegraph::graph::Graph> {
        None
    }
}

#[path = "motion_bridge_clock.rs"]
mod clock;
// Re-exported at `motion_bridge` level: the cook loop here and the GPU/test siblings
// all call `super::ticks_owed` / `motion_bridge::ticks_owed`.
pub(crate) use clock::ticks_owed;
// ⚠️ A derivação tique↔segundos e a trava de entrada da ferramenta mudaram-se para o
// MESMO módulo: é o mesmo assunto que o `ticks_owed`, e este pai estava a UMA linha
// do teto de 600 (dívida latente que esta wave herdou). Os chamadores não mudam —
// `motion_bridge::motion_tick` e `super::motion_tick` continuam a ser o caminho.
use clock::{LAST_ACTIVE, motion_time};
pub(crate) use clock::{forget_tool_transition, motion_tick};

/// O lado de Motion da fronteira de sinais — ver o módulo.
#[path = "motion_bridge_signals.rs"]
pub(crate) mod signals;
use signals::signal_nodes;

#[cfg(feature = "panel-motion-graph")]
#[path = "motion_bridge_intents.rs"]
mod intents;

/// Os 22 módulos de TESTE desta membrana — declarados num irmão pelo teto de LOC (HR-18).
#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_test_mods.rs"]
mod test_mods;
#[cfg(feature = "panel-motion-graph")]
use intents::apply_graph_intents;
#[cfg(test)]
#[path = "motion_bridge_visibility_tests.rs"]
mod visibility_tests;

/// Per-frame Motion-tool plumbing. Safe to call every frame; a no-op when the
/// Motion tool is inactive (beyond flipping panel visibility / the split off).
///
/// - `playhead`: the editor's ONE clock (`ph2d_core::Playhead`, W4.T7). Motion
///   READS it for the tick to cook and WRITES it for transport intents (Space,
///   auto-play on entry) — it no longer keeps a transport of its own.
/// - `fixed_dt`: the fixed timestep in seconds (`tick × fixed_dt` = seconds).
/// - `cursor`: the latest pointer position (screen px) — drives the cursor-gated
///   graph keyboard focus (Blender-style F acts on the hovered area).
/// - `toasts`: the shell toast queue — the connect authority raises a refusal
///   toast here when a dragged edge is rejected (cycle / occupied / typing /
///   membrane).
#[cfg_attr(not(feature = "panel-motion-graph"), allow(unused_variables))]
#[allow(clippy::too_many_arguments)] // the per-frame bridge seam: clock + input + GPU context
pub(super) fn dispatch(
    hero: &mut HeroScreen,
    tools: &ToolRegistry,
    motion: &mut MotionState,
    playhead: &mut ph2d_core::Playhead,
    fixed_dt: f64,
    cursor: (f32, f32),
    toasts: &mut ToastQueue,
    gpu: &ph2d_gpu::GpuContext,
) {
    let motion_active = tools
        .active()
        .is_some_and(|t| t.id() == ToolId::new("motion"));

    // ── 1. Panel visibility (mirror of the Vector dock takeover) ──────────
    hero.panel_visibility.insert(
        ph2d_editor::screens::hero::PANEL_MOTION_GRAPH,
        motion_active,
    );
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
        use std::sync::atomic::Ordering;
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
                // This plays the EDITOR's clock (W4.T7) — the timeline runs with
                // the graph, which is the point of there being only one.
                playhead.play();
                // …and the timeline COMES WITH IT (W4.T4). It was already running (the bridge
                // and the snapshot never cared whether it was visible) and already driving this
                // very clock — it just was not on screen unless the artist happened to have
                // pressed `L`. A tool that auto-plays and hides the transport is a tool that
                // asks you to scrub blind. The layout gives it a band of its own under the
                // graph (`HeroLayout::dock_timeline_into_motion`).
                //
                // Leaving the tool does NOT hide it again: it is the GLOBAL timeline, and taking
                // away a panel the artist can see is not ours to do. `L` still toggles it.
                hero.panel_visibility
                    .insert(ph2d_editor::screens::hero::PANEL_TIMELINE, true);
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
            // Route LAST frame's palette pick into a graph edit BEFORE draining intents (so it lands this
            // frame), then — after the drain — open the palette any gesture asked for. Both live in
            // `library` (add / smart-connect / splice routing + the compatible-filtered open).
            library::route_palette_pick(hero, motion);
            apply_graph_intents(motion, playhead, toasts, &mut hero.view.center_split);
            library::open_pending_palette(hero, motion);
            // The level the artist is standing in can stop existing under their feet
            // (an undo that unmakes the group). Checked AFTER the intents — an undo
            // arrives as one — and before anything is published, so the fold never
            // runs against a room that is not there.
            subgraph::clamp_level(motion);
            // Publish the addable-node catalog for the add-menu. Rebuilt each
            // active frame (cheap: ~dozens of `Copy` entries) alongside the
            // snapshot; memoizing it is a follow-up like the snapshot's own
            // dirty gate.
            ph2d_panel_motion_graph::set_current_node_catalog(library::build_catalog(
                &motion.registry,
            ));
        } else {
            ph2d_panel_motion_graph::set_current_node_catalog(Vec::new());
        }
        // The probe's reading for this frame (a memo lookup on the pump's own cook —
        // see `edit::sample_probe`). Taken before the snapshot borrow.
        //
        // At the COOK's time, not the playhead's raw seconds: the memo is keyed by the tick
        // the pump cooked, and mid-tick (a scrub lands anywhere) those two differ. Asking for
        // a time nobody cooked is how a readout starts lying.
        let cook_time = motion_time(playhead, fixed_dt);
        // The GPU's samples for this frame — ONE tap, feeding both the cards and
        // the probe (see `readout::take_tap`). Taken before the snapshot borrow,
        // and before the probe, which now reads it.
        let tapped = motion_active
            .then(|| readout::take_tap(motion, gpu))
            .flatten();
        let probe = motion_active
            .then(|| edit::sample_probe(motion, cook_time, tapped.as_ref()))
            .flatten();
        // Publish the node-help flag (ADR-0155) so the toolbar chip draws its live state
        // and the toggle it emits flips against the right value. A shell→panel scalar, off
        // the snapshot (editor UX state, like the selection), set every frame.
        ph2d_panel_motion_graph::set_node_help(motion.node_help_enabled);
        ph2d_panel_motion_graph::set_current_motion_graph(motion_active.then(|| {
            let mut snap =
                ph2d_panel_motion_graph::snapshot_from(&motion.doc.graph, &motion.registry);
            snap.probe = probe;
            // The backdrops ride the DOCUMENT, not the graph (they are decoration
            // and never cook), so `snapshot_from` — which only sees the graph —
            // cannot know them. Resolve them here, into the panel's own view type.
            snap.backdrops = motion
                .doc
                .backdrops
                .iter()
                .map(|b| ph2d_panel_motion_graph::GraphBackdropView {
                    id: b.id,
                    x: b.x,
                    y: b.y,
                    w: b.w,
                    h: b.h,
                    color: b.color,
                    title: b.title.clone(),
                })
                .collect();
            // What each card is DOING this frame, read out of the pump's memo (`Cook::peek`,
            // never a second cook): its readout, the mass of its stream (the wire's width),
            // and whether that changed since last frame (the wire's march). A node no sink
            // consumes has no entry and stays blank — which is the diagnosis, not a gap.
            readout::stamp(motion, tapped.as_ref(), &mut snap);
            // **The fold** (doc 57), LAST: everything above published the whole flat
            // graph, and this cuts it down to the level the artist is standing in —
            // folding the nested nodes into cards (which is why it runs after the
            // readouts: a card aggregates what its members are doing) and drawing the
            // outsiders that touch the boundary as ghosts. A document with no
            // subgraphs pays a breadcrumb and nothing else.
            fold::fold(motion, &mut snap);
            // The ONE clock the marching dashes read (`ph2d_core::Playhead`, W4.T7). The panel
            // has none of its own: a flow animation driven by a paint counter would keep
            // marching on a paused graph.
            snap.now = cook_time as f32;
            snap
        }));
        // Stash this frame's tap so the PARAMS panel reads a GPU frame through the
        // SAME door the readouts above just did — one tap, two consumers. `None` on
        // a CPU frame (the memo serves) and one frame behind, matching the memo.
        motion.gpu_tap = tapped;
    }

    // ── Params panel (M1.P1) — published by the params bridge, kept out of the
    // dispatch so this file stays under the shell LOC cap. Needs BOTH panels:
    // the selection comes from the graph, the rows go to params. ──────────────
    #[cfg(all(feature = "panel-motion-graph", feature = "panel-motion-params"))]
    params::publish(motion, &mut hero.store, motion_active, hero.project, toasts);

    if !motion_active {
        return;
    }

    // ── 3. Cook the sink at the tick the PLAYHEAD is on (W4.T7: one clock) ────
    // The render output IS the Output node — the sink follows the graph (wire a
    // chain into an Output node and it draws). `None` (no Output node) → the pump
    // renders nothing. Recomputed each frame so add/delete/rewire just works.
    motion.sinks = output_nodes(&motion.doc.graph);
    // As TOMADAS, recomputadas pelo mesmo motivo dos sinks: elas se curam sozinhas depois
    // de um load, um undo ou uma edição do grafo.
    motion.signal_taps = signal_nodes(&motion.doc.graph);
    // ⚠️ **ARMADAS na bomba, não passadas na chamada** — a marcha tem mais de uma porta (a
    // rota da GPU híbrida marcha por `advance_or_scrub_to_nodes_scoped`), e enquanto a tomada
    // era argumento da porta de sinks um documento híbrido cozinhava, desenhava e **não
    // gritava nada**, com a suíte verde. Medido: a cena `=26` planeja HÍBRIDA.
    // ⚠️ **As tomadas são a UNIÃO de dois pedidos, e cada um mantém o seu nome.** Os
    // sinais precisam das suas; o gizmo de canvas dos deformadores de quadrilátero precisa
    // do stream que ENTRA no nó seleccionado (a caixa a que os offsets dele se referem).
    // Juntar aqui, e não alargar o `signal_taps`, é o que impede o dreno de sinais de um
    // dia ver um nó que não é um `pulse.signal` — *o canal é partilhado, o significado não*.
    let mut taps = motion.signal_taps.clone();
    for n in super::warp_gizmo::taps_for(motion) {
        if !taps.contains(&n) {
            taps.push(n);
        }
    }
    motion.pump.set_taps(&taps);
    // O que este quadro gritou, e o livro-razão de onde isso sai. Limpos aqui, não no dreno:
    // um quadro em que o shell não drena (a ferramenta saiu de foco) não pode deixar o grito
    // de ontem para amanhã.
    motion.signals_out.clear();
    motion.pump.clear_tap_fires();
    // As MEMBRANAS: post-drain, pre-cook. O instante e' propriedade do GRUPO, entao
    // as tres moram numa porta so (`motion_externals`).
    super::motion_externals::publish_all(motion, playhead.time());
    // Time scopes (M2.N1): each `motion.time_remap` node rewrites the clock of
    // its upstream subtree. Rebuilt per frame — one pass over the node list, and
    // empty for a graph with no remapper (the common case), so the cook takes
    // its unscoped path unchanged.
    let scopes = ph2d_node_motion_time_remap::time_scopes(&motion.doc.graph, &motion.registry);
    // Os LEQUES de tempo: um `motion.trail` em `Resampled` tem a própria entrada
    // cozida em N instantes, em vez de lembrada num ring. Vazio para todo grafo
    // sem um (o caso comum), e então o cook toma o caminho de sempre.
    //
    // ⚠️ **Fica ao lado dos escopos de propósito** — os dois são a mesma pergunta
    // (*em que instante a sub-árvore de cima é lida?*) e um deles construído sem
    // o outro é um quadro a cozinhar com metade da resposta. O `fixed_dt` entra
    // aqui porque o `spacing` do rastro conta TIQUES, e a duração de um tique é
    // do shell, não do documento.
    //
    // ⚠️ **TRÊS produtores, um mapa** — o rastro re-cozido, a história da origem do emissor
    // e o atraso por cópia do `motion.clone`. Eles não colidem por construção (o mapa é
    // chaveado por `NodeId`, e um nó é de um tipo só), e a UNIÃO é montada aqui em vez de
    // dentro de um deles: quem sabe que existem três é o shell, e um `time_fans` que
    // chamasse o outro faria de duas crates-folha uma cadeia.
    let mut fans = ph2d_node_motion_trail::time_fans(&motion.doc.graph, &motion.registry, fixed_dt);
    fans.extend(ph2d_node_motion_emitter::time_fans(
        &motion.doc.graph,
        &motion.registry,
        fixed_dt,
    ));
    fans.extend(ph2d_node_motion_clone::fan::time_fans(
        &motion.doc.graph,
        &motion.registry,
        fixed_dt,
    ));
    motion.pump.set_time_fans(fans);
    // ⚠️ **O PLANO DE PREGUIÇA** (doc 89, folha 15) — quais roteadores podem saltar entradas
    // neste quadro. Ele vive ao lado dos leques pela mesma razão que eles vivem ao lado dos
    // escopos: os três são o que o cook precisa de saber e o documento não diz, e os três são
    // reconstruídos por quadro (um ramo que ganhou estado, ou um modo que o artista desligou,
    // tem de sair do plano no quadro em que isso acontece).
    //
    // ⚠️ **Reescreve, nunca acumula** — ver `Cook::set_lazy_branches`. Vazio (o caso comum, o
    // modo nasce desligado) é o caminho de sempre, ao bit.
    motion
        .pump
        .cook
        .set_lazy_branches(ph2d_node_value_switch::lazy::plan(
            &motion.doc.graph,
            &motion.registry,
        ));
    let target = motion_tick(playhead, fixed_dt);

    // ── GPU-resident cook (GPU/M5 Fase 1 + F1.2, ADR-0126) — opt-in preview ──
    // With `PH2D_GPU_COOK=1` a single-sink, unscoped document cooks on the GPU
    // (fully, or hybrid from a CPU boundary). `Handled` = the GPU produced this
    // frame → skip the CPU pump; `FellThrough` = run the pump below. The whole
    // policy + dispatch lives in the `gpu` module (see there); this stays a seam.
    // ⚠️ A bypassed GROUP is short-circuited only on the CPU pump's graph (below),
    // so the GPU path is skipped while one exists — a muted preview must not cook
    // from the un-rewired graph (v1; the group-bypass module documents this).
    if motion.doc.bypassed_subgraphs.is_empty()
        && let gpu::GpuOutcome::Handled = gpu::cook_gpu(motion, gpu, target, fixed_dt, &scopes)
    {
        return;
    }

    // The graph the pump cooks: `doc.graph`, or a clone with every bypassed group
    // rewired to pass input[0] → output[0]. `None` (the common case) means the pump
    // reads the document graph unchanged. `output_nodes`/`time_scopes` above stay on
    // `doc.graph` — a bypass removes no node, so the sinks and scopes are the same.
    let cook = group_bypass::cook_graph(motion);
    for tick in ticks_owed(motion.pump.last_cooked_tick(), target) {
        motion.pump.advance_or_scrub_scoped(
            cook.as_ref().unwrap_or(&motion.doc.graph),
            &motion.registry,
            &motion.sinks,
            tick,
            |t| t as f64 * fixed_dt,
            motion.default_uv_rect,
            motion.default_size,
            &scopes,
        );
    }

    // LOD — the freeze fix (ADR-0154 follow-up). The cook just filled
    // `vector_instances` with one crisp `VectorInstance` per stamped live vector; a
    // grid of 160k is a per-frame freeze (~one Vello fill each). This moves any
    // geometry stamped past the knee onto `instances` as a GPU-instanced tile (which
    // scaled to millions), leaving the below-threshold shapes crisp. It runs ONLY on
    // the CPU pump: a live-vector graph always recuses the GPU path above (it has no
    // `geometry_id` route), so the branch that returns early never carries vectors.
    objects::apply_object_lod(
        &mut motion.pump.instances,
        &mut motion.pump.vector_instances,
        &motion.object_bake,
        objects::LOD_COUNT,
    );
    // A SONDA (`PH2D_PAN_DIAG=1`): o MUNDO de uma amostra de cada rota, depois do
    // cozimento e do LOD — o último sítio antes do desenho.
    if crate::pan_diag::on() {
        let vecs: Vec<[f32; 2]> = motion
            .pump
            .vector_instances
            .iter()
            .map(|v| v.world_pos)
            .collect();
        crate::pan_diag::note_instances(&motion.pump.instances, &vecs);
    }
}

/// **Re-derive everything the graph's SHAPE implies**, after any structural edit —
/// the ONE funnel every mint and every rewire goes through:
///
/// - the **`pre` plumbing** (a sequential node's self-loop follows its `forces` chain);
/// - the **membership** of a node that was just minted (doc 57): created while the
///   artist is inside a group, it belongs to that group — otherwise it would land at
///   the root and **vanish the instant it was created**.
///
/// Both are functions of `graph` vs `before`, and both are wrong the moment somebody
/// adds a fifth way to mint a node and forgets one of them. So there is one door.
#[cfg(feature = "panel-motion-graph")]
pub(super) fn reconcile(motion: &mut MotionState, before: &ph2d_nodegraph::graph::Graph) {
    plumbing::reconcile_after(&mut motion.doc.graph, &motion.registry, before);
    plumbing::seed_text_defaults(&mut motion.doc.graph, &motion.registry, before);
    subgraph::adopt_new(motion, before);
}

#[path = "motion_bridge_remove.rs"]
mod remove;
// Re-exported so `motion_bridge::…` call sites (`dispatch`, `intents`, the tests) are
// unchanged; a private `use` is visible to this module AND its descendants — exactly the
// reach the callers need — and `output_nodes` is called here, the intent handlers by the
// `intents` child.
use remove::output_nodes;
#[cfg(feature = "panel-motion-graph")]
use remove::{apply_delete_selection, apply_disconnect};

/// **Publish the drawn shapes into the cook** (doc 65) — called by the render loop right before the
/// Motion bridge, because that is where the vector document, the world and the entity map are all
/// in hand at once.
///
/// It runs whether or not the Motion tool is active: a graph cooks in the background (the pump is
/// what feeds `render_with_extra`), and a `motion.path` whose curve went stale while the artist was
/// in another tool would be a scene that is wrong until you look at it.
pub(super) fn publish_shapes(
    motion: &mut MotionState,
    sim: &ph2d_ecs::SimWorld,
    scene: &ph2d_vec_scene::VecScene,
    map: &crate::vec_entities::VecEntityMap,
    xforms: &ph2d_vec_scene::VecXforms,
) {
    shapes::publish(&mut motion.pump.cook, sim, scene, map, xforms);
}

/// Publish the world-space **cursor** into the same external table
/// ([`ph2d_nodegraph::external::CURSOR`]) — the editor input a document cannot hold,
/// and what lets `motion.look_at` aim at the mouse.
///
/// ⚠️ Runs LAST of the three publishes: `publish_shapes` clears the table and the
/// objects append to it, so an earlier cursor would be wiped by the shapes of the
/// same frame. The `$` namespace it lands in is the one the artist-name publishes
/// refuse (`shapes::is_reserved`), so the two can never collide.
pub(super) fn publish_cursor(
    motion: &mut MotionState,
    camera: &ph2d_render::Camera2d,
    cursor: (f32, f32),
    split: CenterSplit,
    window: ph2d_host::WindowSize,
) {
    shapes::publish_cursor(&mut motion.pump.cook, camera, cursor, split, window);
}

// The object-bake wiring (`publish_objects`/`bake_objects`/`bake_flip_objects`,
// doc 86 §2 A2/A3) lives in the `objects` membrane module and is re-exported below,
// so `motion_bridge::…` call sites are unchanged and this file stays under the cap.
