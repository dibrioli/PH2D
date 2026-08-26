//! **SONDA — encadear planos já dá uma CAIXA?**
//!
//! A folha 13 marca `P2` em *"mais formas (box, segmento, SDF, forma vetorial)"* e responde
//! **PARCIAL: encadear colisores dá a união das três que existem**. ⚠️ *«União» é a palavra
//! que esta sonda vem testar* — porque encadear colisores é uma CONJUNÇÃO de respostas (cada
//! um empurra a peça para fora do seu obstáculo, um a seguir ao outro), e conjunção e união
//! não são a mesma operação.
//!
//! A diferença decide o que falta:
//!
//! ```text
//!   4 planos encadeados  =>  a peca fica DENTRO de um rectangulo   (uma CAIXA-CONTENTOR)
//!   uma caixa solida     =>  a peca fica FORA  de um rectangulo    (uma CAIXA-OBSTACULO)
//! ```
//!
//! Se o encadeamento der o contentor, então o que a célula pede de facto é o **obstáculo** —
//! e a nota *"a união das três"* está a descrever a operação errada.
//!
//! ⚠️ **E a pré-condição desta célula CAIU sem ninguém reconferir** (a própria célula o diz):
//! o `angle` do plano já existe, então a caixa ROTACIONADA deixou de estar bloqueada pelo
//! tilt. Esta sonda mede com o tilt ligado, que é a metade que ninguém tentou.
//!
//! Ela **imprime e não afirma**. Rode com
//! `cargo test -p ph2d-node-registry-init --test measure_collider_shapes -- --ignored --nocapture`.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// O índice da forma `Plane` no enum do `sim.collide`.
const SHAPE_PLANE: f32 = 0.0;
/// Meia-largura da caixa que os quatro planos delimitam.
const HALF: f32 = 2.0;
/// A grelha de sonda: 11×11 de `−5` a `5`.
const SIDE: f32 = 11.0;

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

/// Quantas peças ficam DENTRO do rectângulo `±HALF`, e quantas FORA, depois da cadeia.
fn inside_outside(reg: &NodeRegistry, angles: &[f32]) -> (usize, usize) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", SIDE);
    g.set_param(grid, "cols", SIDE);
    g.set_param(grid, "gap_x", 1.0);
    g.set_param(grid, "gap_y", 1.0);
    let mut head = grid;
    for &a in angles {
        let c = g.add_node("sim.collide");
        g.set_param(c, "shape", SHAPE_PLANE);
        g.set_param(c, "angle", a);
        // A normal aponta para o lado do MUNDO; `height` mede ao longo dela.
        g.set_param(c, "height", -HALF);
        g.set_param(c, "restitution", 0.0);
        g.set_param(c, "friction", 0.0);
        wire(&mut g, head, 0, c, 0);
        head = c;
    }
    g.validate(reg).expect("bem-tipado");
    let mut cook = Cook::new();
    let out = cook.cook(&g, reg, head, 0.0).expect("coza");
    let s = out[0].as_stream();
    let (mut inside, mut outside) = (0usize, 0usize);
    if let Some(Column::Vec2(p)) = s.get("P") {
        for q in p {
            if q[0].abs() <= HALF + 1e-3 && q[1].abs() <= HALF + 1e-3 {
                inside += 1;
            } else {
                outside += 1;
            }
        }
    }
    (inside, outside)
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn does_a_chain_of_planes_make_a_box_container_or_a_box_obstacle() {
    let reg = registry();
    eprintln!(
        "\n[caixa] uma grelha {SIDE:.0}x{SIDE:.0} contra cadeias de planos (caixa ±{HALF})\n"
    );
    eprintln!("  {:<40}  {:>7}  {:>6}", "cadeia", "dentro", "fora");
    for (rotulo, angles) in [
        ("nenhum colisor (o controlo)", &[][..]),
        ("1 plano (chao)", &[0.0][..]),
        ("2 planos (chao + tecto)", &[0.0, 180.0][..]),
        ("4 planos (caixa alinhada)", &[0.0, 180.0, 90.0, 270.0][..]),
        (
            "4 planos a 30 graus (caixa RODADA)",
            &[30.0, 210.0, 120.0, 300.0][..],
        ),
    ] {
        let (i, o) = inside_outside(&reg, angles);
        eprintln!("  {rotulo:<40}  {i:>7}  {o:>6}");
    }
    eprintln!(
        "\n  LEITURA: se a cadeia de 4 puser TODAS as {} pecas dentro, o encadeamento da' uma
  caixa-CONTENTOR -- e o que a celula pede (um obstaculo rectangular, que empurra as
  pecas para FORA) continua inexprimivel, porque encadear e' conjuncao e nao uniao.
  ⚠️ A cadeia rodada mede a metade que a pre-condicao caida desbloqueou.",
        (SIDE * SIDE) as usize
    );
}
