//! **Rewiring** (Motion Nodes F2, doc 45) — splice a reroute into a wire, and move a wire's
//! end from one input to another. Declared by `motion_bridge` as a `#[path]` sibling, so
//! `super` is `render_loop::motion_bridge`.
//!
//! Both are ONE undo step and both go through the same authority every hand-drawn wire does
//! (`connect` + `validate` on a trial clone). A gesture that half-succeeds — a node spliced
//! in but not wired, a wire unplugged and not re-plugged — would leave the artist with a
//! graph they did not ask for and no single Ctrl+Z to undo it.

use super::subgraph;
use super::{MotionState, plumbing};
use ph2d_editor_core::ToastQueue;
use ph2d_nodegraph::cook::OpResolver;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};
use ph2d_nodegraph::port::PortType;

/// ⭐⭐⭐ **A PORTA por onde um fio entra NESTE nó** — a lei partilhada
/// ([`ph2d_node_registry::landing_port`]) aplicada a um nó que JÁ EXISTE, num grafo qualquer (o
/// documento ou um `trial` a meio de uma edição).
///
/// `ty` é o tipo do fio; `None` quer dizer *«não sei»* e deixa a principal ganhar sem prova — o
/// `validate` a seguir tem a última palavra, como em toda esta família.
fn entrada_de(
    g: &Graph,
    reg: &ph2d_node_registry::NodeRegistry,
    n: NodeId,
    ty: Option<PortType>,
) -> u16 {
    let Some(man) = g
        .node(n)
        .and_then(|x| reg.resolve(x.type_id()))
        .map(|op| op.manifest())
    else {
        return 0;
    };
    ph2d_node_registry::landing_port(reg.primary_input(man.id), man.inputs.len(), |i| {
        ty.is_none_or(|t| man.inputs.get(i as usize).is_some_and(|p| p.ty == t))
    })
    .unwrap_or(0)
}

/// ⭐⭐ **LIGAR `u → nó → v` NUM `trial`**, com a autoridade de sempre (`connect` + `validate`).
///
/// ⚠️ **As DUAS rotas de splice acabam aqui** — a que CRIA o nó ([`splice_into_wire`]) e a que
/// move um que JÁ EXISTE ([`splice_existing_into_wire`]) —, e o corpo delas era **byte a byte o
/// mesmo**. *Uma lei escrita em dois sítios ainda não é uma lei*, e a prova de mutação desta wave
/// tropecçou nisso antes de um humano o ver: a âncora casava duas vezes.
///
/// `false` quer dizer *«o `trial` ficou inutilizável»* — o chamador deita-o fora e avisa.
fn liga_pelo_meio(
    trial: &mut Graph,
    reg: &ph2d_node_registry::NodeRegistry,
    edge: Edge,
    node: NodeId,
    entrada: u16,
) -> bool {
    trial
        .connect(Edge {
            from: edge.from,
            to: (node, entrada),
            delayed: false,
        })
        .is_ok()
        && trial
            .connect(Edge {
                from: (node, 0),
                to: edge.to,
                delayed: false,
            })
            .is_ok()
        && trial.validate(reg).is_ok()
}

/// The reroute node type that fits a given port type. `None` when the wire carries something
/// no reroute speaks — which cannot happen today (the whole library uses three port types and
/// there is a reroute for each), but a new port type must not silently splice the wrong dot.
fn reroute_for(ty: PortType) -> Option<&'static str> {
    use ph2d_node_util_reroute as reroute;
    for (manifest, name) in [
        (&reroute::MANIFEST_STREAM, reroute::TYPE_STREAM),
        (&reroute::MANIFEST_VALUE, reroute::TYPE_VALUE),
        (&reroute::MANIFEST_PULSE, reroute::TYPE_PULSE),
    ] {
        if manifest.outputs[0].ty == ty {
            return Some(name);
        }
    }
    None
}

/// The port type a wire carries — read off its SOURCE output.
fn wire_type(motion: &MotionState, from: NodeId, port: u16) -> Option<PortType> {
    motion
        .doc
        .graph
        .node(from)
        .and_then(|n| motion.registry.resolve(n.type_id()))
        .and_then(|op| op.manifest().outputs.get(port as usize))
        .map(|p| p.ty)
}

/// **Splice a reroute node into the wire landing on `(to_node, to_port)`** — the dot the
/// artist double-clicked onto it (doc 45).
///
/// The type is chosen from the WIRE, not from the artist: the gesture already said which
/// wire, and a wire knows what it carries. So there is no menu, no wrong choice to make, and
/// the three reroute node types are an implementation detail the artist never meets.
///
/// The reroute is a **pass-through**, so the render must not move a pixel — that is the
/// property that makes this gesture safe to reach for while tidying a live scene.
/// The wire feeding `(to, port)`, if any — the ordinary edge, never a managed `pre`.
fn wire_edge(motion: &MotionState, to: NodeId, port: u16) -> Option<Edge> {
    motion
        .doc
        .graph
        .edges()
        .iter()
        .find(|e| e.to.0 == to && e.to.1 == port && !e.delayed)
        .copied()
}

/// Insert `type_name` into `edge` at (x,y): on a TRIAL clone, add the node, cut the wire and
/// rewire source → new:0 → target; commit only if the whole thing VALIDATES — else `refuse_msg`
/// and the ORIGINAL wire stays (a splice that dangles a node and cuts a wire is worse than no
/// splice). One undo step; re-cooks; selects the new node. The single door the fixed reroute
/// and the artist-chosen node both go through — two callers, one splice.
fn splice_into_wire(
    motion: &mut MotionState,
    toasts: &mut ToastQueue,
    edge: Edge,
    type_name: &str,
    x: f32,
    y: f32,
    refuse_msg: &str,
) {
    let pre = motion.doc.clone();
    let mut trial: Graph = motion.doc.graph.clone();
    let node = trial.add_node(type_name.to_string());
    trial.set_pos(node, Pos { x, y });
    trial.disconnect(edge.to.0, edge.to.1);
    // ⭐⭐ **O fio entra pela porta que o TIPO declara como principal**, não pela `0`.
    //
    // ⛔⛔ Report do Enio (2026-09-01): *«automaticamente o fio do emitter entra no input
    // errado do duplicator (o da shape ou objeto)»*. O `0` cravado aqui é certo para os 133
    // tipos cujo fluxo principal É a primeira porta — e **silenciosamente errado** para o
    // `motion.duplicator`, cujas duas entradas são do MESMO tipo (`INST_VEC2`), de modo que
    // nem o `validate` nem o tipo podem acusar. Ver
    // [`ph2d_node_registry::NodeRegistry::register_primary_input`]: ausente ⇒ `0`.
    // ⚠️ Desde 2026-09-19 pela PORTA que as três rotas partilham ([`entrada_de`]) — e com o
    // predicado do TIPO, que esta rota não tinha: uma principal que não aceita o fio passava a
    // recusar o splice inteiro, quando havia uma porta que o levava. *Uma preferência que não
    // cede é um bloqueio.*
    let entrada = entrada_de(
        &trial,
        &motion.registry,
        node,
        wire_type(motion, edge.from.0, edge.from.1),
    );
    let ok = liga_pelo_meio(&mut trial, &motion.registry, edge, node, entrada);

    if !ok {
        toasts.push(ph2d_editor_core::Toast::info(refuse_msg));
        return;
    }
    motion.doc.graph = trial;
    super::reconcile(motion, &pre.graph);
    motion.history.push_undo(pre);
    motion.pump.mark_dirty(); // a spliced node is IN the graph — the graph changed
    ph2d_panel_motion_graph::request_graph_selection(vec![node.0]);
}

pub(super) fn splice_reroute(
    motion: &mut MotionState,
    toasts: &mut ToastQueue,
    to_node: u32,
    to_port: u16,
    x: f32,
    y: f32,
) {
    let Some(edge) = wire_edge(motion, NodeId(to_node), to_port) else {
        return; // no wire there (or a managed `pre`) — nothing to splice into
    };
    // The reroute type is derived from what the wire CARRIES (the artist chose nothing).
    let Some(ty) = wire_type(motion, edge.from.0, edge.from.1) else {
        return;
    };
    let Some(type_name) = reroute_for(ty) else {
        toasts.push(ph2d_editor_core::Toast::info(ph2d_i18n::tr(
            "app.motion.motion_bridge_rewire.no_reroute_exists_for_this_kind_of_wire",
        )));
        return;
    };
    splice_into_wire(
        motion,
        toasts,
        edge,
        type_name,
        x,
        y,
        ph2d_i18n::tr("app.motion.motion_bridge_rewire.can_t_reroute_this_wire"),
    );
}

/// Insert the artist-CHOSEN `type_name` into the wire feeding `(to_node, to_port)` — the
/// R-press-a-wire → add-menu generalisation of [`splice_reroute`]. Same trial-validate-or-refuse
/// (an unfitting type just does not splice, the wire stays), one undo step, one door.
pub(super) fn splice_node(
    motion: &mut MotionState,
    toasts: &mut ToastQueue,
    to_node: u32,
    to_port: u16,
    type_name: &str,
    x: f32,
    y: f32,
) {
    let Some(edge) = wire_edge(motion, NodeId(to_node), to_port) else {
        return;
    };
    splice_into_wire(
        motion,
        toasts,
        edge,
        type_name,
        x,
        y,
        ph2d_i18n::tr("app.motion.motion_bridge_rewire.can_t_insert_this_node_into_this_wire"),
    );
}

/// **Move a wire's end** from the input it was pulled off to wherever it was dropped (doc
/// 45). `new_to = None` → dropped on empty canvas, so the wire is simply unplugged.
///
/// One undo step: unplug + plug. A `Disconnect` followed by a `Connect` would need two
/// Ctrl+Z to put back — and if the second half were refused, would leave the wire destroyed
/// by a gesture that was only ever asking to move it.
///
/// **A refused landing keeps the ORIGINAL wire.** The artist tried to move a wire somewhere
/// it cannot go; the answer to that is "no", not "the wire you had is gone too".
pub(super) fn move_wire_end(
    motion: &mut MotionState,
    toasts: &mut ToastQueue,
    from_node: u32,
    from_port: u16,
    old_to_node: u32,
    old_to_port: u16,
    new_to: Option<(u32, u16)>,
) {
    let old = (NodeId(old_to_node), old_to_port);
    if plumbing::is_managed_pre(&motion.doc.graph, &motion.registry, old.0, old.1) {
        toasts.push(ph2d_editor_core::Toast::info(ph2d_i18n::tr(
            "app.motion.motion_bridge_rewire.state_wiring_is_automatic_disconnect_the_chain_f",
        )));
        return;
    }

    let pre = motion.doc.clone();
    let mut trial: Graph = motion.doc.graph.clone();
    // The end being pulled off may be a PARAM socket (doc 58) — it is drawn like a wire, so
    // it moves like a wire. Resolved before the edit, because the answer lives in the graph
    // the edit is about to change.
    let old_param = subgraph::param_at_in(&trial, &motion.registry, old.0, old.1);
    if !subgraph::unplug_in(&mut trial, old_param.as_deref(), old.0, old.1) {
        return; // the wire is already gone (a stale gesture) — do nothing, quietly
    }

    if let Some((n, p)) = new_to {
        // Dropped back where it came from: nothing happened. No undo step for a gesture that
        // changed nothing.
        if (n, p) == (old_to_node, old_to_port) {
            return;
        }
        // Landing on ANOTHER param's socket: re-drive that one. (Landing on a param that has
        // no wire yet is not a socket at all — the drop opens the body menu instead, and the
        // panel sends `DriveParam`.)
        let dst_param = subgraph::param_at_in(&trial, &motion.registry, NodeId(n), p);
        let landed = match &dst_param {
            Some(name) => trial
                .drive_param(NodeId(n), name.as_str(), (NodeId(from_node), from_port))
                .is_ok(),
            None => trial
                .connect(Edge {
                    from: (NodeId(from_node), from_port),
                    to: (NodeId(n), p),
                    delayed: false,
                })
                .is_ok(),
        } && trial.validate(&motion.registry).is_ok();
        if !landed {
            toasts.push(ph2d_editor_core::Toast::info(ph2d_i18n::tr(
                "app.motion.motion_bridge_rewire.can_t_move_the_wire_there_the_original_stays",
            )));
            return;
        }
    }

    motion.doc.graph = trial;
    super::reconcile(motion, &pre.graph);
    motion.history.push_undo(pre);
    motion.pump.mark_dirty();
}

/// **Delete-and-reconnect** (Blender's Ctrl+X): before a node leaves the graph, bridge the wire
/// feeding its PRIMARY input (port 0) to every wire leaving its PRIMARY output, so a node
/// removed from the middle of a chain leaves the chain HEALED instead of severed. This is
/// [`splice_into_wire`] run in reverse, through the same trial-validate-or-refuse authority.
///
/// On success the node is removed and `true` is returned; on `false` the node is left in place
/// and the caller removes it the plain way (today's behaviour). It heals NOTHING — leaving the
/// wire cut — when the bridge would be ambiguous or wrong: no ordinary wire feeds port 0 (a
/// source node has nothing to carry through), no wire leaves port 0 (nothing downstream to
/// keep), or the bridged graph does not VALIDATE. The `!e.delayed` filters skip engine-managed
/// `pre` plumbing (an input takes exactly one source, so a bridged target is never a `pre`),
/// which the delete's `reconcile` re-derives regardless.
pub(super) fn heal_deleted_node(motion: &mut MotionState, nid: NodeId) -> bool {
    // The wire feeding port 0 — the stream this node sits on. Its source is what carries through.
    // ⛔⛔ **A ponte é pela porta PRINCIPAL e não pela `0`** (2026-09-19): o doc acima dizia
    // *«PRIMARY input (port 0)»*, e as duas coisas deixaram de ser a mesma no dia em que o
    // `motion.duplicator` declarou `primary_input = 1`. Apagar um duplicador do meio de uma
    // cadeia fazia a ponte pela `shape` — a corrente de POSIÇÕES ficava cortada e a cadeia
    // «curada» passava a carregar a aparência. *Nem o tipo nem o `validate` acusam: as duas
    // entradas dele são do mesmo tipo.*
    let principal = entrada_de(&motion.doc.graph, &motion.registry, nid, None);
    let Some(source) = motion
        .doc
        .graph
        .edges()
        .iter()
        .find(|e| e.to.0 == nid && e.to.1 == principal && !e.delayed)
        .map(|e| e.from)
    else {
        return false; // nothing feeds the primary input (a source node) — no chain to heal
    };
    // Every wire leaving port 0 — the targets that must keep their input after nid is gone.
    let targets: Vec<(NodeId, u16)> = motion
        .doc
        .graph
        .edges()
        .iter()
        .filter(|e| e.from.0 == nid && e.from.1 == 0 && !e.delayed)
        .map(|e| e.to)
        .collect();
    if targets.is_empty() {
        return false; // nothing downstream of port 0 — deleting an end, not a middle
    }

    let mut trial: Graph = motion.doc.graph.clone();
    trial.remove_node(nid); // drops nid and all its edges; targets are now dangling
    for &(tn, tp) in &targets {
        if trial
            .connect(Edge {
                from: source,
                to: (tn, tp),
                delayed: false,
            })
            .is_err()
        {
            return false; // a bridge that will not connect — leave the node for plain removal
        }
    }
    if trial.validate(&motion.registry).is_err() {
        return false; // the healed chain is ill-typed — the caller severs it the plain way
    }
    motion.doc.graph = trial;
    true
}

/// ⭐⭐⭐ **TROCAR DOIS NÓS DE LUGAR NA CADEIA** — ordem do dono (2026-09-19): *«Se arrastar um nó
/// no grafo em cima de outro nó, eles mudam de posição na cadeia»*.
///
/// A lei é uma só: **cada um passa a ter as ligações do outro**. Quem alimentava `a` alimenta `b`,
/// quem `a` alimentava passa a ser alimentado por `b`, e um fio que ia de `a` para `b` passa a ir
/// de `b` para `a`. ⚠️ **Escrita como um remapeamento de TODAS as arestas**, e não como um par de
/// casos (adjacentes / distantes): o caso adjacente é o mesmo remapeamento, e escrevê-lo à parte
/// é a segunda lei que diverge da primeira no dia em que alguém mexer numa delas.
///
/// ⚠️ **As portas trocam pelo ÍNDICE**, o que pode não caber (dois nós com manifestos diferentes)
/// — e aí a máquina de sempre (`connect` + `validate` num clone) RECUSA e o grafo fica intacto.
/// *Uma troca meia-feita é pior do que nenhuma.*
///
/// ⛔⛔ **Ela NÃO empurra um passo de undo**: corre DENTRO do parênteses `BeginDrag`/`EndDrag` que
/// o arrasto abriu, e é o `EndDrag` que comita. Empurrar aqui daria ao artista **dois** Ctrl+Z
/// para desfazer **um** gesto.
pub(super) fn swap_in_chain(
    motion: &mut MotionState,
    toasts: &mut ToastQueue,
    a: u32,
    b: u32,
    back_dx: f32,
    back_dy: f32,
) {
    let (a, b) = (NodeId(a), NodeId(b));
    if a == b || motion.doc.graph.node(a).is_none() || motion.doc.graph.node(b).is_none() {
        return;
    }
    let pre = motion.doc.clone();
    let mut trial: Graph = motion.doc.graph.clone();
    // ⚠️ **As `delayed` ficam de fora**: são o `pre` que o motor mantém, e o `reconcile` no fim
    // re-deriva-as para a forma nova do grafo.
    let arestas: Vec<Edge> = trial
        .edges()
        .iter()
        .filter(|e| !e.delayed)
        .cloned()
        .collect();
    for e in &arestas {
        trial.disconnect(e.to.0, e.to.1);
    }
    let troca = |n: NodeId| {
        if n == a {
            b
        } else if n == b {
            a
        } else {
            n
        }
    };
    let ok = arestas.iter().all(|e| {
        trial
            .connect(Edge {
                from: (troca(e.from.0), e.from.1),
                to: (troca(e.to.0), e.to.1),
                delayed: false,
            })
            .is_ok()
    }) && trial.validate(&motion.registry).is_ok();
    if !ok {
        toasts.push(ph2d_editor_core::Toast::info(ph2d_i18n::tr(
            "app.motion.motion_bridge_rewire.these_two_nodes_cannot_trade_places",
        )));
        return;
    }
    // ⭐ **E as CARTAS trocam de sítio também.** O arrastado fica onde o alvo estava; o alvo vai
    // para onde o arrastado COMEÇOU — que a shell já não sabe, porque aplicou cada `MoveNodes` ao
    // vivo, e por isso o painel manda o deslocamento acumulado. *Trocar a cadeia e deixar as duas
    // cartas empilhadas entrega um grafo certo e ilegível.*
    let (pa, pb) = (motion.doc.graph.pos(a), motion.doc.graph.pos(b));
    motion.doc.graph = trial;
    if let (Some(pa), Some(pb)) = (pa, pb) {
        motion.doc.graph.set_pos(a, pb);
        motion.doc.graph.set_pos(
            b,
            Pos {
                x: pa.x - back_dx,
                y: pa.y - back_dy,
            },
        );
    }
    super::reconcile(motion, &pre.graph);
    motion.pump.mark_dirty();
    // ⭐ **O eco**: os dois nós e os fios que os ligam agora — ver `MotionState::piscada`.
    let fios = fios_de(motion, &[a, b]);
    motion.piscada = Some(crate::motion_state::PiscadaPendente {
        nos: vec![a.0, b.0],
        fios,
        inicio: motion.ui_now,
    });
}

/// Os fios (pela ponta de chegada) que TOCAM algum destes nós — o conjunto que um eco de largada
/// acende ao lado das cartas. ⚠️ Lido DEPOIS da edição, de propósito: o que pisca é a fiação
/// NOVA, que é o que o artista precisa de ler.
fn fios_de(motion: &MotionState, nos: &[NodeId]) -> Vec<(u32, u16)> {
    motion
        .doc
        .graph
        .edges()
        .iter()
        .filter(|e| !e.delayed && (nos.contains(&e.from.0) || nos.contains(&e.to.0)))
        .map(|e| (e.to.0.0, e.to.1))
        .collect()
}

/// ⭐⭐⭐ **ENFIAR UM NÓ QUE JÁ EXISTE NUM FIO** — ordem do dono (2026-09-19): *«Se arrastar num nó
/// em cima de uma conexão (linha) mesmo se o nó já está conectado em cadeia ou mesmo se estiver
/// desconectado, ele passa a ser conectado naquela linha, contudo, sem quebrar a cadeia»*.
///
/// ⚠️⚠️ **O *«sem quebrar a cadeia»* tem DUAS metades e nenhuma basta sozinha:**
/// 1. a cadeia de **ONDE ELE SAI** fecha-se — quem o alimentava passa a alimentar quem ele
///    alimentava (a mesma ponte do [`heal_deleted_node`], sem apagar o nó);
/// 2. a cadeia **ONDE ELE ENTRA** continua ligada — `u → nó → v`, nunca `u → nó` com o `v` a
///    pairar.
///
/// *Com a (1) sozinha o artista fica com um buraco onde o nó estava; com a (2) sozinha fica com
/// o nó ligado em dois sítios ao mesmo tempo.*
///
/// ⛔⛔ **Largar um nó no PRÓPRIO fio é inerte — e isso é uma propriedade da MÁQUINA, não uma
/// guarda.** A 1.ª redação tinha um `if` a recusá-lo, e **a mutação que o apagava NÃO
/// SANGROU**: sem ele o splice tenta ligar `nó → nó`, que é um ciclo, e o `connect` recusa-o como
/// recusa qualquer outro — o documento fica intacto na mesma. *Uma linha que a mutação não
/// consegue matar não é lei, é comentário com sintaxe de código* (`CLAUDE.md` §5.0) ⇒ ela saiu, e
/// o gate ficou, a medir a recusa de quem de facto a faz. ⚠️ O artista passa a ver o toast da
/// recusa em vez de um silêncio, o que é melhor.
///
/// ⛔⛔ Como a irmã acima, ela **não empurra undo**: vive dentro do parênteses do arrasto.
pub(super) fn splice_existing_into_wire(
    motion: &mut MotionState,
    toasts: &mut ToastQueue,
    node: u32,
    to_node: u32,
    to_port: u16,
) {
    let node = NodeId(node);
    let Some(edge) = wire_edge(motion, NodeId(to_node), to_port) else {
        return;
    };
    if motion.doc.graph.node(node).is_none() {
        return;
    }
    let pre = motion.doc.clone();
    let mut trial: Graph = motion.doc.graph.clone();

    // (1) SAIR da cadeia onde ele está, fechando-a.
    let principal = entrada_de(&trial, &motion.registry, node, None);
    let fonte = trial
        .edges()
        .iter()
        .find(|e| e.to.0 == node && e.to.1 == principal && !e.delayed)
        .map(|e| e.from);
    let destinos: Vec<(NodeId, u16)> = trial
        .edges()
        .iter()
        .filter(|e| e.from.0 == node && e.from.1 == 0 && !e.delayed)
        .map(|e| e.to)
        .collect();
    let meus: Vec<(NodeId, u16)> = trial
        .edges()
        .iter()
        .filter(|e| !e.delayed && (e.from.0 == node || e.to.0 == node))
        .map(|e| e.to)
        .collect();
    for (n, p) in meus {
        trial.disconnect(n, p);
    }
    if let Some(f) = fonte {
        for d in &destinos {
            // ⚠️ A ponte é **best-effort**: um destino que já não aceite a fonte (o tipo mudou de
            // lado com o nó fora do caminho) fica desligado, e o `validate` no fim decide se o
            // conjunto ainda é um grafo. *Recusar o gesto inteiro por causa de um ramo lateral
            // seria punir o artista pela forma da cadeia dele.*
            let _ = trial.connect(Edge {
                from: f,
                to: *d,
                delayed: false,
            });
        }
    }

    // (2) ENTRAR no fio.
    trial.disconnect(edge.to.0, edge.to.1);
    let entrada = entrada_de(
        &trial,
        &motion.registry,
        node,
        wire_type(motion, edge.from.0, edge.from.1),
    );
    let ok = liga_pelo_meio(&mut trial, &motion.registry, edge, node, entrada);
    if !ok {
        toasts.push(ph2d_editor_core::Toast::info(ph2d_i18n::tr(
            "app.motion.motion_bridge_rewire.can_t_insert_this_node_into_this_wire",
        )));
        return;
    }
    motion.doc.graph = trial;
    super::reconcile(motion, &pre.graph);
    motion.pump.mark_dirty();
    let fios = fios_de(motion, &[node]);
    motion.piscada = Some(crate::motion_state::PiscadaPendente {
        nos: vec![node.0],
        fios,
        inicio: motion.ui_now,
    });
}

#[cfg(test)]
#[path = "motion_bridge_rewire_tests.rs"]
mod tests;

/// Os gates e as sondas do report do duplicator (2026-09-01) — irmão cortado do acima no
/// teto de LOC, por RESPONSABILIDADE: aquele pergunta *«mexer num fio faz o que se pede?»* e
/// este *«o que acontece quando o nó inserido é o `motion.duplicator`?»*.
#[cfg(test)]
#[path = "motion_bridge_rewire_duplicator_tests.rs"]
mod duplicator_tests;

/// Conduzir um param por fio (doc 58) — irmão cortado no teto de LOC, por RESPONSABILIDADE:
/// um param não tem porta, então a maquinaria e a recusa dele são outras.
#[cfg(test)]
#[path = "motion_bridge_drive_param_tests.rs"]
mod drive_param_tests;

/// A tee contra o nó fundido (doc 100) — a sonda que mede se `value.lfo → motion.drive` é o
/// `motion.oscillator` ao bit, se os dois chegam ao device, e o que cada um custa.
#[cfg(test)]
#[path = "motion_bridge_tee_probe_tests.rs"]
mod tee_probe_tests;

/// As duas LARGADAS de uma carta (ordem do dono, 2026-09-19) — irmão por RESPONSABILIDADE: ali o
/// sujeito é o FIO, aqui é o NÓ, e o que ele deixa para trás é metade da lei.
#[cfg(test)]
#[path = "motion_bridge_largada_tests.rs"]
mod largada_tests;
