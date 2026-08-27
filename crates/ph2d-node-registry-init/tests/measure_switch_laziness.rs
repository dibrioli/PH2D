//! **SONDA — o que custa o `value.switch` puxar as quatro entradas?** (doc 89, folha 15).
//!
//! A célula pede a **avaliação preguiçosa** que o Blender documenta duas vezes (*"only the
//! input that is passed through the node is computed"*) e responde **NÃO** — o cook puxa as
//! quatro. Ela classifica-se como *propriedade de escalonamento do cook, não param*.
//!
//! ⚠️ **Antes de mexer no escalonador, esta sonda mede o que a preguiça compraria** — e a
//! resposta depende de duas coisas que a célula não separa:
//!
//! ```text
//!   1. quanto custa um ramo NAO escolhido, quando ele e' caro
//!   2. o cook MEMOIZA -- um ramo partilhado com outro consumidor ja' e' pago uma vez so'
//! ```
//!
//! Ela **imprime e não afirma**. Rode com
//! `cargo test -p ph2d-node-registry-init --test measure_switch_laziness -- --ignored --nocapture`.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// Quantos elementos cada ramo processa — grande o bastante para o relógio dizer alguma coisa.
const N: f32 = 4096.0;
/// Quantas oitavas de ruído tornam um ramo CARO.
const OCTAVES: f32 = 8.0;

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

/// Um ramo CARO: ruído fractal de `OCTAVES` oitavas sobre `N` elementos, lido como valor.
fn costly_branch(g: &mut Graph, src: NodeId) -> NodeId {
    let ns = g.add_node("motion.noise");
    g.set_param(ns, "channel", 1.0);
    g.set_param(ns, "amplitude", 1.0);
    g.set_param(ns, "octaves", OCTAVES);
    wire(g, src, 0, ns, 0);
    let rd = g.add_node("value.attribute");
    g.set_text_param(rd, "attr", "P");
    wire(g, ns, 0, rd, 0);
    rd
}

/// Cozinha `sink` `reps` vezes e devolve o relógio total, em ms.
fn clock(g: &Graph, reg: &NodeRegistry, sink: NodeId, reps: u32) -> f64 {
    let t0 = std::time::Instant::now();
    for k in 0..reps {
        let mut cook = Cook::new();
        let out = cook.cook(g, reg, sink, f64::from(k) / 60.0).expect("coza");
        // consome, para o optimizador não apagar o trabalho
        if let Some(Column::Scalar(v)) = out[0].as_stream().get("v") {
            std::hint::black_box(v.first());
        }
    }
    t0.elapsed().as_secs_f64() * 1000.0 / f64::from(reps)
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn what_would_lazy_evaluation_of_the_switch_buy() {
    let reg = registry();
    eprintln!(
        "\n[preguica] o custo de UM cozimento, com ramos de {OCTAVES:.0} oitavas sobre {N:.0} pecas\n"
    );
    eprintln!("  {:<46}  {:>9}", "grafo", "ms/cook");

    let grid = |g: &mut Graph| {
        let n = g.add_node("motion.grid");
        g.set_param(n, "rows", 64.0);
        g.set_param(n, "cols", 64.0);
        n
    };

    // 1. UM ramo caro, direto — o piso.
    let mut g = Graph::new();
    let src = grid(&mut g);
    let one = costly_branch(&mut g, src);
    g.validate(&reg).expect("bem-tipado");
    let floor = clock(&g, &reg, one, 5);
    eprintln!("  {:<46}  {floor:>9.3}", "1 ramo caro, sem switch (o piso)");

    // 2. QUATRO ramos caros num switch, `select = 0`.
    let mut g = Graph::new();
    let src = grid(&mut g);
    let sw = g.add_node("value.switch");
    let sel = g.add_node("value.instance_field");
    g.set_param(sel, "mode", 0.0); // Index
    wire(&mut g, src, 0, sel, 0);
    wire(&mut g, sel, 0, sw, 0);
    for k in 0..4u16 {
        let b = costly_branch(&mut g, src);
        wire(&mut g, b, 0, sw, k + 1);
    }
    g.validate(&reg).expect("bem-tipado");
    let four = clock(&g, &reg, sw, 5);
    eprintln!("  {:<46}  {four:>9.3}", "4 ramos caros num switch");

    // 3. O MESMO ramo caro nas quatro portas — o caso da MEMOIZAÇÃO.
    let mut g = Graph::new();
    let src = grid(&mut g);
    let sw = g.add_node("value.switch");
    let sel = g.add_node("value.instance_field");
    g.set_param(sel, "mode", 0.0);
    wire(&mut g, src, 0, sel, 0);
    wire(&mut g, sel, 0, sw, 0);
    let shared = costly_branch(&mut g, src);
    for k in 0..4u16 {
        wire(&mut g, shared, 0, sw, k + 1);
    }
    g.validate(&reg).expect("bem-tipado");
    let shared_ms = clock(&g, &reg, sw, 5);
    eprintln!(
        "  {:<46}  {shared_ms:>9.3}",
        "o MESMO ramo nas 4 portas (memoizado)"
    );

    eprintln!(
        "\n  LEITURA: a preguica so' compra a diferenca entre a linha 2 e a linha 1 -- e SO'
  quando os ramos sao caros E exclusivos do switch. A linha 3 mede a outra metade: se
  ela ficar perto do piso, o cook JA' nao paga um ramo partilhado duas vezes, e nesse
  caso a preguica compra menos do que a celula sugere."
    );
}
