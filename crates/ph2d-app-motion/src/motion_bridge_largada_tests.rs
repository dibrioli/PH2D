//! **OS GATES DAS DUAS LARGADAS** — ordem do dono (2026-09-19): *«Se arrastar um nó no grafo em
//! cima de outro nó, eles mudam de posição na cadeia. Se arrastar num nó em cima de uma conexão
//! (linha) […] ele passa a ser conectado naquela linha, contudo, sem quebrar a cadeia.»*
//!
//! ⚠️ **Irmão do [`super::tests`] por RESPONSABILIDADE:** ali mede-se *«mexer num FIO faz o que
//! se pede?»* (mover a ponta, inserir um tipo novo, recusar) e aqui *«largar uma CARTA faz o que
//! se pede?»* — o sujeito é o nó, e o que ele deixa para trás é metade da lei.
//!
//! `super` é `motion_bridge::rewire`.

use super::*;
use crate::motion_state::MotionState;
use ph2d_editor_core::ToastQueue;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// Uma cadeia de tipos, ligados `0 → 0` pela ordem. Devolve os ids.
fn cadeia(motion: &mut MotionState, tipos: &[&str]) -> Vec<NodeId> {
    let mut g = Graph::new();
    let ids: Vec<NodeId> = tipos.iter().map(|t| g.add_node(*t)).collect();
    for par in ids.windows(2) {
        g.connect(Edge {
            from: (par[0], 0),
            to: (par[1], 0),
            delayed: false,
        })
        .expect("a cadeia liga");
    }
    motion.doc.graph = g;
    ids
}

/// Quem alimenta a porta `0` de `n`.
fn fonte(motion: &MotionState, n: NodeId) -> Option<NodeId> {
    motion
        .doc
        .graph
        .edges()
        .iter()
        .find(|e| e.to.0 == n && e.to.1 == 0 && !e.delayed)
        .map(|e| e.from.0)
}

/// ⭐⭐⭐ **OS DOIS TROCAM DE LUGAR NA CADEIA** — a ordem do dono, medida na cadeia inteira.
///
/// `grid → move → scale → output`, e a troca de `scale` com `move` tem de dar
/// `grid → scale → move → output`. ⚠️ **A régua é a CADEIA INTEIRA e não «o `scale` mudou de
/// pai»:** meia troca (quem alimenta muda, quem é alimentado não) deixa o `scale` com dois pais e
/// o `output` órfão, e uma asserção sobre um elo só não vê isso.
#[test]
fn dois_nos_trocam_de_lugar_na_cadeia() {
    let mut motion = MotionState::new();
    let ids = cadeia(
        &mut motion,
        &[
            "motion.grid",
            "motion.move",
            "motion.scale",
            "motion.output",
        ],
    );
    let (grid, mv, sc, out) = (ids[0], ids[1], ids[2], ids[3]);
    let mut toasts = ToastQueue::default();
    swap_in_chain(&mut motion, &mut toasts, sc.0, mv.0, 0.0, 0.0);

    assert_eq!(
        fonte(&motion, sc),
        Some(grid),
        "o `scale` passa a vir do grid"
    );
    assert_eq!(fonte(&motion, mv), Some(sc), "e o `move` vem do `scale`");
    assert_eq!(fonte(&motion, out), Some(mv), "e a saida vem do `move`");
}

/// ⭐⭐ **E AS CARTAS TROCAM DE SÍTIO** — *«mudam de posição»* é as duas coisas. O arrastado fica
/// onde o alvo estava; o alvo vai para onde o arrastado COMEÇOU, que é o que o deslocamento
/// acumulado do arrasto (`back`) diz. ⛔ Sem isto a troca entrega um grafo certo com as duas
/// cartas empilhadas uma sobre a outra.
#[test]
fn as_cartas_trocam_de_sitio() {
    use ph2d_nodegraph::graph::Pos;
    let mut motion = MotionState::new();
    // ⚠️ **Os dois trocados têm de ter ENTRADA:** trocar uma FONTE (`motion.grid`, zero entradas)
    // com um filtro é recusado pela máquina de sempre — uma fonte não pode ficar no meio —, e a
    // 1.ª redacção deste gate media as posições sobre uma troca que nunca aconteceu. *Um gate
    // cujo sujeito é recusado afirma sobre o nada.*
    let ids = cadeia(
        &mut motion,
        &[
            "motion.grid",
            "motion.move",
            "motion.scale",
            "motion.output",
        ],
    );
    let (mv, sc) = (ids[1], ids[2]);
    motion.doc.graph.set_pos(sc, Pos { x: 300.0, y: 40.0 });
    motion.doc.graph.set_pos(mv, Pos { x: 100.0, y: 10.0 });
    let mut toasts = ToastQueue::default();
    // O `scale` foi arrastado de (100, 0) até (300, 40) — logo o deslocamento é (200, 40).
    swap_in_chain(&mut motion, &mut toasts, sc.0, mv.0, 200.0, 40.0);

    assert_eq!(
        motion.doc.graph.pos(sc),
        Some(Pos { x: 100.0, y: 10.0 }),
        "o arrastado fica onde o alvo estava"
    );
    assert_eq!(
        motion.doc.graph.pos(mv),
        Some(Pos { x: 100.0, y: 0.0 }),
        "e o alvo vai para onde o arrastado comecou"
    );
}

/// ⛔⛔ **UMA TROCA QUE NÃO CABE RECUSA, E O GRAFO FICA INTACTO** — os manifestos podem ter
/// contagens de porta diferentes, e as portas trocam pelo ÍNDICE. *Uma troca meia-feita é pior do
/// que nenhuma*, e é a mesma máquina (`connect` + `validate` num clone) que toda esta família usa.
///
/// ⭐ **A segunda espécie de recusa apareceu ao escrever o gate das POSIÇÕES:** trocar uma FONTE
/// (zero entradas) com um filtro é recusado pela mesma máquina — e isso é a lei, não uma
/// limitação: *uma fonte não tem por onde ser alimentada, logo não pode ficar no meio da cadeia.*
#[test]
fn uma_troca_que_nao_cabe_recusa_e_nao_mexe_no_grafo() {
    let mut motion = MotionState::new();
    // O `duplicator` tem DUAS entradas; a `motion.output` tem uma. Trocar as ligações de uma
    // `motion.grid` (nenhuma entrada) com o duplicador deixa a porta 1 dele sem onde ir.
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let forma = g.add_node("source.shape");
    let dup = g.add_node("motion.duplicator");
    let out = g.add_node("motion.output");
    for (from, to, port) in [(forma, dup, 0u16), (grid, dup, 1), (dup, out, 0)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .expect("a cena liga");
    }
    motion.doc.graph = g;
    let antes = motion.doc.graph.edges().to_vec();
    let mut toasts = ToastQueue::default();
    swap_in_chain(&mut motion, &mut toasts, grid.0, dup.0, 0.0, 0.0);
    assert_eq!(
        motion.doc.graph.edges(),
        antes.as_slice(),
        "a troca recusada nao pode deixar o grafo meio rewired"
    );

    // A segunda espécie: uma FONTE não pode ir para o meio da cadeia.
    let mut motion = MotionState::new();
    let ids = cadeia(
        &mut motion,
        &["motion.grid", "motion.move", "motion.output"],
    );
    let antes = motion.doc.graph.edges().to_vec();
    swap_in_chain(&mut motion, &mut toasts, ids[0].0, ids[1].0, 0.0, 0.0);
    assert_eq!(
        motion.doc.graph.edges(),
        antes.as_slice(),
        "uma fonte nao tem por onde ser alimentada, logo nao pode ficar no meio"
    );
}

/// ⭐⭐⭐ **O NÓ ENTRA NO FIO E FECHA A CADEIA DE ONDE SAIU** — as DUAS metades do *«sem quebrar a
/// cadeia»*, na mesma corrida.
///
/// `a → b → c` e `d → e`; enfiar o `b` no fio `d → e` tem de dar `a → c` **e** `d → b → e`.
/// ⛔ **Sem a primeira metade** o artista fica com um buraco onde o nó estava; **sem a segunda**,
/// com o nó ligado em dois sítios ao mesmo tempo.
#[test]
fn um_no_enfiado_num_fio_fecha_a_cadeia_de_onde_saiu() {
    let mut motion = MotionState::new();
    let mut g = Graph::new();
    let a = g.add_node("motion.grid");
    let b = g.add_node("motion.move");
    let c = g.add_node("motion.output");
    let d = g.add_node("motion.grid");
    let e = g.add_node("motion.output");
    for (from, to) in [(a, b), (b, c), (d, e)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        })
        .expect("a cena liga");
    }
    motion.doc.graph = g;
    let mut toasts = ToastQueue::default();
    splice_existing_into_wire(&mut motion, &mut toasts, b.0, e.0, 0);

    assert_eq!(
        fonte(&motion, c),
        Some(a),
        "a cadeia de onde ele saiu FECHOU"
    );
    assert_eq!(fonte(&motion, b), Some(d), "e ele entrou no fio");
    assert_eq!(
        fonte(&motion, e),
        Some(b),
        "sem deixar a outra ponta a pairar"
    );
}

/// ⭐ **E um nó DESCONECTADO também entra** — *«mesmo se estiver desconectado»*. Aqui não há
/// cadeia para fechar, e a ausência dela não pode ser lida como um motivo para recusar.
#[test]
fn um_no_desconectado_tambem_entra_no_fio() {
    let mut motion = MotionState::new();
    let mut g = Graph::new();
    let d = g.add_node("motion.grid");
    let e = g.add_node("motion.output");
    let solto = g.add_node("motion.move");
    g.connect(Edge {
        from: (d, 0),
        to: (e, 0),
        delayed: false,
    })
    .expect("o fio");
    motion.doc.graph = g;
    let mut toasts = ToastQueue::default();
    splice_existing_into_wire(&mut motion, &mut toasts, solto.0, e.0, 0);
    assert_eq!(fonte(&motion, solto), Some(d));
    assert_eq!(fonte(&motion, e), Some(solto));
}

/// ⛔ **Largar um nó no PRÓPRIO fio é inerte** — seria um laço. O painel já não o oferece; esta é
/// a segunda porta, porque a intenção chega por outros caminhos.
#[test]
fn enfiar_um_no_no_proprio_fio_e_inerte() {
    let mut motion = MotionState::new();
    let ids = cadeia(
        &mut motion,
        &["motion.grid", "motion.move", "motion.output"],
    );
    let (mv, out) = (ids[1], ids[2]);
    let antes = motion.doc.graph.edges().to_vec();
    let mut toasts = ToastQueue::default();
    splice_existing_into_wire(&mut motion, &mut toasts, mv.0, out.0, 0);
    assert_eq!(motion.doc.graph.edges(), antes.as_slice());
}

/// ⛔⛔ **O HEAL DE UM NÓ APAGADO FAZ A PONTE PELA PORTA PRINCIPAL** — o mesmo defeito do report
/// do dono, um gesto mais atrás: o `heal_deleted_node` dizia *«PRIMARY input (port 0)»* e usava a
/// `0`, e as duas coisas deixaram de ser a mesma no dia em que o `motion.duplicator` declarou
/// `primary_input = 1`.
///
/// `grid → duplicator(points)` com uma `source.shape` na `shape`: apagar o duplicador tem de
/// ligar o **grid** à saída, nunca a forma. ⚠️ **Nem o tipo nem o `validate` acusam** — as duas
/// entradas dele são `INST_VEC2`, logo a cadeia «curada» pela porta errada é um grafo VÁLIDO que
/// carrega a aparência no lugar das posições.
#[test]
fn o_heal_de_um_duplicador_apagado_faz_a_ponte_pela_principal() {
    let mut motion = MotionState::new();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let forma = g.add_node("source.shape");
    let dup = g.add_node("motion.duplicator");
    let out = g.add_node("motion.output");
    for (from, to, port) in [(forma, dup, 0u16), (grid, dup, 1), (dup, out, 0)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .expect("a cena liga");
    }
    motion.doc.graph = g;
    assert!(heal_deleted_node(&mut motion, dup), "a cadeia cura");
    assert_eq!(
        fonte(&motion, out),
        Some(grid),
        "a ponte e' pelos POINTS (a porta principal), nunca pela shape"
    );
}
