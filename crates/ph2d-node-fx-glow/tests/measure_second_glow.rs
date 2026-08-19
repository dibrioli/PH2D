//! **SONDA — um SEGUNDO `fx.glow` faz alguma coisa?**
//!
//! A folha 11 marca `P1` com o defeito nomeado: *"`from_graph` faz `.find(…)` e devolve o
//! **primeiro** (`lib.rs:149`)"*, e classifica-o não como um vão de catálogo mas como a lei
//! anti-knob-morto desta casa (doc 88 — *"botão que não faz nada é pior que botão que
//! falta"*). O 2º nó **pinta, aceita clique, entra no undo e não faz nada**.
//!
//! ⚠️ **O `fx.glow` é estruturalmente diferente dos irmãos `fx.*`.** Os outros
//! (`drop_shadow`, `rgb_split`) fazem o trabalho no `eval`, por nó, e por isso **compõem**:
//! dois deles aplicam duas vezes. Este é um passe de **tela inteira** configurado por um nó,
//! lido UMA vez em `present.rs` (`ph2d_node_fx_glow::from_graph`). Medido: ele é o único
//! `fx.*` com um `from_graph`, então isto não é uma classe — é este nó.
//!
//! Ela **imprime e não afirma**. Rode com
//! `cargo test -p ph2d-node-fx-glow --test measure_second_glow -- --ignored --nocapture`.

use ph2d_nodegraph::graph::Graph;

/// Um grafo com `n` nós de glow, cada um com uma `intensity` distinta (`1, 2, 3, …`) para
/// se saber QUAL deles a leitura devolveu.
fn with_glows(n: usize) -> Graph {
    let mut g = Graph::new();
    for k in 0..n {
        let node = g.add_node(ph2d_node_fx_glow::TYPE_NAME);
        g.set_param(node, "intensity", (k + 1) as f32);
    }
    g
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn which_glow_the_screen_pass_reads() {
    eprintln!("\n[glow] qual nó o passe de tela lê");
    for n in 1..=3 {
        let g = with_glows(n);
        let read = ph2d_node_fx_glow::from_graph(&g);
        eprintln!(
            "  {n} nó(s) no grafo, intensities 1..{n}  =>  o passe lê intensity {:?}",
            read.map(|x| x.intensity)
        );
    }
    eprintln!(
        "\n  LEITURA: se o numero lido for SEMPRE 1, todo no' depois do primeiro e' inerte —
  ele pinta no grafo, aceita clique, entra no undo, e a tela nao muda."
    );
}
