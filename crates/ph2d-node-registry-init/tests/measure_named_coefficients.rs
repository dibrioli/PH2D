//! **SONDA — a fórmula já lê valores NOMEADOS?** (doc 89, folha 06, célula 43).
//!
//! A célula pede *"coeficientes NOMEADOS (não `a..d` fixos)"* e responde **PARCIAL**, com uma
//! frase que é ela própria a resposta: *"um campo extra chega por COLUNA escalar do `in` (lido
//! por nome, **ilimitado**)"*.
//!
//! ⚠️ Antes de acrescentar quatro campos de texto ao painel — um nó cujo módulo já teve de
//! DOBRAR uma secção por altura —, esta sonda mede se a composição já entrega o que a célula
//! pede: um número **constante**, com o **nome do artista**, legível na fórmula.
//!
//! ```text
//!   motion.drive(Custom "speed", Set) --> motion.expression("speed * 2")
//! ```
//!
//! Ela **imprime e não afirma**. Rode com
//! `cargo test -p ph2d-node-registry-init --test measure_named_coefficients -- --ignored --nocapture`.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

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

fn values(g: &Graph, reg: &NodeRegistry, sink: NodeId) -> Vec<f32> {
    let mut cook = Cook::new();
    let out = cook.cook(g, reg, sink, 0.0).expect("coza");
    match out[0].as_stream().get("v") {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn can_a_formula_already_read_a_value_the_artist_named() {
    let reg = registry();
    eprintln!("\n[coeficientes nomeados] o que cada rota entrega a` formula\n");
    eprintln!("  {:<52}  saida", "rota");

    let build = |formula: &str, named: Option<(&str, f32)>| -> Vec<f32> {
        let mut g = Graph::new();
        let grid = g.add_node("motion.grid");
        g.set_param(grid, "rows", 1.0);
        g.set_param(grid, "cols", 4.0);
        let mut head = grid;
        if let Some((name, v)) = named {
            // O artista BATIZA uma coluna com o valor dele.
            // ⚠️ Nao ha' `value.constant` neste catalogo: a constante mais barata e' um
            // `value.map_range` com `out_lo == out_hi` (o mesmo truque da sonda do `value.gain`).
            let ramp = g.add_node("value.instance_field");
            g.set_param(ramp, "mode", 1.0); // Ramp 0..1
            wire(&mut g, head, 0, ramp, 0);
            let k = g.add_node("value.map_range");
            g.set_param(k, "in_lo", 0.0);
            g.set_param(k, "in_hi", 1.0);
            g.set_param(k, "out_lo", v);
            g.set_param(k, "out_hi", v);
            wire(&mut g, ramp, 0, k, 0);
            let dr = g.add_node("motion.drive");
            g.set_param(dr, "channel", 9.0); // Custom…
            g.set_param(dr, "mode", 1.0); // Set
            g.set_text_param(dr, "column", name);
            wire(&mut g, head, 0, dr, 0);
            wire(&mut g, k, 0, dr, 1);
            head = dr;
        }
        let ex = g.add_node("motion.expression");
        g.set_text_param(ex, "expr", formula);
        wire(&mut g, head, 0, ex, 0);
        g.validate(&reg).expect("bem-tipado");
        values(&g, &reg, ex)
    };

    let fmt = |v: Vec<f32>| {
        v.iter()
            .map(|x| format!("{x:.2}"))
            .collect::<Vec<_>>()
            .join(" ")
    };
    eprintln!(
        "  {:<52}  {}",
        "coeficiente `a` (o de hoje), a = 0",
        fmt(build("a * 2", None))
    );
    eprintln!(
        "  {:<52}  {}",
        "nome DESCONHECIDO na formula (`speed * 2`)",
        fmt(build("speed * 2", None))
    );
    eprintln!(
        "  {:<52}  {}",
        "`speed` BATIZADO por um drive, = 3",
        fmt(build("speed * 2", Some(("speed", 3.0))))
    );
    // ⚠️ O `gain` NUNCA e' batizado aqui: esta linha mede o preco de errar UM nome numa
    // formula em que o outro esta' certo -- a expressao inteira colapsa para zero, calada.
    eprintln!(
        "  {:<52}  {}",
        "`speed` certo, `gain` NUNCA batizado (o custo do typo)",
        fmt(build("speed * gain", Some(("speed", 3.0))))
    );

    eprintln!(
        "\n  LEITURA: se a 3a linha der `6,00`, a formula JA' le^ um valor com o nome que o
  artista escolheu, e a celula fecha por composicao -- sem quatro campos de texto num
  painel que ja' teve de dobrar uma seccao por altura.
  ⚠️ A 2a linha e' o CONTROLE do modo de falha: um nome desconhecido vale `0` em
  SILENCIO (`ph2d_expr`: *«a missing input is zero, not a panic»*), entao o preco de
  errar o nome e' um no' que nao faz nada sem dizer porque^."
    );
}
