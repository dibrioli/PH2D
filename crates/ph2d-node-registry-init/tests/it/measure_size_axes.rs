//! **SONDA — a composição já dá `Size X ≠ Size Y` por elemento?**
//!
//! A folha 06 linha 39 marca `P1` no `motion.drive` pedindo o *Scale + S.XYZ* do C4D, e
//! justifica-se com *"o braço Size escreve `si[0]` **e** `si[1]` com o mesmo `v`; dois
//! drives também escrevem os dois"*. Isso é sobre o NÓ — a pergunta da conferência é
//! sobre o **catálogo**, e desde que a célula foi escrita o `motion.scale` já shipa
//! `uniform`/`amount`/`amount_y` e o Grupo P deu ao drive o canal `Custom…`.
//!
//! Três rotas medidas antes de escrever knob nenhum:
//!
//! 1. **CONTROLE** — o `Size` de hoje: os dois eixos saem iguais?
//! 2. **`drive(Size) → motion.scale(não-uniforme)`** — anisotropia FIXA sobre uma
//!    magnitude DIRIGIDA. Se isto medir, metade do pedido já existe.
//! 3. **DOIS campos independentes**, um por eixo — o `Custom…` consegue escrever
//!    `size.x` sem levar o `y` junto?
//!
//! Ela **imprime e não afirma**. Rode com
//! `cargo test -p ph2d-node-registry-init --test measure_size_axes -- --ignored --nocapture`.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::value::CookValue;

const N: usize = 8;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

fn wire(g: &mut Graph, from: NodeId, to: NodeId, port: u16) {
    g.connect(Edge {
        from: (from, 0),
        to: (to, port),
        delayed: false,
    })
    .expect("wire");
}

/// Uma fileira de `N` peças com um campo de valor por-índice a montante.
fn row(g: &mut Graph) -> NodeId {
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", N as f32);
    g.set_param(grid, "gap_x", 1.0);
    grid
}

/// Uma rampa `0..1` por índice (o `Ramp` do `value.instance_field`).
fn ramp(g: &mut Graph, src: NodeId, seed: f32) -> NodeId {
    let f = g.add_node("value.instance_field");
    g.set_param(f, "mode", 1.0); // Ramp
    g.set_param(f, "seed", seed);
    wire(g, src, f, 0);
    f
}

fn sizes(g: &Graph, reg: &NodeRegistry, sink: NodeId) -> Vec<[f32; 2]> {
    let mut cook = Cook::new();
    let out = cook.cook(g, reg, sink, 0.0).expect("coza");
    let CookValue::Instances(s) = &out[0] else {
        panic!("stream")
    };
    match s.get("size") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

fn print_sizes(tag: &str, v: &[[f32; 2]]) {
    let worst = v.iter().fold(0.0f32, |m, s| m.max((s[0] - s[1]).abs()));
    eprintln!(
        "  {tag:<26} pior |x−y| {worst:.6}  {}",
        v.iter()
            .take(4)
            .map(|s| format!("({:.3},{:.3})", s[0], s[1]))
            .collect::<Vec<_>>()
            .join(" ")
    );
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn what_the_catalogue_can_already_do_to_the_two_size_axes() {
    let reg = registry();
    eprintln!("\n[size] o que o catalogo ja faz com os DOIS eixos");

    // 1. CONTROLE — o canal Size de hoje.
    let mut g = Graph::new();
    let src = row(&mut g);
    let f = ramp(&mut g, src, 0.0);
    let dr = g.add_node("motion.drive");
    g.set_param(dr, "channel", 3.0); // Size
    g.set_param(dr, "mode", 1.0); // Set
    g.set_param(dr, "scale", 1.0);
    wire(&mut g, src, dr, 0);
    wire(&mut g, f, dr, 1);
    print_sizes("1 controle: drive(Size)", &sizes(&g, &reg, dr));

    // 2. drive(Size) -> motion.scale nao-uniforme.
    let sc = g.add_node("motion.scale");
    g.set_param(sc, "uniform", 0.0);
    g.set_param(sc, "amount", 2.0);
    g.set_param(sc, "amount_y", 0.5);
    wire(&mut g, dr, sc, 0);
    print_sizes("2 + scale(2,0 / 0,5)", &sizes(&g, &reg, sc));

    // 3. DOIS campos independentes, um por eixo, pelo canal Custom.
    let mut g3 = Graph::new();
    let src3 = row(&mut g3);
    let fa = ramp(&mut g3, src3, 0.0);
    let fb = ramp(&mut g3, src3, 7.0);
    let d1 = g3.add_node("motion.drive");
    g3.set_param(d1, "channel", 3.0); // Size, os dois eixos
    g3.set_param(d1, "mode", 1.0);
    wire(&mut g3, src3, d1, 0);
    wire(&mut g3, fa, d1, 1);
    let d2 = g3.add_node("motion.drive");
    g3.set_param(d2, "channel", 9.0); // Custom...
    g3.set_param(d2, "mode", 1.0);
    g3.set_text_param(d2, "column", "size");
    wire(&mut g3, d1, d2, 0);
    wire(&mut g3, fb, d2, 1);
    print_sizes("3 + drive(Custom \"size\")", &sizes(&g3, &reg, d2));

    eprintln!("  => se a rota 3 nao separa os eixos, o vao e' REAL e e' o par de canais.");
}
