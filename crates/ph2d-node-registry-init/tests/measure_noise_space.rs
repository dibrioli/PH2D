//! **SONDA — o catálogo já transforma o ESPAÇO do `motion.noise`?**
//!
//! A folha 06 linha 20 marca `P1` pedindo *offset · rotation · scale do espaço do ruído*
//! (TD Noise CHOP *Transform*; AE *Fractal Noise → Transform*; Cavalry *Noise
//! Position/Rotation/Scale*), e justifica-se com: *"offset SIM (`motion.move(+d) → noise →
//! motion.move(−d)`); rotation/scale NÃO — verificado: `motion.rotate` escreve a coluna
//! `rot`, não gira `P`"*.
//!
//! ⚠️ **A frase sobre o `motion.rotate` é VERDADE e não responde à pergunta.** Desde que a
//! célula foi escrita o catálogo tem dois nós que mexem em `P` e que ela não cita:
//!
//! - **`motion.orbit`** (`speed = 0`) — *"rotates each instance's position `P` around a
//!   pivot"*, e o doc dele diz com todas as letras que ele **É** a rotação de layout da
//!   família inteira;
//! - **`motion.transform`** — escala `P` sobre um pivô (origem / ponto / centróide).
//!
//! Então o **sanduíche** que já dá o offset tem, à partida, um irmão para cada eixo do
//! pedido. Esta sonda mede as quatro rotas e **imprime**, não afirma:
//!
//! 1. **CONTROLE** — `grid → noise`.
//! 2. **OFFSET** — `move(+d) → noise → move(−d)` (o que a célula já dá por bom).
//! 3. **ROTAÇÃO** — `orbit(+θ) → noise → orbit(−θ)`.
//! 4. **ESCALA** — `transform(s) → noise → transform(1/s)`.
//!
//! ⚠️ **O que se mede não é só *"o padrão mudou"*.** O `motion.noise` soma um **delta escalar
//! a um canal** (aqui o Y), e o segundo nó do sanduíche age sobre `P` **depois** de o delta lá
//! estar. Uma translação comuta com isso; uma rotação e uma escala **não**. Então a sonda
//! pergunta as duas coisas de cada rota:
//!
//! - **o X voltou?** (se não, o sanduíche não devolveu a pose — é um efeito, não um espaço);
//! - **o delta sobreviveu inteiro?** (comparado com o mesmo campo amostrado no mesmo sítio).
//!
//! Rode com
//! `cargo test -p ph2d-node-registry-init --test measure_noise_space -- --ignored --nocapture`.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::value::CookValue;

/// Peças na fileira. Ímpar, e o suficiente para o campo variar ao longo dela.
const N: usize = 9;
/// O vão entre peças, em unidades de mundo.
const GAP: f32 = 0.5;
/// O deslocamento do sanduíche de OFFSET.
const SHIFT: f32 = 1.7;
/// O ângulo do sanduíche de ROTAÇÃO, em graus.
const TURN: f32 = 90.0;
/// O fator do sanduíche de ESCALA.
const ZOOM: f32 = 2.0;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

fn wire(g: &mut Graph, from: NodeId, to: NodeId) {
    g.connect(Edge {
        from: (from, 0),
        to: (to, 0),
        delayed: false,
    })
    .expect("wire");
}

fn row(g: &mut Graph) -> NodeId {
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", N as f32);
    g.set_param(grid, "gap_x", GAP);
    grid
}

/// O ruído desta sonda: **estático** (`speed = 0`, então o relógio não entra), uma oitava,
/// e uma escala espacial que faz o campo variar visivelmente ao longo da fileira.
fn noise(g: &mut Graph, src: NodeId) -> NodeId {
    let n = g.add_node("motion.noise");
    g.set_param(n, "channel", 1.0); // Y
    g.set_param(n, "amplitude", 1.0);
    g.set_param(n, "scale", 0.6);
    g.set_param(n, "octaves", 1.0);
    g.set_param(n, "speed", 0.0);
    wire(g, src, n);
    n
}

fn positions(g: &Graph, reg: &NodeRegistry, sink: NodeId) -> Vec<[f32; 2]> {
    let mut cook = Cook::new();
    let out = cook.cook(g, reg, sink, 0.0).expect("coza");
    let CookValue::Instances(s) = &out[0] else {
        panic!("stream")
    };
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// `(pior |Δx| contra o controle, pior |Δy| contra o controle)`.
fn against(base: &[[f32; 2]], other: &[[f32; 2]]) -> (f32, f32) {
    base.iter()
        .zip(other)
        .fold((0.0f32, 0.0f32), |(dx, dy), (a, b)| {
            (dx.max((a[0] - b[0]).abs()), dy.max((a[1] - b[1]).abs()))
        })
}

fn show(tag: &str, p: &[[f32; 2]], base: &[[f32; 2]]) {
    let (dx, dy) = against(base, p);
    eprintln!(
        "  {tag:<34} |Δx| {dx:.6}  |Δy| {dy:.6}   y: {}",
        p.iter()
            .take(5)
            .map(|q| format!("{:+.3}", q[1]))
            .collect::<Vec<_>>()
            .join(" ")
    );
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn what_the_catalogue_can_already_do_to_the_noise_space() {
    let reg = registry();
    eprintln!("\n[noise-space] o que o catalogo ja faz com o ESPACO do ruido");
    eprintln!("  (fileira de {N} pecas, gap {GAP}, ruido estatico)\n");

    // 1 — CONTROLE.
    let mut g = Graph::new();
    let src = row(&mut g);
    let ctrl_node = noise(&mut g, src);
    let ctrl = positions(&g, &reg, ctrl_node);
    show("1 CONTROLE  grid -> noise", &ctrl, &ctrl);

    // 2 — OFFSET: move(+d) -> noise -> move(-d).
    let mut g = Graph::new();
    let src = row(&mut g);
    let m1 = g.add_node("motion.move");
    g.set_param(m1, "dx", SHIFT);
    wire(&mut g, src, m1);
    let ns = noise(&mut g, m1);
    let m2 = g.add_node("motion.move");
    g.set_param(m2, "dx", -SHIFT);
    wire(&mut g, ns, m2);
    show(
        "2 OFFSET    move(+d)..move(-d)",
        &positions(&g, &reg, m2),
        &ctrl,
    );

    // 3 — ROTACAO: orbit(+t) -> noise -> orbit(-t).  ⚠️ `speed` default e' 72 graus/s.
    let mut g = Graph::new();
    let src = row(&mut g);
    let o1 = g.add_node("motion.orbit");
    g.set_param(o1, "angle", TURN);
    g.set_param(o1, "speed", 0.0);
    wire(&mut g, src, o1);
    let ns = noise(&mut g, o1);
    let o2 = g.add_node("motion.orbit");
    g.set_param(o2, "angle", -TURN);
    g.set_param(o2, "speed", 0.0);
    wire(&mut g, ns, o2);
    show(
        "3 ROTACAO   orbit(+t)..orbit(-t)",
        &positions(&g, &reg, o2),
        &ctrl,
    );

    // 4 — ESCALA: transform(s) -> noise -> transform(1/s).
    let mut g = Graph::new();
    let src = row(&mut g);
    let t1 = g.add_node("motion.transform");
    g.set_param(t1, "scale", ZOOM);
    wire(&mut g, src, t1);
    let ns = noise(&mut g, t1);
    let t2 = g.add_node("motion.transform");
    g.set_param(t2, "scale", 1.0 / ZOOM);
    wire(&mut g, ns, t2);
    show(
        "4 ESCALA    transform(s)..(1/s)",
        &positions(&g, &reg, t2),
        &ctrl,
    );

    let zoomed = positions(&g, &reg, t2);

    // 5 — O PARAM DO PRÓPRIO NÓ: `scale` já é a frequência espacial, ou seja o ZOOM do
    // espaço. Amostrar `P·(0,6·2)` é a MESMA coisa que amostrar `(2P)·0,6`.
    let mut g = Graph::new();
    let src = row(&mut g);
    let ns = noise(&mut g, src);
    g.set_param(ns, "scale", 0.6 * ZOOM);
    let own = positions(&g, &reg, ns);
    show("5 O PARAM   noise(scale = 0.6*s)", &own, &ctrl);

    // A comparação decisiva: a rota 4 é a rota 5 com o delta DIVIDIDO por `s`?
    let worst = own
        .iter()
        .zip(&zoomed)
        .fold(0.0f32, |m, (a, b)| m.max((a[1] / ZOOM - b[1]).abs()));
    eprintln!(
        "\n  rota 4 contra rota 5/{ZOOM}: pior |Δy| = {worst:.6}
  (se ~0, o sanduíche de ESCALA é o param `scale` do nó, com a amplitude dividida por s)"
    );

    eprintln!(
        "\n  LEITURA: |Δx| ~0 = o sanduiche devolveu a POSE (e um espaco, nao um efeito).
  |Δy| > 0 = o campo foi amostrado noutro sitio (o espaco de facto mudou).
  |Δx| > 0 = o segundo no' mexeu no DELTA que o ruido acabou de somar."
    );
}
