//! **ONDE O `motion.verlet_rope` DEIXA DE HONRAR OS NÚMEROS** (doc 89, folha 03 — a linha 52).
//!
//! A folha marca `length`/`gravity`/`iterations` sem `ParamHardMax` (o `count` e o `damping` já
//! têm). Os três quebram por mecanismos diferentes:
//!
//! * **`iterations` já é CLAMPADO** (`.clamp(1, 128)`, `lib.rs:380`) — a lei do `lattice` 400:
//!   acima do clamp a caixa **aceita e mente**. Não há o que medir, o teto É o clamp.
//! * **`gravity` é ACELERAÇÃO**, e o Verlet a integra contra um `dt` que o `eval` clampa em
//!   `MAX_DT = 0,1`. Uma gravidade grande move a corda mais por passo do que a relaxação de
//!   posição consegue recolher ⇒ ela ESTICA. Medível: o comprimento real contra o de repouso.
//! * **`length` é o comprimento de REPOUSO** — a corda inteira escala com ele. Se o nó for
//!   escala-invariante (como o `radius` do `motion.collide` provou ser), não há teto a escrever.
//!
//! Rodar: `cargo test -p ph2d-node-registry-init --release --test measure_rope_ceiling -- --ignored --nocapture`

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_nodegraph::value::CookValue;

/// O pior passo que o `eval` admite — um quadro perdido ou um salto de régua.
const WORST_DT: f64 = 0.1;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

/// Uma corda solta pela cabeça. ⚠️ O `pre` self-loop é escrito à MÃO (o editor o plumba ao
/// SOLTAR o nó); sem ele o estado chega vazio todo tique e a corda **nunca integra**.
fn rope(length: f32, gravity: f32, iterations: f32) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let n = g.add_node("motion.verlet_rope");
    g.set_param(n, "count", 24.0);
    g.set_param(n, "length", length);
    g.set_param(n, "gravity", gravity);
    g.set_param(n, "iterations", iterations);
    // ⚠️ A porta de estado é a **2** — as 0 e 1 são `anchor_x`/`anchor_y`. A 1ª versão desta
    // sonda ligou o self-loop na porta 1 e o `connect` **aceitou sem reclamar** (um `VALUE` a
    // receber um stream): a corda nunca recebeu estado, nunca integrou, e devolvia a pose de
    // repouso — daí `6,0000` exacto em toda a varredura, até gravidade 1e8. O índice vem do
    // MANIFESTO, nunca do palpite.
    g.connect(ph2d_nodegraph::graph::Edge {
        from: (n, 0),
        to: (n, 2),
        delayed: true,
    })
    .expect("o self-loop de estado");
    (g, n)
}

fn positions(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// Marcha e devolve (o comprimento percorrido pela corda, o pior estiramento de um segmento).
///
/// ⚠️ **O oráculo é o ESTIRAMENTO, não a posição.** Uma corda que cai muito não está errada —
/// gravidade alta faz isso. O que a relaxação promete é o comprimento de REPOUSO de cada
/// segmento, e é essa promessa que uma gravidade absurda quebra.
fn march(g: &Graph, reg: &NodeRegistry, node: NodeId, frames: u64) -> (f32, f32) {
    let mut cook = Cook::new();
    let mut last = Vec::new();
    for t in 0..frames {
        let playhead = t as f64 * WORST_DT;
        cook.advance_tick(g, reg, playhead).expect("tick");
        let out = cook.cook(g, reg, node, playhead).expect("cook");
        let CookValue::Instances(s) = &out[0] else {
            panic!("a saida da corda e um stream")
        };
        last = positions(s);
    }
    if last.len() < 2 {
        return (f32::NAN, f32::NAN);
    }
    // ⚠️ O comprimento total é o que a RESTRIÇÃO garante — usá-lo como oráculo é medir a
    // promessa, não o que a corda faz. O que responde *"esta corda está viva?"* é a QUEDA da
    // ponta; o que responde *"a promessa aguentou?"* é o pior segmento contra o repouso.
    let mut worst = 0.0f32;
    for w in last.windows(2) {
        let d = ((w[0][0] - w[1][0]).powi(2) + (w[0][1] - w[1][1]).powi(2)).sqrt();
        worst = worst.max(d);
    }
    let tail = last[last.len() - 1];
    // ⚠️ Em **f64**: o quadrado de uma coordenada de `f32` estoura em ~1e19, então uma sonda que
    // some `powi(2)` reporta `inf`/0 e o leitor atribui à CORDA um defeito que é dela própria.
    let drop = ((f64::from(tail[0])).hypot(f64::from(tail[1]))) as f32;
    (drop, worst)
}

#[test]
#[ignore = "probe: prints the measured ceilings, asserts nothing"]
fn measure_where_the_rope_stops_honouring_its_numbers() {
    let reg = registry();

    println!("\n== ITERATIONS: o kernel CLAMPA em 128 -- acima disso a caixa aceita e mente ==");
    println!(
        "{:>12}  {:>14}  {:>28}",
        "iterations", "queda da ponta", "identico ao de 128?"
    );
    let (gc, nc) = rope(6.0, 9.0, 128.0);
    let at_cap = march(&gc, &reg, nc, 60);
    for &it in &[8.0, 64.0, 128.0, 129.0, 512.0, 100_000.0] {
        let (g, n) = rope(6.0, 9.0, it);
        let (total, _) = march(&g, &reg, n, 60);
        println!(
            "{it:>12.0}  {total:>14.4}  {:>28}",
            if total.to_bits() == at_cap.0.to_bits() {
                "SIM (byte a byte)"
            } else {
                "-"
            }
        );
    }

    println!("\n== GRAVITY: a aceleracao contra o dt do PIOR caso (0,1) ==");
    println!(
        "{:>12}  {:>14}  {:>16}  {:>12}",
        "gravity", "queda da ponta", "estiramento", "honra o repouso?"
    );
    for &gr in &[
        9.0, 40.0, 1e3, 1e6, 1e12, 1e15, 1e17, 1e18, 1e19, 1e20, 1e21, 1e24,
    ] {
        let (g, n) = rope(6.0, gr, 24.0);
        let (total, worst) = march(&g, &reg, n, 60);
        // O segmento de repouso e 6/23 = 0,2609; o estiramento e a razao contra ele.
        let stretch = worst / (6.0 / 23.0);
        println!(
            "{gr:>12.3e}  {total:>14.4e}  {stretch:>16.3e}  {:>12}",
            // ⚠️ O oráculo é a corda ainda EXISTIR e ainda ser esticada pela gravidade. O
            // colapso (`queda 0`) e o não-finito são o mesmo fim: a corda deixa de estar lá.
            if total.is_finite() && total > 0.0 && stretch.is_finite() {
                "viva"
            } else {
                "MORREU"
            }
        );
    }

    println!("\n== LENGTH: a corda escala com ele? (o teste do `radius` do collide) ==");
    println!(
        "{:>14}  {:>16}  {:>16}",
        "length", "queda da ponta", "queda/repouso"
    );
    for &len in &[1.0, 6.0, 1e4, 1e8, 1e12, 1e16, 1e18, 1e19, 1e20, 1e21, 1e24] {
        let (g, n) = rope(len, 9.0, 24.0);
        let (total, _) = march(&g, &reg, n, 60);
        println!(
            "{len:>14.3e}  {total:>16.4e}  {:>16.4}  {:>10}",
            total / len,
            if total.is_finite() && total > 0.0 {
                "viva"
            } else {
                "MORREU"
            }
        );
    }
    println!();
}
