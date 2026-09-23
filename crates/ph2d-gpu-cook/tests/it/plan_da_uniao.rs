//! ⭐⭐⭐ **O PLANO DA UNIÃO de várias saídas** (doc 119 W2) — sem device, corre em toda lane.
//!
//! A cerca que manda para a CPU todo grafo com mais de um sink (`motion.sinks.len() != 1`) cai
//! em três passos, e este é o primeiro: *que nós a placa corre quando há N saídas?* A resposta é
//! a UNIÃO das cadeias, com cada nó **uma vez** — e a razão de não ser «N planos de um sink
//! cozinhados lado a lado» é o laço: um nó que alimenta um `pre` cozinhado duas vezes por tique
//! avança a simulação duas vezes.
//!
//! ⚠️ **A régua é o plano de UM sink**, que a casa já gateia inteira (`plan_analysis`,
//! `boundary_arity`): a união afirma-se CONTRA ele, nunca contra uma lista escrita à mão.

use ph2d_gpu_cook::{DrivenParams, GpuPlan, GpuSource, plan_driven, plan_driven_many};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use std::collections::BTreeSet;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_grid::register(&mut reg).unwrap();
    ph2d_node_motion_output::register(&mut reg).unwrap();
    ph2d_node_motion_move::register(&mut reg).unwrap();
    ph2d_node_motion_integrate::register(&mut reg).unwrap();
    ph2d_node_motion_emitter::register(&mut reg).unwrap();
    ph2d_node_force_wind::register(&mut reg).unwrap();
    // Sem kernel: o substituto de «um nó que a placa não corre».
    ph2d_node_motion_sort::register(&mut reg).unwrap();
    reg
}

fn edge(g: &mut Graph, a: NodeId, b: NodeId) {
    g.connect(Edge {
        from: (a, 0),
        to: (b, 0),
        delayed: false,
    })
    .unwrap();
}

fn um(g: &Graph, reg: &NodeRegistry, sink: NodeId) -> GpuPlan {
    plan_driven(g, reg, reg, sink, &DrivenParams::new())
}

fn uniao(g: &Graph, reg: &NodeRegistry, sinks: &[NodeId]) -> GpuPlan {
    plan_driven_many(g, reg, reg, sinks, &DrivenParams::new())
}

fn nos(p: &GpuPlan) -> Vec<NodeId> {
    p.stages.iter().map(|s| s.node).collect()
}

/// Topológico: toda entrada `Stage(n)` aponta para um estágio ANTERIOR, e cada nó aparece uma vez.
fn assert_topologico(p: &GpuPlan) {
    let mut vistos = BTreeSet::new();
    for s in &p.stages {
        for i in &s.inputs {
            if let GpuSource::Stage(n) = i {
                assert!(
                    vistos.contains(n),
                    "{:?} lê {n:?} antes de ele ser encenado",
                    s.node
                );
            }
        }
        assert!(vistos.insert(s.node), "{:?} encenado duas vezes", s.node);
    }
}

/// `grid → move → output`, duas vezes, sem nada em comum.
fn duas_cadeias(reg: &NodeRegistry) -> (Graph, [NodeId; 2]) {
    let mut g = Graph::new();
    let mut saidas = [NodeId(0); 2];
    for s in &mut saidas {
        let grid = g.add_node("motion.grid");
        let mv = g.add_node("motion.move");
        let out = g.add_node("motion.output");
        edge(&mut g, grid, mv);
        edge(&mut g, mv, out);
        *s = out;
    }
    g.validate(reg).expect("bem tipado");
    (g, saidas)
}

/// ⭐ **Com UMA saída, a união É o plano de sempre** — a porta nova delega-lhe, e é isso que põe
/// todos os gates de paridade da crate a guardar este caso. Inclui o sink que a placa não pode
/// encenar, que tem de continuar a ser `(sink, 0)` e nada mais.
#[test]
fn com_uma_saida_a_uniao_e_o_plano_de_sempre() {
    let reg = registry();
    let (g, [a, _]) = duas_cadeias(&reg);
    let p = uniao(&g, &reg, &[a]);
    let q = um(&g, &reg, a);
    assert_eq!(nos(&p), nos(&q));
    assert_eq!(p.stages, q.stages);
    assert_eq!(p.boundaries, q.boundaries);
    assert_eq!(p.sinks, vec![a], "o sink encenado é o último estágio");
    assert_eq!(p.stages.last().map(|s| s.node), Some(a));

    // O sink que a placa não encena: a fronteira é ele próprio, e ele NÃO entra em `sinks`.
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let srt = g.add_node("motion.sort");
    edge(&mut g, grid, srt);
    let p = uniao(&g, &reg, &[srt]);
    assert_eq!(p.boundaries, vec![(srt, 0)]);
    assert!(p.stages.is_empty() && p.sinks.is_empty());
    assert_eq!(p.boundaries, um(&g, &reg, srt).boundaries);
}

/// ⭐⭐ **Duas cadeias independentes dão a união dos dois planos**, pela ordem pedida, cada nó uma
/// vez e em ordem topológica.
#[test]
fn duas_cadeias_independentes_dao_a_uniao_dos_dois_planos() {
    let reg = registry();
    let (g, [a, b]) = duas_cadeias(&reg);
    let p = uniao(&g, &reg, &[a, b]);
    let esperado: BTreeSet<NodeId> = nos(&um(&g, &reg, a))
        .into_iter()
        .chain(nos(&um(&g, &reg, b)))
        .collect();
    let lido: BTreeSet<NodeId> = nos(&p).into_iter().collect();
    assert_eq!(lido, esperado);
    assert_eq!(p.stages.len(), 6, "três por cadeia, nenhum a mais");
    assert_eq!(p.sinks, vec![a, b], "a ordem é a que o documento pediu");
    assert!(p.is_fully_gpu());
    assert_topologico(&p);
    // E a ordem pedida manda: ao contrário, as saídas saem ao contrário.
    assert_eq!(uniao(&g, &reg, &[b, a]).sinks, vec![b, a]);
}

/// ⭐⭐⭐ **O LAÇO PARTILHADO É ENCENADO UMA VEZ** — a razão de a W2 existir.
///
/// `emitter → integrate` com o laço `integrate --pre--> wind`, e o `integrate` a alimentar DUAS
/// saídas (uma por um `move`). Cada plano de um sink contém o laço inteiro — cozinhá-los lado a
/// lado avançaria a simulação DUAS vezes por tique. A união contém-no uma vez.
#[test]
fn o_laco_partilhado_e_encenado_uma_vez() {
    let reg = registry();
    let mut g = Graph::new();
    let em = g.add_node("motion.emitter");
    let wind = g.add_node("force.wind");
    let it = g.add_node("motion.integrate");
    edge(&mut g, em, it);
    g.connect(Edge {
        from: (wind, 0),
        to: (it, 1),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (it, 0),
        to: (wind, 0),
        delayed: true,
    })
    .unwrap();
    let out_a = g.add_node("motion.output");
    let mv = g.add_node("motion.move");
    let out_b = g.add_node("motion.output");
    edge(&mut g, it, out_a);
    edge(&mut g, it, mv);
    edge(&mut g, mv, out_b);
    g.validate(&reg).expect("bem tipado");

    // O CONTROLO: cada plano de um sink tem o laço inteiro, logo lado a lado seriam DOIS.
    let conta = |p: &GpuPlan, n: NodeId| p.stages.iter().filter(|s| s.node == n).count();
    let (pa, pb) = (um(&g, &reg, out_a), um(&g, &reg, out_b));
    assert_eq!(
        conta(&pa, it) + conta(&pb, it),
        2,
        "o controlo contém o laço duas vezes"
    );
    assert!(pa.drives_a_loop() && pb.drives_a_loop());

    let p = uniao(&g, &reg, &[out_a, out_b]);
    assert_eq!(conta(&p, it), 1, "o integrador é encenado UMA vez na união");
    assert_eq!(conta(&p, wind), 1, "e a força do laço também");
    assert!(p.drives_a_loop() && p.is_fully_gpu());
    assert_eq!(p.sinks, vec![out_a, out_b]);
    assert_topologico(&p);
}

/// ⚠️ **Uma saída que a placa não encena NÃO entra em `sinks`** — ela fica como fronteira
/// `(sink, 0)`, e a outra saída continua encenada. *Pedir uma saída não é garantir que ela chega à
/// placa*: quem escolhe a rota compara `sinks` com a lista que pediu.
#[test]
fn uma_saida_que_a_placa_nao_encena_fica_fora_das_sinks() {
    let reg = registry();
    let (mut g, [a, _]) = duas_cadeias(&reg);
    let grid = g.add_node("motion.grid");
    let srt = g.add_node("motion.sort");
    edge(&mut g, grid, srt);
    let p = uniao(&g, &reg, &[a, srt]);
    assert_eq!(p.sinks, vec![a]);
    assert!(p.boundaries.contains(&(srt, 0)));
    assert_eq!(p.stages.last().map(|s| s.node), Some(a));
}

/// ⚠️ **O recuo do laço é da UNIÃO, e é declarado conservador.** Um laço na saída A e uma
/// fronteira TEMPORAL na saída B (um `sort` sem kernel sobre um `integrate`, logo não estático):
/// sozinha, A fica inteira na placa; na união o recuo proíbe o laço de A também, porque ele
/// pergunta *«há alguma fronteira não estática?»* sobre o plano inteiro — que é a mesma pergunta
/// que o plano de um sink já faz sobre todas as fronteiras dele, sem perguntar se ela alcança o
/// laço. ⚠️ Se um dia o recuo aprender alcançabilidade este gate reprova, e a premissa morre à
/// vista no diff.
#[test]
fn o_recuo_do_laco_e_da_uniao() {
    let reg = registry();
    let mut g = Graph::new();
    let laco = |g: &mut Graph| {
        let em = g.add_node("motion.emitter");
        let wind = g.add_node("force.wind");
        let it = g.add_node("motion.integrate");
        edge(g, em, it);
        g.connect(Edge {
            from: (wind, 0),
            to: (it, 1),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (it, 0),
            to: (wind, 0),
            delayed: true,
        })
        .unwrap();
        it
    };
    let it_a = laco(&mut g);
    let out_a = g.add_node("motion.output");
    edge(&mut g, it_a, out_a);
    let it_b = laco(&mut g);
    let srt = g.add_node("motion.sort");
    let out_b = g.add_node("motion.output");
    edge(&mut g, it_b, srt);
    edge(&mut g, srt, out_b);
    g.validate(&reg).expect("bem tipado");

    // O CONTROLO: sozinha, a saída A fica inteira na placa, com o laço.
    let a = um(&g, &reg, out_a);
    assert!(a.is_fully_gpu() && a.drives_a_loop());

    let p = uniao(&g, &reg, &[out_a, out_b]);
    assert!(
        !p.stages.iter().any(|s| s.node == it_a),
        "o recuo é da união: o laço de A volta à CPU porque B tem uma fronteira temporal"
    );
    // ⚠️ E o re-plano do recuo é da UNIÃO INTEIRA: as duas saídas continuam lá, cada uma com a
    // fronteira dela. (Nasceu de uma mutação SOBREVIVENTE — re-planear só a primeira saída
    // deixava a afirmação de cima verde e apagava a saída B do plano.)
    assert_eq!(p.sinks, vec![out_a, out_b]);
    assert!(p.boundaries.contains(&(it_a, 0)) && p.boundaries.contains(&(srt, 0)));
}

/// ⚠️ **A cerca do sufixo pergunta a CADA saída** — a partição de texturas de uma saída
/// desalinha-se pelo sufixo DELA, e o `suffix_changes_count` perguntava só ao último estágio.
/// Uma saída cujo sufixo não muda a contagem (um `move` com a entrada por ligar — a grelha tem
/// lei de contagem, e medi-lo por ela era o engano da 1.ª redacção) mais uma de emissor: cada uma
/// sozinha responde o seu, e a união responde «sim» pelas duas ordens.
#[test]
fn a_cerca_do_sufixo_pergunta_a_cada_saida() {
    let reg = registry();
    let mut g = Graph::new();
    let mv = g.add_node("motion.move");
    let a = g.add_node("motion.output");
    edge(&mut g, mv, a);
    let em = g.add_node("motion.emitter");
    let out_e = g.add_node("motion.output");
    edge(&mut g, em, out_e);
    g.validate(&reg).expect("bem tipado");
    // O CONTROLO: as duas respostas sozinhas são diferentes.
    assert!(!um(&g, &reg, a).suffix_changes_count(&reg));
    assert!(um(&g, &reg, out_e).suffix_changes_count(&reg));
    // A união responde «sim» seja qual for a ordem — não é o último que decide.
    assert!(uniao(&g, &reg, &[a, out_e]).suffix_changes_count(&reg));
    assert!(uniao(&g, &reg, &[out_e, a]).suffix_changes_count(&reg));
}
