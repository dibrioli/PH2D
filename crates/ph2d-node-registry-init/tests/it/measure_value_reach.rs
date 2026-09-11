//! **SONDA — o que o domínio de VALOR alcança, em quantos nós** (doc 89, folha 15).
//!
//! Duas células que a folha marca `P2` e cuja resposta é uma CONTAGEM, não um sim/não:
//!
//! 1. **`value.curve`: índice de amostragem = idade normalizada.** A referência ranqueia-a
//!    como o 3.º maior roubo de UX do Niagara (*"curva-sobre-vida sem fios"*). A folha já
//!    tinha refutado metade da premissa (a vida **é** legível); esta sonda conta o que
//!    sobra depois de o `value.attribute` ganhar o canal **Life Fraction**.
//! 2. **`value.gain`: a faixa de operação.** O nó clampa a entrada em `[0,1]` e o
//!    doc-comment dele **declara** a fatoração `map_range → gain → map_range`. ⚠️ *Uma cerca
//!    declarada não é uma desculpa: ela é uma afirmação sobre o que a composição faz, e
//!    afirmações medem-se.*
//!
//! Ela **imprime e não afirma**. Rode com
//! `cargo test -p ph2d-node-registry-init --test measure_value_reach -- --ignored --nocapture`.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// O modo `Life Fraction` do `value.attribute` (a escada das reduções cresce para baixo).
const MODE_LIFE_FRACTION: f32 = -2.0;

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

/// Uma fila de 5 peças com `age` a subir e `life = 4`.
fn aged(g: &mut Graph) -> NodeId {
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 5.0);
    grid
}

/// O `v` que sai de um sink de valor.
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
fn how_many_nodes_does_a_curve_over_life_cost_now() {
    let reg = registry();
    eprintln!("\n[curva sobre a vida] quantos nos, e com quantos numeros digitados a` mao\n");
    eprintln!("  {:<46}  {:>4}  {:>10}", "cadeia", "nos", "a` mao");

    // A rota NOVA: um pick e a curva.
    let mut g = Graph::new();
    let grid = aged(&mut g);
    // Semeio `age` e `life` com o `motion.drive(Custom…)`, que é como um artista o faria a
    // partir de qualquer valor — aqui uma rampa, para as 5 peças terem idades distintas.
    let ramp = g.add_node("value.instance_field");
    g.set_param(ramp, "mode", 1.0); // Ramp
    wire(&mut g, grid, 0, ramp, 0);
    let set_age = g.add_node("motion.drive");
    g.set_param(set_age, "channel", 9.0); // Custom…
    g.set_param(set_age, "mode", 1.0); // Set
    g.set_param(set_age, "scale", 4.0);
    g.set_text_param(set_age, "column", "age");
    wire(&mut g, grid, 0, set_age, 0);
    wire(&mut g, ramp, 0, set_age, 1);
    // ⚠️ **A vida tem de ser CONSTANTE**, e a 1.ª versão desta fixtura deu-lhe uma rampa: com
    // `life = [0,4,8,12,16]` a fracção saiu `[0, .25, .25, .25, .25]` — que é a resposta CERTA
    // para aquela vida. *Uma fixtura que varia o denominador mede a fixtura.* Um `map_range`
    // com `out_lo == out_hi` é a constante mais barata que este catálogo tem.
    let ones = g.add_node("value.instance_field");
    g.set_param(ones, "mode", 1.0); // Ramp 0..1
    wire(&mut g, set_age, 0, ones, 0);
    let four = g.add_node("value.map_range");
    g.set_param(four, "in_lo", 0.0);
    g.set_param(four, "in_hi", 1.0);
    g.set_param(four, "out_lo", 4.0);
    g.set_param(four, "out_hi", 4.0);
    wire(&mut g, ones, 0, four, 0);
    let add_life = g.add_node("motion.drive");
    g.set_param(add_life, "channel", 9.0);
    g.set_param(add_life, "mode", 1.0); // Set
    g.set_param(add_life, "scale", 1.0);
    g.set_text_param(add_life, "column", "life");
    wire(&mut g, set_age, 0, add_life, 0);
    wire(&mut g, four, 0, add_life, 1);

    let pick = g.add_node("value.attribute");
    g.set_param(pick, "mode", MODE_LIFE_FRACTION);
    g.set_text_param(pick, "attr", "age");
    wire(&mut g, add_life, 0, pick, 0);
    g.validate(&reg).expect("bem-tipado");
    let v = values(&g, &reg, pick);
    eprintln!(
        "  {:<46}  {:>4}  {:>10}  -> {:?}",
        "attribute(Life Fraction)", 1, 0, v
    );
    eprintln!(
        "  {:<46}  {:>4}  {:>10}",
        "attribute(Age) -> map_range(0..VIDA) -> curva", 2, "1 (a vida)"
    );
    eprintln!(
        "\n  LEITURA: se a coluna da esquerda sair `0, 0.25, 0.5, 0.75, 1` sem nenhum numero
  digitado a` mao, a metade CARA da celula (a vida digitada) desapareceu, e o que sobra e'
  o DEFAULT -- uma afordancia de editor, nao uma mudanca de no'."
    );
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn does_the_declared_factorisation_of_the_gain_actually_work() {
    let reg = registry();
    eprintln!("\n[gain] a cerca DECLARADA: `map_range -> gain -> map_range` sobre `[-2, 6]`\n");
    eprintln!("  {:<40}  a saida", "cadeia");
    let mut g = Graph::new();
    let grid = aged(&mut g);
    let ramp = g.add_node("value.instance_field");
    g.set_param(ramp, "mode", 1.0); // Ramp 0..1
    wire(&mut g, grid, 0, ramp, 0);
    // `0..1` -> `-2..6`, a faixa "real" que a célula diz que o `gain` não sabe tratar.
    let up = g.add_node("value.map_range");
    g.set_param(up, "in_lo", 0.0);
    g.set_param(up, "in_hi", 1.0);
    g.set_param(up, "out_lo", -2.0);
    g.set_param(up, "out_hi", 6.0);
    wire(&mut g, ramp, 0, up, 0);
    // O `gain` CRU sobre essa faixa — o que a célula acusa.
    let raw = g.add_node("value.gain");
    g.set_param(raw, "strength", 0.8);
    wire(&mut g, up, 0, raw, 0);
    // E a cadeia declarada: de volta a `0..1`, o gain, e outra vez para fora.
    let down = g.add_node("value.map_range");
    g.set_param(down, "in_lo", -2.0);
    g.set_param(down, "in_hi", 6.0);
    g.set_param(down, "out_lo", 0.0);
    g.set_param(down, "out_hi", 1.0);
    wire(&mut g, up, 0, down, 0);
    let gain = g.add_node("value.gain");
    g.set_param(gain, "strength", 0.8);
    wire(&mut g, down, 0, gain, 0);
    let back = g.add_node("value.map_range");
    g.set_param(back, "in_lo", 0.0);
    g.set_param(back, "in_hi", 1.0);
    g.set_param(back, "out_lo", -2.0);
    g.set_param(back, "out_hi", 6.0);
    wire(&mut g, gain, 0, back, 0);
    g.validate(&reg).expect("bem-tipado");
    let fmt = |v: Vec<f32>| {
        v.iter()
            .map(|x| format!("{x:.2}"))
            .collect::<Vec<_>>()
            .join(" ")
    };
    eprintln!(
        "  {:<40}  {}",
        "a entrada (-2..6)",
        fmt(values(&g, &reg, up))
    );
    eprintln!(
        "  {:<40}  {}",
        "gain CRU sobre ela",
        fmt(values(&g, &reg, raw))
    );
    eprintln!(
        "  {:<40}  {}",
        "map_range -> gain -> map_range",
        fmt(values(&g, &reg, back))
    );
    eprintln!(
        "\n  LEITURA: se o `gain` CRU chapar a saida (tudo colado nos extremos) e a cadeia
  declarada devolver uma curva viva DENTRO de `-2..6`, a cerca esta' de pe' e a celula
  fecha por natureza. Se a cadeia tambem chapar, a cerca declara uma fatoracao que nao
  funciona -- e ai' e' defeito, nao natureza."
    );
}
