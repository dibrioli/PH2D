//! **O TETO DE POPULAÇÃO** (doc 89, folha 13 — o P1 `sim.spawn`), medido na cadeia REAL.
//!
//! A folha diz: *"Uma zona com spawn e sem lifetime cresce sem limite"*, e que o `motion.cull`
//! no modo Fraction **não é equivalente** — ele mantém `amount·n`, uma FRAÇÃO, então rala a
//! população inteira em vez de capar.
//!
//! ⚠️ **Quem ganha o teto é o `motion.cull`, não o `sim.spawn`, e não é escolha de gosto:** o
//! `sim.spawn` tem duas portas (`template` e `pulse`) e **não pode ver a população** — ela é o
//! estado da ZONA, e o desenho do nó é explícito sobre ele não a possuir (*"It does not merge
//! them into anything: `motion.combine` does that, and that is the whole design"*). Um `max` no
//! nascimento precisaria de uma terceira porta trazendo o estado vivo, isto é, do laço que ele
//! alimenta. **O teto é propriedade da POPULAÇÃO, então mora no nó que a vê.**
//!
//! ⚠️ **E o teto mantém os mais NOVOS**, pela lei que o `motion.emitter` — o irmão que a própria
//! folha cita — já traz escrita: *"the cap keeps the NEWEST particles: an emitter whose rate
//! outruns the cap should look like a dense young jet, not a frozen ancient cloud"*. Este
//! arquivo MEDE a premissa em vez de a assumir: numa zona o `motion.combine` apende os
//! recém-nascidos ao estado, então o prefixo é o mais VELHO.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

/// Uma zona que nasce e nunca mata, com um `motion.cull` opcional no fim do corpo.
///
/// ⚠️ A aresta `pre` vai da zona para o **PRIMEIRO** nó do corpo e a volta ao `state` é normal —
/// escrita ao contrário, o grafo compila e o mundo nunca envelhece.
fn growing_zone(cap: Option<f32>) -> (Graph, NodeId, Option<NodeId>) {
    let mut g = Graph::new();
    let seed = g.add_node("motion.grid");
    g.set_param(seed, "rows", 1.0);
    g.set_param(seed, "cols", 1.0);
    g.set_param(seed, "gap_x", 1.0);
    g.set_param(seed, "gap_y", 1.0);

    let zone = g.add_node("sim.zone");
    let merge = g.add_node("motion.combine");
    let spawn = g.add_node("sim.spawn");
    g.set_param(spawn, "rate", 12.0);
    g.set_param(spawn, "scatter", 0.0);

    let tail = cap.map(|max| {
        let c = g.add_node("motion.cull");
        g.set_param(c, "mode", 2.0); // Max Count
        g.set_param(c, "max", max);
        c
    });

    let mut wires: Vec<(NodeId, u16, NodeId, u16, bool)> = vec![
        (seed, 0, zone, 0, false),  // a semente é o init
        (seed, 0, spawn, 0, false), // e o template do nascimento
        (zone, 0, merge, 0, true),  // o `pre`: o estado do tique anterior
        (spawn, 0, merge, 1, false),
    ];
    match tail {
        Some(c) => {
            wires.push((merge, 0, c, 0, false));
            wires.push((c, 0, zone, 1, false));
        }
        None => wires.push((merge, 0, zone, 1, false)),
    }
    for (from, fp, to, tp, delayed) in wires {
        g.connect(Edge {
            from: (from, fp),
            to: (to, tp),
            delayed,
        })
        .expect("wire");
    }
    (g, zone, tail)
}

/// Corre `ticks` quadros e devolve o estado da zona a cada um.
fn population(g: &Graph, reg: &NodeRegistry, zone: NodeId, ticks: u64) -> Vec<Stream> {
    let mut cook = Cook::new();
    let mut out = Vec::new();
    for k in 0..ticks {
        let t = k as f64 / 60.0;
        out.push(
            cook.cook(g, reg, zone, t).expect("cooks")[0]
                .as_stream()
                .clone(),
        );
        cook.advance_tick(g, reg, t).expect("tick closes");
    }
    out
}

fn ids(s: &Stream) -> Vec<u32> {
    match s.get("id") {
        Some(Column::Scalar(v)) => v.iter().map(|x| *x as u32).collect(),
        _ => Vec::new(),
    }
}

/// **A queixa da folha, e o seu fecho:** sem teto a população cresce sem limite; com o teto ela
/// PLATEIA no número autorado. O controle é a primeira metade — sem ele, um cull que matasse
/// tudo passaria por "capado".
#[test]
fn a_zone_that_spawns_stops_growing_at_the_cap() {
    let reg = registry();

    let (gc, zc, _) = growing_zone(None);
    let grew = population(&gc, &reg, zc, 600);
    let unbounded = grew.last().expect("ticks").count();
    assert!(
        unbounded > 100,
        "o controle: sem teto a zona cresce sem limite, e cresceu {unbounded}"
    );

    let (g, z, _) = growing_zone(Some(20.0));
    let capped = population(&g, &reg, z, 600);
    let peak = capped.iter().map(Stream::count).max().expect("ticks");
    let last = capped.last().expect("ticks").count();
    assert_eq!(peak, 20, "a população nunca passa do teto");
    assert_eq!(
        last, 20,
        "e ENCHE o teto — um cull que matasse tudo daria 0"
    );
}

/// **Os sobreviventes são os mais NOVOS** — a premissa que este arquivo mede em vez de assumir.
/// O `sim.spawn` numera pelo relógio (ordinal crescente), então *"os mais novos"* é *"os ids
/// mais altos"*: com o teto cheio, o MENOR id da população tem de andar para frente a cada
/// nascimento. FALSIFICADO se o teto guardasse o prefixo — ali o menor id ficaria congelado no
/// primeiro nascimento para sempre, que é a nuvem antiga que o `motion.emitter` recusa.
#[test]
fn the_cap_keeps_the_young_and_the_oldest_leave_first() {
    let reg = registry();
    let (g, z, _) = growing_zone(Some(20.0));
    let frames = population(&g, &reg, z, 600);

    let full: Vec<&Stream> = frames.iter().filter(|s| s.count() == 20).collect();
    assert!(
        full.len() > 50,
        "a fixture precisa de tiques CHEIOS: {}",
        full.len()
    );

    let first = ids(full[0]);
    let last = ids(full[full.len() - 1]);
    let (lo_first, lo_last) = (
        first.iter().copied().min().expect("ids"),
        last.iter().copied().min().expect("ids"),
    );
    assert!(
        lo_last > lo_first,
        "o mais velho tem de SAIR: o menor id foi de {lo_first} para {lo_last}"
    );
    // E a metade que prova que não é o oposto: os mais novos ficam.
    let hi_last = last.iter().copied().max().expect("ids");
    assert!(
        hi_last > lo_last,
        "a população é uma janela dos ids ALTOS: {lo_last}..={hi_last}"
    );
}

/// **Uma FRAÇÃO não é um teto**, que é a frase que a folha usa para recusar a composição: com
/// `Fraction 0.5` a população não plateia — ela persegue um alvo móvel e a cada tique perde
/// metade do que ganhou, e é isso que a torna *"ralar"* em vez de *"capar"*. Este gate existe
/// para ninguém "simplificar" o modo novo de volta ao que já havia.
#[test]
fn a_fraction_is_not_a_cap() {
    let reg = registry();
    let (mut g, z, cull) = growing_zone(Some(20.0));
    // O MESMO grafo, com o cull de volta ao modo Fraction e o mesmo 0,5 de sempre.
    let cull = cull.expect("o cull do fixture");
    g.set_param(cull, "mode", 0.0);
    g.set_param(cull, "amount", 0.5);
    let frames = population(&g, &reg, z, 600);
    let counts: Vec<usize> = frames.iter().map(Stream::count).collect();
    let peak = counts.iter().copied().max().expect("ticks");
    assert!(
        peak <= 2,
        "uma fração rala a população a cada tique em vez de a capar em 20: pico {peak}"
    );
}
