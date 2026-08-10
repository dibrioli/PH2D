//! **Quanto custa usar o `motion.orbit` como uma rotação ESTÁTICA de layout?**
//!
//! A pergunta não é acadêmica: a folha 05 do doc 89 REFUTA pôr rotação no `motion.transform`
//! justamente porque *"`motion.orbit(speed = 0)` **é** a rotação de layout em torno de um pivô"*,
//! e o doc-comment do próprio orbit abençoa o `speed = 0` como *"a static reposition"*. A
//! fatoração está certa. O que ninguém tinha medido é o **PREÇO** dela.
//!
//! O mecanismo é o `Fingerprint` do cook (`cook.rs`): um nó `Effect::Temporal` keya no
//! `playhead.to_bits()`, então **todo frame o invalida**, mesmo que a saída dele não possa ter
//! mudado. O `motion.transform` é `Effect::Pure` e é memoizado. A sonda mede os dois lado a
//! lado, sobre o MESMO layout e o MESMO número de frames.
//!
//! ⚠️ É sonda e não gate: o número mede um TRADE de arquitetura (a granularidade do memo), não
//! uma regressão — e o dia em que ele mudar é o dia em que alguém deu ao cook uma forma de
//! perguntar *"este nó é temporal NESTE instante?"*, que é uma decisão.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};
use ph2d_nodegraph::node::{NodeOp, NodeTypeId};
use std::time::Instant;

struct Reg(NodeRegistry);
impl OpResolver for Reg {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        self.0.resolve(ty)
    }
}

fn registry() -> Reg {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_grid::register(&mut reg).unwrap();
    ph2d_node_motion_transform::register(&mut reg).unwrap();
    ph2d_node_motion_orbit::register(&mut reg).unwrap();
    ph2d_node_motion_output::register(&mut reg).unwrap();
    Reg(reg)
}

/// `motion.grid` de `side²` elementos, seguido do nó que a `arm` escolher.
fn scene(reg: &Reg, side: f32, tail: &str) -> (Graph, ph2d_nodegraph::graph::NodeId) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", side);
    g.set_param(grid, "cols", side);
    g.set_param(grid, "gap_x", 0.1);
    g.set_param(grid, "gap_y", 0.1);
    let t = g.add_node(tail);
    if tail == "motion.orbit" {
        // A rotação de layout que a folha 05 declara correta: um ângulo, e NENHUM movimento.
        g.set_param(t, "angle", 30.0);
        g.set_param(t, "speed", 0.0);
    } else {
        g.set_param(t, "offset_x", 1.0);
    }
    g.connect(Edge {
        from: (grid, 0),
        to: (t, 0),
        delayed: false,
    })
    .unwrap();
    let out = g.add_node("motion.output");
    g.connect(Edge {
        from: (t, 0),
        to: (out, 0),
        delayed: false,
    })
    .unwrap();
    g.validate(&reg.0).expect("well-typed");
    (g, out)
}

#[test]
#[ignore = "sonda de diagnóstico: cargo test -p ph2d-gpu-cook --test measure_static_orbit -- --ignored --nocapture"]
fn measure_what_a_static_orbit_costs_the_memo() {
    let reg = registry();
    const FRAMES: usize = 120;
    eprintln!("  lado   elementos   transform(Pure)   orbit(Temporal)   razao");
    for &side in &[40.0f32, 100.0, 200.0, 320.0] {
        let n = (side * side) as usize;
        let mut ms = [0.0f64; 2];
        for (i, tail) in ["motion.transform", "motion.orbit"].iter().enumerate() {
            let (g, out) = scene(&reg, side, tail);
            let mut cook = Cook::new();
            // Um frame de aquecimento: o primeiro cook de QUALQUER nó é um miss, e medi-lo
            // junto misturaria o custo de nascer com o custo de re-cozinhar.
            cook.cook(&g, &reg, out, 0.0).unwrap();
            let t0 = Instant::now();
            for f in 1..=FRAMES {
                // O playhead ANDA, que é a única coisa que separa os dois casos: o layout, os
                // params e as arestas são idênticos e imóveis.
                cook.cook(&g, &reg, out, f as f64 / 60.0).unwrap();
            }
            ms[i] = t0.elapsed().as_secs_f64() * 1000.0 / FRAMES as f64;
        }
        eprintln!(
            "{side:6.0} {n:11} {:14.4} ms {:14.4} ms {:7.1}x",
            ms[0],
            ms[1],
            ms[1] / ms[0].max(1e-9)
        );
    }
    eprintln!(
        "\n  (`transform` e Pure e o memo o segura; `orbit` e Temporal e o fingerprint keya\n   no playhead, entao TODO frame o re-cozinha — com `speed = 0`, para o mesmo resultado.)"
    );
}
