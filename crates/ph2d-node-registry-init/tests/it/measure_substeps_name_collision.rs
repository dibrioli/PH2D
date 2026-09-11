//! **SONDA — a palavra `substeps` quer dizer DUAS coisas, e o motor lê as duas?**
//!
//! A convenção que o substep por-relógio estabeleceu (folha 13, 2026-08-12) é de **manifesto**:
//! `ph2d_nodegraph::cook::substep_islands` varre o grafo, e *todo* nó cujo manifesto declara um
//! param chamado `substeps` vira **declarante** — o sequenciador passa a rodar `n` sub-passadas do
//! cone dele por tique.
//!
//! Quatro dias depois (folha 03, 2026-08-16) o `motion.verlet_rope` ganhou um param com o **mesmo
//! nome** e um significado **diferente**: um laço `for` DENTRO do `eval` dele, que re-integra a
//! corda `n` vezes com um `dt` menor e re-escala o `prev_local` para a inércia atravessar a
//! fronteira do tique uma vez só.
//!
//! Os dois são legítimos e nenhum sabe do outro. Esta sonda mede se eles se **compõem** — isto é,
//! se pôr uma corda a `substeps = 8` num grafo faz o sequenciador rodá-la 8 vezes, cada uma delas
//! a fazer 8 sub-passos internos. Se sim, a corda integra **64** vezes onde o artista pediu 8, e a
//! re-escala de inércia é calculada para a fatia errada.
//!
//! ⚠️ **A viagem é cega a isto** — as duas rotas produzem uma corda que cai e mantém o
//! comprimento; o que muda é *quanto* ela caiu. O oráculo tem de ser numérico.
//!
//! Ela **imprime e não afirma**. Rode com
//! `cargo test -p ph2d-node-registry-init --test measure_substeps_name_collision -- --ignored --nocapture`.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, graph_substeps, substep_islands};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// 60 fps, o passo do pump real.
const DT: f64 = 1.0 / 60.0;
/// Quadros marchados — o bastante para a cauda de uma corda solta se abrir.
const FRAMES: u64 = 60;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

/// Uma corda solta pela cabeça, com o self-loop de estado na porta **2** (as 0/1 são os âncoras).
fn rope(substeps: f32) -> (Graph, NodeId) {
    rope_damped(substeps, None)
}

/// A mesma corda, com o `damping` escrito à mão — a composição que a folha 03 mediu e **não
/// curou de propósito** é `substeps × damping`, então uma prova que só corre no default não a
/// contém.
fn rope_damped(substeps: f32, damping: Option<f32>) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let n = g.add_node("motion.verlet_rope");
    g.set_param(n, "count", 24.0);
    g.set_param(n, "length", 6.0);
    g.set_param(n, "gravity", 98.0);
    g.set_param(n, "iterations", 24.0);
    g.set_param(n, "substeps", substeps);
    if let Some(d) = damping {
        g.set_param(n, "damping", d);
    }
    g.connect(Edge {
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

/// Marcha `FRAMES` quadros. `via_pump` liga o achador de ilhas — exactamente o que o
/// `MotionCookPump::substep_declared_zones` faz antes do cook de cada quadro.
fn march(g: &Graph, reg: &NodeRegistry, node: NodeId, via_pump: bool) -> Vec<[f32; 2]> {
    let mut cook = Cook::new();
    let mut last = Vec::new();
    for k in 0..FRAMES {
        let t = (k + 1) as f64 * DT;
        if via_pump && let Some(frame_start) = cook.prev_playhead() {
            for island in substep_islands(g, reg) {
                cook.substep(g, reg, island.root, frame_start, t, island.substeps)
                    .expect("substep");
            }
        }
        last = positions(cook.cook(g, reg, node, t).expect("coza")[0].as_stream());
        cook.advance_tick(g, reg, t).expect("tick");
    }
    last
}

/// Marcha pedindo `n` sub-passadas ao RELÓGIO, seja qual for o que o nó declara — para comparar
/// as duas leis lado a lado sobre a MESMA corda.
fn march_forced(g: &Graph, reg: &NodeRegistry, node: NodeId, n: u32) -> Vec<[f32; 2]> {
    let mut cook = Cook::new();
    let mut last = Vec::new();
    for k in 0..FRAMES {
        let t = (k + 1) as f64 * DT;
        if let Some(frame_start) = cook.prev_playhead() {
            cook.substep(g, reg, node, frame_start, t, n).expect("sub");
        }
        last = positions(cook.cook(g, reg, node, t).expect("coza")[0].as_stream());
        cook.advance_tick(g, reg, t).expect("tick");
    }
    last
}

/// A cauda (o último ponto) e o pior estiramento de um segmento contra o repouso.
fn read(p: &[[f32; 2]]) -> (f32, f32) {
    let rest = 6.0 / (p.len().max(2) - 1) as f32;
    let worst = p
        .windows(2)
        .map(|w| {
            let (dx, dy) = (w[1][0] - w[0][0], w[1][1] - w[0][1]);
            (dx * dx + dy * dy).sqrt() / rest
        })
        .fold(0.0f32, f32::max);
    (p.last().map_or(f32::NAN, |q| q[1]), worst)
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn does_the_ropes_own_substeps_also_drive_the_clock() {
    let reg = registry();
    eprintln!("\n[substeps-collision] a corda a 24 pontos, gravidade 98, 1 s a 60 fps\n");
    eprintln!(
        "  {:>9}  {:>7}  {:>7}  {:>11}  {:>11}",
        "substeps", "ilhas", "ritmo", "cauda Y", "pior stretch"
    );
    for sub in [1.0f32, 2.0, 8.0] {
        let (g, node) = rope(sub);
        g.validate(&reg).expect("bem-tipado");
        let islands = substep_islands(&g, &reg).len();
        let rate = graph_substeps(&g, &reg);
        for (tag, via) in [("sem pump", false), ("com pump", true)] {
            let (tail, stretch) = read(&march(&g, &reg, node, via));
            eprintln!(
                "  {sub:>9.0}  {islands:>7}  {rate:>7}  {tail:>11.6}  {stretch:>11.6}   {tag}"
            );
        }
    }

    eprintln!(
        "\n  LEITURA: se as duas linhas de cada `substeps` forem IGUAIS, a corda nao e'
  declarante e os dois sentidos da palavra nao se cruzam. Se DIFERIREM para
  `substeps > 1` e nao para `= 1`, o sequenciador esta' a rodar o laco interno
  dela `n` vezes — `n x n` integracoes onde o artista pediu `n`."
    );
}

/// **A pergunta que decide a CURA:** as duas leis dizem a mesma coisa?
///
/// - **LOCAL** — a corda a `substeps = n`, sem relógio nenhum a subdividir (o laço interno dela).
/// - **RELÓGIO** — a corda a `substeps = 1`, com `n` sub-passadas pedidas ao `Cook::substep`.
///
/// Se convergirem para o mesmo lugar, há UMA lei e o laço interno é o que sobra. Se não, são duas
/// leis diferentes e a palavra tem mesmo de deixar de ser ambígua.
#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn are_the_two_laws_the_same_law() {
    let reg = registry();
    eprintln!("\n[substeps-collision] as duas leis, lado a lado, sobre a MESMA corda\n");
    eprintln!(
        "  {:>8}  {:>4}  {:>13}  {:>13}  {:>13}",
        "damping", "n", "LOCAL cauda", "RELOGIO cauda", "diferenca"
    );
    for damping in [None, Some(0.3f32)] {
        for n in [1u32, 2, 4, 8, 16] {
            let (g_local, node_l) = rope_damped(n as f32, damping);
            g_local.validate(&reg).expect("bem-tipado");
            let local = read(&march_forced(&g_local, &reg, node_l, 1)).0;

            let (g_clock, node_c) = rope_damped(1.0, damping);
            g_clock.validate(&reg).expect("bem-tipado");
            let clock = read(&march_forced(&g_clock, &reg, node_c, n)).0;

            let tag = damping.map_or("default".into(), |d| format!("{d:.2}"));
            eprintln!(
                "  {tag:>8}  {n:>4}  {local:>13.6}  {clock:>13.6}  {:>13.6}",
                (local - clock).abs()
            );
        }
    }
    eprintln!(
        "\n  LEITURA: se as duas colunas convergirem para o mesmo numero, ha' UMA lei e o
  laco interno da corda e' redundante com o relogio. Se nao, sao duas leis."
    );
}

/// Uma zona que CAI sob aceleração constante — o VIZINHO, que nada tem com a corda.
fn falling_zone(g: &mut Graph) -> NodeId {
    let seed = g.add_node("motion.grid");
    g.set_param(seed, "rows", 1.0);
    g.set_param(seed, "cols", 1.0);
    let zone = g.add_node("sim.zone");
    let wind = g.add_node("force.wind");
    g.set_param(wind, "angle", 0.0);
    g.set_param(wind, "strength", 40.0);
    g.set_param(wind, "gust", 0.0);
    let step = g.add_node("sim.step");
    g.set_param(step, "damping", 1.0);
    g.connect(Edge {
        from: (seed, 0),
        to: (zone, 0),
        delayed: false,
    })
    .expect("w");
    g.connect(Edge {
        from: (zone, 0),
        to: (wind, 0),
        delayed: true,
    })
    .expect("w");
    g.connect(Edge {
        from: (wind, 0),
        to: (step, 0),
        delayed: false,
    })
    .expect("w");
    g.connect(Edge {
        from: (step, 0),
        to: (zone, 1),
        delayed: false,
    })
    .expect("w");
    zone
}

/// **O terceiro fato, e o que condena a inferência por NOME:** o ritmo é do GRAFO — o maior que
/// qualquer declarante pede. Se uma corda conta como declarante, o `substeps` dela (um laço local
/// dentro do `eval` dela) passa a mandar no vizinho que nada tem com ela, e no plano inteiro do
/// device.
#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn does_a_rope_set_the_rhythm_of_a_neighbour_it_never_touches() {
    let reg = registry();
    eprintln!("\n[substeps-collision] a zona vizinha, com e sem a corda no MESMO grafo\n");
    eprintln!(
        "  {:>18}  {:>7}  {:>7}  {:>13}",
        "grafo", "ritmo", "ilhas", "zona: P final"
    );
    for (tag, with_rope) in [("so' a zona", false), ("zona + corda(8)", true)] {
        let mut g = Graph::new();
        let zone = falling_zone(&mut g);
        if with_rope {
            let r = g.add_node("motion.verlet_rope");
            g.set_param(r, "count", 24.0);
            g.set_param(r, "substeps", 8.0);
            g.connect(Edge {
                from: (r, 0),
                to: (r, 2),
                delayed: true,
            })
            .expect("loop");
        }
        g.validate(&reg).expect("bem-tipado");
        let rate = graph_substeps(&g, &reg);
        let islands = substep_islands(&g, &reg).len();
        // O vento sopra em +X, então é o X do único elemento que mede a queda dela.
        let p = march(&g, &reg, zone, true)
            .first()
            .map_or(f32::NAN, |q| q[0]);
        eprintln!("  {tag:>18}  {rate:>7}  {islands:>7}  {p:>13.6}");
    }
    eprintln!(
        "\n  LEITURA: a zona pede `substeps = 1` (o default) nos dois grafos. Se o ritmo e o
  numero dela mudarem so' por haver uma corda ao lado, o knob LOCAL de um no'
  esta' a mandar no relogio do grafo inteiro — e no device, no plano inteiro."
    );
}
