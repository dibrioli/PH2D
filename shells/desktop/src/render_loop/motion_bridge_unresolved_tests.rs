//! **O NOME QUE NÃO RESOLVE, do lado do EDITOR** — irmão dos gates de produtor
//! inerte, e separado deles por ASSUNTO: aqueles medem estrutura, estes medem uma
//! resposta que só existe **depois de um cook**.
//!
//! ⚠️ **Todo gate aqui BOMBEIA antes de perguntar.** Sem o `pump` o memo está vazio,
//! a porta `columns` devolve `None`, e a regra cala-se por desenho — a suíte inteira
//! ficaria VERDE sobre um badge que nunca aparece. É a mesma família da fixture
//! envenenada que a `line/Painter` mediu três vezes: *um harness que não roda o que
//! o produto roda mede o silêncio, não o produto.*

use super::*;
use crate::motion_state::MotionState;
use ph2d_motion_doc::MotionDoc;

/// `grid → value.attribute(attr) → motion.drive → output`, cozido uma vez.
///
/// ⚠️ O `motion.grid` publica `P`/`size`/`Index`/`Count` e **não** `vel` — então
/// `"vel"` é um nome legítimo do vocabulário que ESTA stream não carrega, que é
/// exactamente a forma do engano real (um `value.attribute` posto antes da fonte
/// que produziria a coluna).
fn scene(attr: &str) -> (MotionState, NodeId, NodeId) {
    let mut m = MotionState::new();
    m.doc = MotionDoc::new();
    let grid = m.doc.graph.add_node("motion.grid");
    let at = m.doc.graph.add_node("value.attribute");
    let drive = m.doc.graph.add_node("motion.drive");
    let out = m.doc.graph.add_node("motion.output");
    m.doc.graph.set_text_param(at, "attr", attr);
    for (f, fp, t, tp) in [
        (grid, 0u16, at, 0u16),
        (grid, 0, drive, 0),
        (at, 0, drive, 1),
        (drive, 0, out, 0),
    ] {
        m.doc
            .graph
            .connect(Edge {
                from: (f, fp),
                to: (t, tp),
                delayed: false,
            })
            .expect("wire");
    }
    m.pump.mark_dirty();
    m.pump.pump(
        &m.doc.graph,
        &m.registry,
        &[out],
        1,
        0.0,
        [0.0; 4],
        [1.0; 2],
    );
    (m, at, out)
}

/// **Um nome que a stream de entrada não carrega ganha o badge ⚠.**
#[test]
fn a_name_the_stream_lacks_gets_a_badge() {
    let (m, at, _) = scene("vel");
    assert!(
        inert_reaching_output(&m).contains(&at.0),
        "o `value.attribute` a ler `vel` de uma grade tem de ser marcado"
    );
}

/// **E o CONTROLE: um nome que ela carrega NÃO ganha badge.**
///
/// ⚠️ Sem esta metade um badge posto em todo `value.attribute` passaria no gate
/// acima — e um badge que está sempre aceso é um badge que o artista aprende a
/// ignorar, que é o custo real de um falso positivo.
#[test]
fn a_name_the_stream_carries_gets_none() {
    let (m, at, _) = scene("P");
    assert!(
        !inert_reaching_output(&m).contains(&at.0),
        "`P` está na grade — nada a reportar"
    );
}

/// **O chip de node-help desliga os DOIS diagnósticos**, não só o estrutural.
#[test]
fn node_help_off_silences_the_unresolved_name_too() {
    let (mut m, at, _) = scene("vel");
    assert!(inert_reaching_output(&m).contains(&at.0));
    m.node_help_enabled = false;
    assert!(
        inert_reaching_output(&m).is_empty(),
        "com o node help desligado não há badge nenhum"
    );
}

/// **Clicar o badge EXPLICA e cita o nome, sem tocar no grafo.**
///
/// ⚠️ As duas metades importam. *Explicar* porque não há cura canônica — qual
/// coluna o artista queria é escolha dele, e adivinhar é o que o ADR-0155 proíbe.
/// *Sem tocar no grafo* porque este nó não passa pelo `plan_heal`, e um caminho que
/// caísse nele por engano reestruturaria uma cadeia perfeitamente sã.
#[test]
fn clicking_the_badge_explains_and_leaves_the_graph_alone() {
    let (mut m, at, _) = scene("vel");
    let before = m.doc.graph.clone();
    let mut toasts = ToastQueue::default();
    heal_one(&mut m, &mut toasts, at);
    assert_eq!(
        m.doc.graph.edges().len(),
        before.edges().len(),
        "o grafo não é reestruturado"
    );
    assert_eq!(m.doc.graph.nodes().len(), before.nodes().len());
    let said: Vec<String> = toasts.iter().map(|t| t.message.clone()).collect();
    assert!(
        said.iter().any(|t| t.contains("'vel'")),
        "a mensagem cita o nome que o artista escreveu: {said:?}"
    );
}

/// **Sem cook não há acusação** — a porta `columns` devolve `None` e a regra cala.
///
/// É a metade que torna *"zero falsos positivos"* verificável em vez de prosa: a
/// mesma cena, a mesma pergunta, e a única diferença é ter havido um cook.
#[test]
fn without_a_cook_there_is_no_accusation() {
    let mut m = MotionState::new();
    m.doc = MotionDoc::new();
    let grid = m.doc.graph.add_node("motion.grid");
    let at = m.doc.graph.add_node("value.attribute");
    m.doc.graph.set_text_param(at, "attr", "vel");
    m.doc
        .graph
        .connect(Edge {
            from: (grid, 0),
            to: (at, 0),
            delayed: false,
        })
        .expect("wire");
    assert!(
        inert_reaching_output(&m).is_empty(),
        "sem stream cozida a resposta é DESCONHECIDA, e desconhecido cala"
    );
}

/// **A camada de baixo também honra o chip** — e este gate existe porque a mutação
/// que a apaga **não sangra** nenhum dos outros: os dois chamadores de hoje já
/// saem cedo, logo o guard interno é redundante *através deles*.
///
/// ⚠️ Sem um gate por CAMADA não há como distinguir *"esta linha é load-bearing"* de
/// *"esta linha é redundante"*, e a redundância só sobrevive enquanto ninguém
/// escrever o terceiro chamador. Aqui a camada é exercitada DIRETAMENTE.
#[test]
fn the_inner_layer_honours_the_chip_too() {
    let (mut m, _, _) = scene("vel");
    assert_eq!(
        unresolved_names(&m).len(),
        1,
        "com o chip ligado, um achado"
    );
    m.node_help_enabled = false;
    assert!(
        unresolved_names(&m).is_empty(),
        "o helper cala sozinho, sem depender do chamador"
    );
}

/// **A CENA `=45` de facto ganha o badge** — a metade da instrução do smoke que não
/// está no canvas.
///
/// ⚠️ Sem este gate a mensagem manda o artista *abrir o painel de grafo e procurar
/// um badge*, e uma fiação errada da cena o mandaria procurar o que não existe —
/// que é a forma exacta de uma cena afirmar o que a medição desmente.
#[test]
fn the_compare_scene_earns_the_badge_it_advertises() {
    let mut m = MotionState::new();
    m.doc = MotionDoc::new();
    let sinks = crate::motion_state::conferencia_demos_compare::build_compare_demo_document(
        &mut m.doc,
        &m.registry,
    )
    .expect("a cena monta");
    m.pump.mark_dirty();
    m.pump.pump(
        &m.doc.graph,
        &m.registry,
        &sinks,
        1,
        0.0,
        [0.0; 4],
        [1.0; 2],
    );
    let badges = inert_reaching_output(&m);
    let reader = m
        .doc
        .graph
        .nodes()
        .iter()
        .find(|n| n.type_name == "value.attribute")
        .expect("a cena tem o leitor");
    assert!(
        badges.contains(&reader.id.0),
        "o leitor do nome ausente tem de ser marcado; marcados: {badges:?}"
    );
}
