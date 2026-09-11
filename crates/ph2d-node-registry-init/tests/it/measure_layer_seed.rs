//! **SONDA — o `motion.noise` sabe distinguir DUAS peças no mesmo sítio?** (doc 89, folha 06).
//!
//! A célula 24 marca `P2` em *"Separate Channels · **Use Layer as Seed**"* e a primeira metade
//! fechou em 2026-08-23. A segunda diz respeito a uma pergunta que a folha nunca mediu: o campo
//! deste nó é **espacialmente coerente** por construção (o ponto de amostragem é a posição do
//! elemento), então duas peças que ocupam o MESMO ponto lêem o MESMO número.
//!
//! ```text
//!   8 pecas empilhadas na origem  ->  o deslocamento de cada uma
//!   se todas lerem o mesmo, elas movem-se como UMA -- e nenhum knob de hoje as separa
//! ```
//!
//! ⚠️ *A justificativa escrita da célula era «dois nós com `channel` X/Y e seeds distintos»* —
//! isso separa os EIXOS, que é a metade já fechada. Ele **não** separa duas peças, porque o seed
//! é um param do nó e não uma coluna: dois nós dão dois campos, e as oito peças continuam a
//! partilhar cada um deles.
//!
//! Ela **imprime e não afirma**. Rode com
//! `cargo test -p ph2d-node-registry-init --test measure_layer_seed -- --ignored --nocapture`.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// Quantas peças empilhar.
const N: f32 = 8.0;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .expect("wire");
}

/// As posições que saem de um sink, e se a coluna `id` chegou lá.
fn positions(g: &Graph, reg: &NodeRegistry, sink: NodeId) -> (Vec<[f32; 2]>, bool) {
    let mut cook = Cook::new();
    let out = cook.cook(g, reg, sink, 0.0).expect("coza");
    let s = out[0].as_stream();
    let p = match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    };
    (p, s.get("id").is_some())
}

/// A maior distância entre duas peças do conjunto — zero significa «moveram-se como uma».
fn spread(p: &[[f32; 2]]) -> f32 {
    let mut m = 0.0f32;
    for a in p {
        for b in p {
            m = m.max(((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt());
        }
    }
    m
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn can_the_noise_tell_two_pieces_at_the_same_spot_apart() {
    let reg = registry();
    eprintln!("\n[layer seed] {N:.0} pecas EMPILHADAS na origem, contra {N:.0} espalhadas\n");
    eprintln!("  {:<44}  {:>9}  {:>4}", "arranjo", "envergad.", "id?");

    for (rotulo, gap, layer) in [
        ("espalhadas (gap = 1,0) -- o CONTROLE", 1.0, 0.0),
        ("EMPILHADAS, seed da CENA (o defeito)", 0.0, 0.0),
        ("EMPILHADAS, seed por ELEMENTO (a cura)", 0.0, 1.0),
        ("espalhadas, seed por ELEMENTO", 1.0, 1.0),
    ] {
        let mut g = Graph::new();
        let grid = g.add_node("motion.grid");
        g.set_param(grid, "rows", 1.0);
        g.set_param(grid, "cols", N);
        g.set_param(grid, "gap_x", gap);
        g.set_param(grid, "gap_y", gap);
        // ⚠️⚠️ **A PILHA TEM DE SAIR DO PONTO DE REDE.** A 1.ª versão desta sonda
        // empilhava as peças na ORIGEM, e a origem é um ponto de rede do ruído de
        // gradiente — onde ele vale **zero para todo seed, por construção**. A
        // envergadura saía `0,000000` nas duas metades, e a leitura *"elas partilham o
        // campo"* estava certa pelo motivo ERRADO: o que se media era a base do ruído,
        // não a partilha. Um deslocamento irracional-ish tira a pilha da rede.
        let off = g.add_node("motion.move");
        g.set_param(off, "dx", 0.37);
        g.set_param(off, "dy", 0.21);
        wire(&mut g, grid, 0, off, 0);
        let ns = g.add_node("motion.noise");
        g.set_param(ns, "channel", 4.0); // Position XY
        g.set_param(ns, "amplitude", 1.0);
        g.set_param(ns, "scale", 1.0);
        g.set_param(ns, "own_field", layer);
        wire(&mut g, off, 0, ns, 0);
        g.validate(&reg).expect("bem-tipado");
        let (p, has_id) = positions(&g, &reg, ns);
        eprintln!(
            "  {rotulo:<44}  {:>9.6}  {:>4}",
            spread(&p),
            if has_id { "sim" } else { "NAO" }
        );
    }

    eprintln!(
        "\n  LEITURA: a 2a linha (`0,000000` EXACTA) e' o DEFEITO -- as oito pecas leem o mesmo
  numero e movem-se como uma, e nenhum knob antigo as separava, porque o seed e' um
  PARAM do no' e nao uma coluna: dois nos dao dois campos, e as oito partilham cada um.
  A 3a e' a CURA. As linhas 1 e 4 sao o controle de que o campo continua a funcionar
  espalhado -- sem elas, `own_field` podia estar so' a injectar ruido em tudo.
  A coluna `id?` diz de onde a identidade sai quando a houver (e a queda para o INDICE
  quando nao houver e' a lei que o `sim.collide` ja' escreveu -- e ela e' o caminho
  NORMAL, porque uma `motion.grid` nao publica `id` nenhum)."
    );
}
