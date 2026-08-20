//! **SONDA — o *Pick Instances* do Blender já é exprimível?**
//!
//! A folha 14 marca `P1` no `source.object`: *"escolher UM de vários objetos por índice /
//! aleatório — Blender `Instance on Points → Pick Instances`"*, e nota que o próprio repo já o
//! lista como faltante.
//!
//! ⚠️ **Mas a folha 08 fechou um `P0` que é exactamente esse mecanismo, noutro nó**: o
//! `motion.duplicator` tem um param `pick` com **Off / Cycle / Random**, e o doc dele diz
//! *"`pick` escolhe qual forma pousa num ponto"*. Se ele aceita um stream de VÁRIAS formas na
//! porta `shape`, então N objetos combinados + `pick = Cycle` é o Pick Instances — e a célula
//! do `source.object` está a pedir uma segunda porta para o que a composição já dá.
//!
//! ⚠️ **A sonda usa `motion.grid`+`motion.move` como "formas", não `source.shape`**: aquele nó
//! lê um EXTERNAL que só o SHELL publica, então fora do app ele devolve um stream vazio — a
//! sonda mediria o shell ausente. O mecanismo do `pick` não sabe o que é uma aparência; ele
//! empareja linhas.
//!
//! Ela **imprime e não afirma**. Rode com
//! `cargo test -p ph2d-node-registry-init --test measure_pick_instances -- --ignored --nocapture`.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::value::CookValue;

/// Quantos pontos a grelha tem.
const POINTS: f32 = 6.0;

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

/// Uma "forma" de UMA linha, marcada por um `dy` distinto — o duplicator SOMA o `P` do ponto
/// ao `P` da forma, então o `y` da saída diz QUAL forma pousou ali.
fn marked_shape(g: &mut Graph, mark: f32) -> NodeId {
    let seed = g.add_node("motion.grid");
    g.set_param(seed, "rows", 1.0);
    g.set_param(seed, "cols", 1.0);
    let mv = g.add_node("motion.move");
    g.set_param(mv, "dy", mark);
    wire(g, seed, 0, mv, 0);
    mv
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn does_the_duplicator_already_deal_several_shapes_around_the_points() {
    let reg = registry();
    eprintln!("\n[pick] tres 'formas' (marcas 100/200/300) sobre uma fileira de {POINTS} pontos\n");
    eprintln!(
        "  {:>8}  {:>8}  a marca que pousou em cada ponto",
        "pick", "saidas"
    );
    // 0 Off · 1 Cycle · 2 Random (a ordem do param; ver o doc do `motion.duplicator`).
    for pick in [0.0f32, 1.0, 2.0] {
        let mut g = Graph::new();
        let shapes: Vec<NodeId> = [100.0f32, 200.0, 300.0]
            .into_iter()
            .map(|m| marked_shape(&mut g, m))
            .collect();
        let merge = g.add_node("motion.combine");
        for (k, &s) in shapes.iter().enumerate() {
            wire(&mut g, s, 0, merge, k as u16);
        }
        let row = g.add_node("motion.grid");
        g.set_param(row, "rows", 1.0);
        g.set_param(row, "cols", POINTS);
        g.set_param(row, "gap_x", 1.0);

        let dup = g.add_node("motion.duplicator");
        g.set_param(dup, "pick", pick);
        wire(&mut g, merge, 0, dup, 0); // as formas
        wire(&mut g, row, 0, dup, 1); // os pontos
        g.validate(&reg).expect("bem-tipado");

        let mut cook = Cook::new();
        let out = cook.cook(&g, &reg, dup, 0.0).expect("coza");
        let CookValue::Instances(s) = &out[0] else {
            panic!("stream")
        };
        let marks: Vec<String> = match s.get("P") {
            Some(Column::Vec2(v)) => v.iter().map(|q| format!("{:.0}", q[1])).collect(),
            _ => Vec::new(),
        };
        eprintln!("  {pick:>8.0}  {:>8}  {}", marks.len(), marks.join(" "));
    }
    eprintln!(
        "\n  LEITURA: `Off` = produto cartesiano (3 formas x {POINTS} pontos = 18 saidas). Se
  `Cycle` der {POINTS} saidas com as marcas a alternar 100/200/300, o Pick Instances
  ja' existe e a celula do `source.object` pede uma porta para o que a composicao da'."
    );
}

/// **O `Random` distribui, ou ele encosta numa forma?** A primeira leitura da sonda acima deu
/// `100 100 100 100 200 100` em seis pontos — cinco de seis na mesma forma. Com seis amostras
/// isso pode ser só o acaso; com 600 já não é.
#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn how_evenly_the_random_pick_deals_the_shapes() {
    let reg = registry();
    eprintln!("\n[pick] a distribuicao do `Random` — quantos pontos caem em cada forma\n");
    eprintln!(
        "  {:>7}  {:>7}  contagem por forma (esperado: parelho)",
        "formas", "pontos"
    );
    // ⚠️ **Para em 4, e o teto é do `motion.combine`** (`in0..in3`): cinco variantes precisam
    // de encadear duas junções. É o custo honesto de compor isto — 1 nó a cada 3 variantes a
    // mais —, e a 1ª versão desta sonda descobriu-o levando um `BadInputPort`.
    for shapes_n in [2usize, 3, 4] {
        for points in [60.0f32, 600.0] {
            let mut g = Graph::new();
            let marks: Vec<f32> = (0..shapes_n).map(|k| 100.0 * (k + 1) as f32).collect();
            let nodes: Vec<NodeId> = marks.iter().map(|&m| marked_shape(&mut g, m)).collect();
            let merge = g.add_node("motion.combine");
            for (k, &s) in nodes.iter().enumerate() {
                wire(&mut g, s, 0, merge, k as u16);
            }
            let row = g.add_node("motion.grid");
            g.set_param(row, "rows", 1.0);
            g.set_param(row, "cols", points);
            g.set_param(row, "gap_x", 0.1);
            let dup = g.add_node("motion.duplicator");
            g.set_param(dup, "pick", 2.0); // Random
            wire(&mut g, merge, 0, dup, 0);
            wire(&mut g, row, 0, dup, 1);
            g.validate(&reg).expect("bem-tipado");

            let mut cook = Cook::new();
            let out = cook.cook(&g, &reg, dup, 0.0).expect("coza");
            let CookValue::Instances(st) = &out[0] else {
                panic!("stream")
            };
            let mut hist = vec![0usize; shapes_n];
            if let Some(Column::Vec2(v)) = st.get("P") {
                for q in v {
                    // O `y` traz a marca mais o `y` do ponto (que é 0 nesta fileira).
                    let k = (q[1] / 100.0).round() as usize;
                    if (1..=shapes_n).contains(&k) {
                        hist[k - 1] += 1;
                    }
                }
            }
            eprintln!("  {shapes_n:>7}  {points:>7.0}  {hist:?}");
        }
    }
    eprintln!(
        "\n  LEITURA: com N formas e P pontos, o parelho e' P/N. ⚠️ COMPARE COM O DESVIO
  PADRAO, nao com o olho: para P = 600 ele e' ~12 num corte de 2 vias, ~11,5 em 3
  e ~10,6 em 4. Medido em 2026-08-19: [320,280] · [219,194,187] · [171,149,145,135]
  -- o primeiro balde e' o maior nos tres, mas cada um esta' a 1,6-2,0 sigma, e os
  tres partilham a MESMA sequencia de hashes (nao sao amostras independentes).
  Isso NAO sustenta um defeito; um hash torto de verdade poe o primeiro balde a
  varios sigma e a distancia CRESCE com P."
    );
}
