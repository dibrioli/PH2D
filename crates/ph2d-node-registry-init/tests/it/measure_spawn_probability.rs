//! **A PROBABILIDADE DE NASCIMENTO JÁ É EXPRIMÍVEL?** (doc 89, folha 13 — o P1 `sim.spawn`).
//!
//! A folha marca `Spawn Probability` como **omissão P1** citando o Niagara (§C.10) e o
//! `Probability` do Particle Emitter da Cavalry, e o mecanismo que ela nomeia é sobre **uma**
//! tentativa: dirigir o `rate` por um valor aleatório não *filtra* nascimentos, **re-deriva a
//! história** (`born_in` usa o `rate` de AGORA nos dois termos do `floor`).
//!
//! ⚠️ **Mas a metodologia desta wave é TENTAR a cadeia contra o catálogo real** — foi ela que
//! rebaixou o `sim.kill_zone` de P1 para P2 (§7 da folha). E existe uma cadeia plausível que
//! ninguém tentou: os recém-nascidos saem do `sim.spawn` como um stream comum, então
//!
//! ```text
//!   sim.spawn → value.instance_field(Random) → motion.drive(Falloff) → motion.cull(Falloff)
//! ```
//!
//! deveria matar uma fração deles ANTES do `motion.combine` que os funde no estado — e um
//! nascimento morto no mesmo tique em que nasceu **é** um nascimento que não aconteceu.
//!
//! Esta sonda roda essa cadeia e conta o que ela de fato entrega. Nenhuma linha de produto
//! depende dela.
//!
//! Rodar: `cargo test -p ph2d-node-registry-init --release --test measure_spawn_probability -- --ignored --nocapture`

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

fn connect(g: &mut Graph, from: NodeId, from_port: u16, to: NodeId, to_port: u16) {
    g.connect(Edge {
        from: (from, from_port),
        to: (to, to_port),
        delayed: false,
    })
    .expect("edge");
}

/// A cadeia que a composição oferece hoje: nasce, sorteia, escreve na máscara, corta.
///
/// `keep` é o limiar do `motion.cull` no modo Falloff (mantém `falloff >= keep`), então com um
/// sorteio uniforme em `[0,1)` a fração esperada de sobreviventes é `1 - keep`.
fn composed_chain(g: &mut Graph, rate: f32, seed: f32, keep: f32) -> NodeId {
    let src = g.add_node("motion.grid");
    g.set_param(src, "rows", 1.0);
    g.set_param(src, "cols", 1.0);
    g.set_param(src, "gap_x", 1.0);
    g.set_param(src, "gap_y", 1.0);

    let spawn = g.add_node("sim.spawn");
    g.set_param(spawn, "rate", rate);
    g.set_param(spawn, "scatter", 1.0);
    g.set_param(spawn, "seed", seed);
    connect(g, src, 0, spawn, 0);

    // O único sorteio por-elemento do domínio de VALOR (mode 2 = Random).
    let draw = g.add_node("value.instance_field");
    g.set_param(draw, "mode", 2.0);
    g.set_param(draw, "seed", seed);
    connect(g, spawn, 0, draw, 0);

    // channel 5 = Falloff · mode 1 = Set.
    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", 5.0);
    g.set_param(drive, "mode", 1.0);
    g.set_param(drive, "scale", 1.0);
    connect(g, spawn, 0, drive, 0);
    connect(g, draw, 0, drive, 1);

    // mode 1 = Falloff (mantém falloff >= amount).
    let cull = g.add_node("motion.cull");
    g.set_param(cull, "mode", 1.0);
    g.set_param(cull, "amount", keep);
    g.set_param(cull, "invert", 0.0);
    connect(g, drive, 0, cull, 0);
    cull
}

/// Quantos nascimentos o `sim.spawn` DEVE a cada tique, sem filtro nenhum.
fn due_chain(g: &mut Graph, rate: f32, seed: f32) -> NodeId {
    let src = g.add_node("motion.grid");
    g.set_param(src, "rows", 1.0);
    g.set_param(src, "cols", 1.0);
    g.set_param(src, "gap_x", 1.0);
    g.set_param(src, "gap_y", 1.0);
    let spawn = g.add_node("sim.spawn");
    g.set_param(spawn, "rate", rate);
    g.set_param(spawn, "scatter", 1.0);
    g.set_param(spawn, "seed", seed);
    connect(g, src, 0, spawn, 0);
    spawn
}

/// Corre `ticks` quadros a 60 fps e devolve `(total de linhas, ids vistos)`.
fn run(g: &Graph, reg: &NodeRegistry, out: NodeId, ticks: usize) -> (usize, Vec<u32>) {
    let mut cook = Cook::new();
    let mut total = 0usize;
    let mut ids = Vec::new();
    for k in 0..ticks {
        let t = k as f64 / 60.0;
        let s: Stream = cook.cook(g, reg, out, t).expect("cooks")[0]
            .as_stream()
            .clone();
        total += s.count();
        if let Some(Column::Scalar(v)) = s.get("id") {
            ids.extend(v.iter().map(|x| *x as u32));
        }
        cook.advance_tick(g, reg, t).expect("advances");
    }
    (total, ids)
}

/// O hash do `sim.spawn` (`hash::rand01`), re-escrito aqui para a sonda poder dizer o que um
/// filtro por **ID** entregaria — o CONTROLE contra o qual a composição é medida.
fn rand01(seed: u32, id: u32, lane: u32) -> f32 {
    let mut h = seed
        .wrapping_mul(0x9e37_79b9)
        .wrapping_add(id.wrapping_mul(0x85eb_ca6b))
        .wrapping_add(lane.wrapping_mul(0xc2b2_ae35));
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb_352d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846c_a68b);
    h ^= h >> 16;
    (h >> 8) as f32 / 16_777_216.0
}

#[test]
#[ignore = "measurement probe; run with --ignored --nocapture"]
fn measure_whether_the_catalog_can_already_thin_the_births() {
    let reg = registry();
    let ticks = 600; // 10 s a 60 fps
    let keep = 0.5; // metade dos nascimentos deveria sobreviver

    println!(
        "\n=== A CADEIA COMPOSTA (spawn -> instance_field(Random) -> drive(Falloff) -> cull) ==="
    );
    println!("rate  seed   devidos   sobreviventes   fracao   ALVO 0.50");
    for &rate in &[12.0f32, 40.0] {
        for &seed in &[1.0f32, 2.0, 3.0, 7.0] {
            let mut gd = Graph::new();
            let due = due_chain(&mut gd, rate, seed);
            let (n_due, _) = run(&gd, &reg, due, ticks);

            let mut gc = Graph::new();
            let cull = composed_chain(&mut gc, rate, seed, keep);
            let (n_kept, _) = run(&gc, &reg, cull, ticks);

            let frac = if n_due == 0 {
                0.0
            } else {
                n_kept as f32 / n_due as f32
            };
            println!("{rate:4.0}  {seed:4.0}   {n_due:7}   {n_kept:13}   {frac:6.3}");
        }
    }

    // O CONTROLE: um filtro pelo ID do recém-nascido, que é o que a referência descreve.
    println!("\n=== O CONTROLE: o mesmo corte, decidido pelo ID de cada recem-nascido ===");
    println!("rate  seed   devidos   sobreviventes   fracao   ALVO 0.50");
    for &rate in &[12.0f32, 40.0] {
        for &seed in &[1.0f32, 2.0, 3.0, 7.0] {
            let mut gd = Graph::new();
            let due = due_chain(&mut gd, rate, seed);
            let (n_due, ids) = run(&gd, &reg, due, ticks);
            let kept = ids
                .iter()
                .filter(|id| rand01(seed as u32, **id, 11) < 1.0 - keep)
                .count();
            let frac = if n_due == 0 {
                0.0
            } else {
                kept as f32 / n_due as f32
            };
            println!("{rate:4.0}  {seed:4.0}   {n_due:7}   {kept:13}   {frac:6.3}");
        }
    }

    // E o MECANISMO, dito num numero: quantos nascimentos cabem num tique.
    println!("\n=== POR QUE ===");
    for &rate in &[12.0f32, 40.0] {
        let mut gd = Graph::new();
        let due = due_chain(&mut gd, rate, 1.0);
        let mut cook = Cook::new();
        let mut hist = [0usize; 4];
        for k in 0..ticks {
            let t = k as f64 / 60.0;
            let n = cook.cook(&gd, &reg, due, t).expect("cooks")[0]
                .as_stream()
                .count();
            hist[n.min(3)] += 1;
            cook.advance_tick(&gd, &reg, t).expect("advances");
        }
        println!(
            "rate {rate:4.0}: tiques com 0 nascimentos {} | com 1 {} | com 2 {} | com 3+ {}",
            hist[0], hist[1], hist[2], hist[3]
        );
        println!(
            "  => o sorteio do instance_field e' rand01(seed, INDICE-NA-LINHA) (lib.rs:117),\n     e o indice de um nascimento solto e' SEMPRE 0 ==> a mesma constante todo tique."
        );
    }
}
