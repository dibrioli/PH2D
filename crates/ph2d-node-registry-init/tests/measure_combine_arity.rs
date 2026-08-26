//! **SONDA — o `motion.combine` de N entradas já é exprimível por encadeamento?**
//!
//! A folha 08 marca `P2` na *aridade*: *"4 entradas fixas vs socket multi-input — Blender
//! **Join Geometry** usa socket em forma de pílula (aceita N links)"*, e a própria coluna
//! *"exprimível?"* responde **SIM, 1 nó extra por 3 entradas**.
//!
//! ⚠️ **Mas «exprimível» e «equivalente» não são a mesma afirmação**, e é a segunda que decide
//! se a célula fecha por refutação ou por knob. O encadeamento tem duas costuras onde a
//! igualdade se poderia perder, e nenhuma delas se lê do código sem correr:
//!
//! 1. **A ORDEM da união de colunas.** O `combine` monta a lista de nomes pela ordem da
//!    primeira aparição e escolhe o *protótipo* da variante na primeira entrada que tem a
//!    coluna. Numa junção plana isso vê as 7 entradas de uma vez; encadeada, a interior já
//!    fixou o protótipo antes de a exterior ver `e`, `f`, `g`.
//! 2. **A RENUMERAÇÃO.** `reindex` reescreve `Index`/`Count` para a lista junta. Ligado na
//!    interior **e** na exterior, a interior numera 0..k−1 e a exterior por cima; ligado só
//!    na interior, a lista final sai com a numeração da metade. *Uma renumeração aplicada
//!    duas vezes é o modo de falha clássico desta família.*
//!
//! ⚠️ **E o preenchimento a ZERO é a terceira**: uma entrada sem a coluna recebe zeros. Se a
//! interior fabricar a coluna para as suas linhas, a exterior deixa de saber que `e` não a
//! tinha — o zero passa a vir de dois sítios diferentes com o mesmo valor.
//!
//! Ela **imprime e não afirma**. Rode com
//! `cargo test -p ph2d-node-registry-init --test measure_combine_arity -- --ignored --nocapture`.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::value::CookValue;

/// Quantas entradas o `motion.combine` tem hoje.
const FANIN: usize = 4;

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

/// Uma entrada de UMA linha, marcada no `y` — a marca diz de qual entrada a linha veio, e a
/// ORDEM das marcas na saída é a ordem da concatenação.
fn marked(g: &mut Graph, mark: f32) -> NodeId {
    let seed = g.add_node("motion.grid");
    g.set_param(seed, "rows", 1.0);
    g.set_param(seed, "cols", 1.0);
    let mv = g.add_node("motion.move");
    g.set_param(mv, "dy", mark);
    wire(g, seed, 0, mv, 0);
    mv
}

/// As marcas da saída, na ordem em que saíram.
fn marks(cook: &mut Cook, g: &Graph, reg: &NodeRegistry, sink: NodeId) -> Vec<String> {
    let out = cook.cook(g, reg, sink, 0.0).expect("coza");
    let CookValue::Instances(s) = &out[0] else {
        panic!("stream")
    };
    match s.get("P") {
        Some(Column::Vec2(v)) => v.iter().map(|q| format!("{:.0}", q[1])).collect(),
        _ => Vec::new(),
    }
}

/// A coluna de identidade, como texto (`—` quando ela não existe).
fn ident(cook: &mut Cook, g: &Graph, reg: &NodeRegistry, sink: NodeId, name: &str) -> String {
    let out = cook.cook(g, reg, sink, 0.0).expect("coza");
    let CookValue::Instances(s) = &out[0] else {
        panic!("stream")
    };
    match s.get(name) {
        Some(Column::Scalar(v)) => v
            .iter()
            .map(|x| format!("{x:.0}"))
            .collect::<Vec<_>>()
            .join(" "),
        _ => "—".to_string(),
    }
}

/// Liga `n` entradas marcadas a uma cadeia de `combine`s, devolvendo `(sink, quantos nós de
/// junção)`. A cadeia é *à esquerda*: a saída da anterior entra na porta `0` da seguinte.
fn chain(g: &mut Graph, n: usize, reindex_outer: bool) -> (NodeId, usize) {
    let sources: Vec<NodeId> = (0..n).map(|k| marked(g, 100.0 * (k + 1) as f32)).collect();
    let mut joins = 0usize;
    let mut acc: Option<NodeId> = None;
    let mut next = 0usize;
    while next < n {
        let j = g.add_node("motion.combine");
        joins += 1;
        let mut port = 0u16;
        if let Some(prev) = acc {
            wire(g, prev, 0, j, 0);
            port = 1;
        }
        while port < FANIN as u16 && next < n {
            wire(g, sources[next], 0, j, port);
            next += 1;
            port += 1;
        }
        acc = Some(j);
    }
    let sink = acc.expect("pelo menos uma juncao");
    // A renumeração da EXTERIOR é a que o artista liga; a da INTERIOR é a armadilha, e o
    // segundo teste constrói a cadeia à mão para poder ligar cada uma das duas.
    if reindex_outer {
        g.set_param(sink, "reindex", 1.0);
    }
    (sink, joins)
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn does_chaining_combine_reproduce_a_flat_join_of_n_inputs() {
    let reg = registry();
    eprintln!("\n[combine] N entradas por encadeamento — ordem, contagem e custo em nos\n");
    eprintln!(
        "  {:>3}  {:>5}  {:>6}  as marcas, na ordem em que sairam",
        "N", "nos", "linhas"
    );
    for n in [2usize, 4, 5, 7, 8, 10] {
        let mut g = Graph::new();
        let (sink, joins) = chain(&mut g, n, false);
        g.validate(&reg).expect("bem-tipado");
        let mut cook = Cook::new();
        let m = marks(&mut cook, &g, &reg, sink);
        let esperado: Vec<String> = (0..n).map(|k| format!("{}", 100 * (k + 1))).collect();
        let igual = if m == esperado { "==" } else { "!= ⚠️" };
        eprintln!(
            "  {n:>3}  {joins:>5}  {:>6}  {} {igual}",
            m.len(),
            m.join(" ")
        );
    }
    eprintln!(
        "\n  LEITURA: se a ordem sai `100 200 300 …` para todo N, o encadeamento reproduz a
  junção plana e a célula pede uma porta para o que a composição já dá. O custo honesto
  e' `ceil((N-1)/{})` nos de juncao.",
        FANIN - 1
    );
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn where_the_renumbering_lands_when_the_join_is_a_chain() {
    let reg = registry();
    eprintln!("\n[combine] a RENUMERACAO numa cadeia — quem escreve `Index`/`Count` por ultimo\n");
    eprintln!(
        "  {:<26}  {:>6}  Index (esperado: 0..n-1 uma vez so')",
        "reindex ligado em", "linhas"
    );
    // Sete entradas ⇒ duas junções. Construo à mão para poder ligar cada uma.
    for (rotulo, inner_on, outer_on) in [
        ("nenhuma", false, false),
        ("so' a interior", true, false),
        ("so' a exterior", false, true),
        ("as duas", true, true),
    ] {
        let mut g = Graph::new();
        let src: Vec<NodeId> = (0..7)
            .map(|k| marked(&mut g, 100.0 * (k + 1) as f32))
            .collect();
        let inner = g.add_node("motion.combine");
        for (k, &s) in src.iter().take(FANIN).enumerate() {
            wire(&mut g, s, 0, inner, k as u16);
        }
        let outer = g.add_node("motion.combine");
        wire(&mut g, inner, 0, outer, 0);
        for (k, &s) in src.iter().skip(FANIN).enumerate() {
            wire(&mut g, s, 0, outer, (k + 1) as u16);
        }
        if inner_on {
            g.set_param(inner, "reindex", 1.0);
        }
        if outer_on {
            g.set_param(outer, "reindex", 1.0);
        }
        g.validate(&reg).expect("bem-tipado");
        let mut cook = Cook::new();
        let idx = ident(&mut cook, &g, &reg, outer, "Index");
        let cnt = ident(&mut cook, &g, &reg, outer, "Count");
        let n = marks(&mut cook, &g, &reg, outer).len();
        eprintln!("  {rotulo:<26}  {n:>6}  Index [{idx}]  Count [{cnt}]");
    }
    eprintln!(
        "\n  LEITURA: se `so' a exterior` da' `0 1 2 3 4 5 6` e `as duas` da' o MESMO, a
  renumeracao e' idempotente na cadeia e nao ha' armadilha. Se `so' a interior` deixar
  a lista com a numeracao da METADE, isso e' a nota que a documentacao tem de trazer."
    );
}
