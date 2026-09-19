//! Gates da cena `=124` — **as posições e a marca**.
//!
//! ⚠️⚠️ **O que estes gates NÃO podem medir, e porquê:** a fileira de baixo passa por um
//! `source.shape`, e a geometria dele é assada pela SHELL — num arnês headless ele coze `n = 0`.
//! Medido sobre a cena `=121`, que funciona no app e dá exactamente o mesmo zero aqui. ⇒ *a
//! metade com forma prova-se pela ESTRUTURA do grafo; a corrente dela é do smoke do dono.*

use super::*;
use ph2d_node_registry::NodeRegistry;

fn cena() -> (MotionDoc, NodeRegistry, Vec<NodeId>) {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    let mut doc = MotionDoc::default();
    let sinks = build(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipada");
    (doc, reg, sinks)
}

/// ⭐⭐⭐ **A FILEIRA DE CIMA É SÓ POSIÇÕES — a metade que a cena existe para mostrar.**
///
/// ⚠️ **A régua é a mesma porta que o lowering e o gizmo usam** (`tem_aparencia`): um predicado
/// próprio aqui divergiria no dia em que uma origem de aparência nova nascesse, e o gate passaria
/// a afirmar uma coisa sobre uma cena que desenha outra.
#[test]
fn a_fileira_de_cima_nao_traz_aparencia() {
    let (doc, reg, sinks) = cena();
    assert_eq!(sinks.len(), 2, "duas fileiras");
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    cook.advance_tick(&doc.graph, &reg, 0.0).expect("avanca");
    let s = cook.cook(&doc.graph, &reg, sinks[0], 0.0).expect("coze");
    let c = s[0].as_stream();
    let n = (LADO * LADO) as usize;
    assert_eq!(c.count(), n, "a fileira de cima tem {n} posicoes");
    assert!(
        !ph2d_eval_motion::tem_aparencia(c),
        "ela nao pode trazer aparencia -- e' o que faz dela MARCAS e nao pecas"
    );
}

/// ⭐⭐ **A FILEIRA DE BAIXO É O CONTROLO, e a estrutura dela é a prova que este arnês alcança.**
///
/// ⚠️ **Sem ela a cena não ensina nada:** *«não desenha»* e *«está partido»* têm o mesmo aspecto
/// no ecrã, e o que os separa é ver a MESMA nuvem a virar peças assim que uma forma chega. ⇒ o
/// gate exige o duplicador, a forma, e que a forma entre na porta que o manifesto declara.
#[test]
fn a_fileira_de_baixo_veste_a_mesma_nuvem() {
    let (doc, reg, _) = cena();
    let g = &doc.graph;
    let tipo = |n: &ph2d_nodegraph::graph::NodeInstance| n.type_name.clone();
    let acha = |t: &str| g.nodes().iter().find(|n| tipo(n) == t).map(|n| n.id);
    let dup = acha("motion.duplicator").expect("a cena tem um duplicador");
    let forma = acha("source.shape").expect("a cena tem uma forma");
    let entradas: Vec<(u16, ph2d_nodegraph::graph::NodeId)> = g
        .edges()
        .iter()
        .filter(|e| e.to.0 == dup)
        .map(|e| (e.to.1, e.from.0))
        .collect();
    assert!(
        entradas.contains(&(0, forma)),
        "a forma tem de entrar na porta 0 do duplicador, e as entradas sao {entradas:?}"
    );
    // ⚠️ E a porta `1` tem de vir de uma grelha (por um `motion.move`, que é como a fileira
    // desce): sem esta metade, um duplicador ligado só à forma passaria.
    assert!(
        entradas.iter().any(|(p, _)| *p == 1),
        "os PONTOS tem de entrar na porta 1: {entradas:?}"
    );
    // O `reg` é o que valida o grafo no `cena()`; nomeá-lo aqui mantém a leitura honesta.
    let _ = reg;
}

/// ⭐⭐ **A GRELHA CABE INTEIRA NO TECTO DO GIZMO, e a folga é o assunto.**
///
/// ⚠️ A `=2` (o demo de PERFORMANCE que o report confundiu com esta) entrega `129 600` posições,
/// `31×` o tecto — ali a amostra ENGATA e o que se vê é uma nuvem escolhida. Aqui não engata,
/// logo **o artista vê a grelha inteira, posição a posição**, que é o que um smoke de marcas
/// precisa.
#[test]
fn a_grelha_cabe_inteira_no_tecto_do_gizmo() {
    let n = (LADO * LADO) as usize;
    assert!(
        n <= crate::ponto_gizmo::MAX_PONTOS,
        "{n} posicoes contra um tecto de {} -- a amostra engataria e a cena mostraria uma \
         escolha em vez da grelha",
        crate::ponto_gizmo::MAX_PONTOS
    );
    // ⛔ E o CONTROLO: o número do dono (`20 × 20`) é o que está MONTADO, não outro que por acaso
    // também caiba. *Sem esta metade, a cena podia derivar para `3 × 3` e o gate ficava verde.*
    let (doc, ..) = cena();
    let g = &doc.graph;
    let grelhas: Vec<f32> = g
        .nodes()
        .iter()
        .filter(|n| n.type_name == "motion.grid")
        .filter_map(|n| {
            g.node_params()
                .get(&n.id)
                .and_then(|m| m.get("rows"))
                .copied()
        })
        .collect();
    assert_eq!(
        grelhas,
        vec![LADO, LADO],
        "as duas fileiras sao {LADO}x{LADO}"
    );
}

/// ⚠️ **AS DUAS FILEIRAS NÃO SE SOBREPÕEM** — a cena tem de se ler como DUAS coisas.
///
/// A régua mede a nuvem que a fileira de baixo produz ANTES do duplicador (que é a parte deste
/// grafo que um arnês headless alcança) contra a de cima.
#[test]
fn as_duas_fileiras_ficam_separadas() {
    let (doc, reg, sinks) = cena();
    let g = &doc.graph;
    // A cabeça da fileira de baixo: o `motion.move` que a desce.
    let desce = g
        .nodes()
        .iter()
        .find(|n| n.type_name == "motion.move")
        .map(|n| n.id)
        .expect("a fileira de baixo desce por um `motion.move`");
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    cook.advance_tick(g, &reg, 0.0).expect("avanca");
    let faixa = |n: ph2d_nodegraph::graph::NodeId, cook: &mut ph2d_nodegraph::cook::Cook| {
        let o = cook.cook(g, &reg, n, 0.0).expect("coze");
        let Some(ph2d_nodegraph::attr::Column::Vec2(p)) = o[0].as_stream().get("P") else {
            panic!("sem P")
        };
        p.iter().fold((f32::MAX, f32::MIN), |(lo, hi), q| {
            (lo.min(q[1]), hi.max(q[1]))
        })
    };
    let (_, topo_de_baixo) = faixa(desce, &mut cook);
    let (base_de_cima, _) = faixa(sinks[0], &mut cook);
    assert!(
        topo_de_baixo < base_de_cima,
        "a de baixo sobe ate' {topo_de_baixo} e a de cima comeca em {base_de_cima}"
    );
}

/// O roteiro do dono nomeia o que aparece na tela, e nada que não apareça.
#[test]
fn o_roteiro_nomeia_o_que_a_cena_tem() {
    let texto = include_str!("motion_state_pontos_demo.rs");
    for nome in ["Grid", "Duplicator", "Shape", "Gap X", "Gap Y"] {
        assert!(
            texto.contains(nome),
            "o roteiro tem de nomear {nome:?}, que e' o que o artista procura na tela"
        );
    }
}
