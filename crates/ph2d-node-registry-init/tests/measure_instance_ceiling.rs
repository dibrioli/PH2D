//! **SONDA — O `MAX_INSTANCES` NUNCA FOI MEDIDO** (CLAUDE.md §0, doc 89 folha 07).
//!
//! Três nós carregam o MESMO literal `65_536` e nenhum traz uma medição ao lado:
//!
//! * `motion.trail` — *"4096 vivas × 32 ecos já é 131k quads"*, e o teto **CLAMPA** o número
//!   de gerações (o rastro encurta, em silêncio);
//! * `fx.drop_shadow` (`2 × n`) e `fx.rgb_split` (`3 × n`) — acima do teto o efeito se
//!   **DESLIGA** (devolve a entrada verbatim, em silêncio).
//!
//! ⚠️ **A célula da folha nomeou UM nó; medido, são TRÊS** — a mesma forma do canal `falloff`
//! do grupo F. E os dois modos de falha são opostos: um encurta a cauda, os outros somem com
//! o efeito, e nenhum diz nada.
//!
//! ⚠️ **Um limite legítimo diz DE QUE RECURSO ele é.** Estes três não dizem, e a justificativa
//! escrita (*"131k quads"*) é uma contagem, não um custo: a `line/gpu-nodes` mediu **4,19 M
//! partículas em 3,6 ms** na GPU no mesmo módulo. Esta sonda mede o custo do caminho que estes
//! três nós de facto percorrem — **CPU**, porque os três são `LoweringKind::Cpu` — e o número
//! que ela der é o que vai no lugar do literal.
//!
//! Rodar:
//! `cargo test -p ph2d-node-registry-init --release --test measure_instance_ceiling -- --ignored --nocapture`

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use std::time::Instant;

/// Um quadro de 60 fps, em milissegundos — a régua contra a qual todo custo por-tick desta
/// sonda é lido.
const FRAME_MS: f64 = 1000.0 / 60.0;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

/// Uma grade de `side²` elementos **QUE SE MEXE**, e o `motion.oscillator` não é enfeite.
///
/// ⚠️ **A 1ª versão desta sonda alimentava uma grade PARADA e mediu `0,001 ms` para um milhão
/// de linhas.** Um rastro sobre uma fonte imóvel converge para um ponto fixo — a saída do tick
/// `n` é byte a byte a do tick `n−1` —, e aí o **memo** do `Cook` devolve o valor sem cozer
/// nada: a sonda estava a cronometrar uma consulta de cache e a chamar-lhe *o custo de um
/// rastro*. A fonte tem de se mexer para o nó ter trabalho a fazer.
fn grid(g: &mut Graph, side: f32) -> NodeId {
    let seed = g.add_node("motion.grid");
    g.set_param(seed, "rows", side);
    g.set_param(seed, "cols", side);
    let osc = g.add_node("motion.oscillator");
    g.set_param(osc, "amplitude", 3.0);
    g.set_param(osc, "frequency", 0.7);
    wire(g, seed, 0, osc, 0);
    osc
}

fn wire(g: &mut Graph, from: NodeId, port: u16, to: NodeId, to_port: u16) {
    g.connect(Edge {
        from: (from, port),
        to: (to, to_port),
        delayed: false,
    })
    .expect("edge");
}

/// A MEDIANA de `runs` corridas — a máquina é compartilhada, e um mínimo premiaria a corrida
/// mais sortuda enquanto uma média deixaria um pico de escalonador decidir o número.
fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// Cozinha `ticks` ticks de um grafo com `pre` self-loop e devolve `(linhas emitidas, ms por
/// tick)`. O 1º tick é DESCARTADO: ele aloca, e o que interessa é o regime.
fn cost_sequential(g: &Graph, reg: &NodeRegistry, target: NodeId, ticks: u32) -> (usize, f64) {
    let mut cook = Cook::new();
    let mut rows = 0usize;
    let mut ms = Vec::new();
    for t in 0..ticks {
        let t = f64::from(t);
        let start = Instant::now();
        let out = cook.cook(g, reg, target, t).expect("cooks");
        rows = out[0].as_stream().count();
        cook.advance_tick(g, reg, t).expect("advance");
        let e = start.elapsed().as_secs_f64() * 1000.0;
        if t > 0.0 {
            ms.push(e);
        }
    }
    (rows, median(ms))
}

/// O mesmo para um nó SEM estado: `runs` cozeduras independentes.
fn cost_pure(g: &Graph, reg: &NodeRegistry, target: NodeId, runs: u32) -> (usize, f64) {
    let mut rows = 0usize;
    let mut ms = Vec::new();
    for _ in 0..runs {
        let mut cook = Cook::new();
        let start = Instant::now();
        let out = cook.cook(g, reg, target, 0.0).expect("cooks");
        rows = out[0].as_stream().count();
        ms.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    (rows, median(ms))
}

/// **Quanto custa um eco.** Varre `live × length` e reporta o custo por tick e por linha.
#[test]
#[ignore = "sonda: mede, nao afirma"]
fn measure_what_a_trail_costs_per_emitted_row() {
    let reg = registry();
    println!("\n== motion.trail: o custo de um tick, por linha emitida ==");
    println!(
        "{:>7} {:>7} {:>10} {:>10} {:>10}",
        "live", "length", "emitidas", "ms/tick", "ns/linha"
    );
    for &side in &[16.0f32, 32.0, 64.0] {
        for &length in &[4.0f32, 16.0, 32.0] {
            let mut g = Graph::new();
            let seed = grid(&mut g, side);
            let tr = g.add_node("motion.trail");
            g.set_param(tr, "length", length);
            wire(&mut g, seed, 0, tr, 0);
            g.connect(Edge {
                from: (tr, 0),
                to: (tr, 1),
                delayed: true,
            })
            .expect("ring");
            let (rows, ms) = cost_sequential(&g, &reg, tr, 40);
            let live = (side * side) as usize;
            println!(
                "{live:>7} {length:>7} {rows:>10} {ms:>10.3} {:>10.1}",
                ms * 1.0e6 / rows.max(1) as f64
            );
        }
    }
    println!("(um quadro de 60 fps = {FRAME_MS:.2} ms)");
}

/// **Quanto custa uma cópia de FX.** Os dois nós que se desligam acima do teto.
#[test]
#[ignore = "sonda: mede, nao afirma"]
fn measure_what_the_fx_copies_cost_per_emitted_row() {
    let reg = registry();
    println!("\n== fx.drop_shadow (2x) e fx.rgb_split (3x): o custo de uma cozedura ==");
    println!(
        "{:>10} {:>16} {:>10} {:>10} {:>10}",
        "entrada", "no", "emitidas", "ms", "ns/linha"
    );
    for &side in &[32.0f32, 64.0, 128.0] {
        for ty in ["fx.drop_shadow", "fx.rgb_split"] {
            let mut g = Graph::new();
            let seed = grid(&mut g, side);
            let fx = g.add_node(ty);
            wire(&mut g, seed, 0, fx, 0);
            let (rows, ms) = cost_pure(&g, &reg, fx, 9);
            println!(
                "{:>10} {ty:>16} {rows:>10} {ms:>10.3} {:>10.1}",
                (side * side) as usize,
                ms * 1.0e6 / rows.max(1) as f64
            );
        }
    }
    println!("(um quadro de 60 fps = {FRAME_MS:.2} ms)");
}

/// Bytes que o stream emitido de facto ocupa — a soma das colunas que ele carrega, não uma
/// estimativa: a memória é o segundo recurso que um teto de instâncias pode ser de.
fn stream_bytes(s: &ph2d_nodegraph::attr::Stream) -> usize {
    use ph2d_nodegraph::attr::Column;
    s.columns()
        .map(|(_, c)| match c {
            Column::Scalar(v) => v.len() * 4,
            Column::Vec2(v) => v.len() * 8,
            Column::Vec3(v) => v.len() * 12,
            Column::Vec4(v) => v.len() * 16,
        })
        .sum()
}

/// **A VARREDURA ALTA — onde é a parede?** Na faixa que o teto de hoje alcança o custo é
/// linear; esta sonda vai muito acima dele, com o teto do nó NEUTRALIZADO pela forma
/// (`length` baixo × muitas vivas produz o mesmo número de linhas sem bater no clamp), para
/// que a coluna `emitidas` seja o que foi pedido e não o que sobreviveu.
#[test]
#[ignore = "sonda: mede, nao afirma"]
fn measure_where_the_wall_actually_is() {
    let reg = registry();
    println!("\n== o rastro muito acima do teto de hoje: onde a linearidade acaba ==");
    println!(
        "{:>8} {:>7} {:>10} {:>10} {:>10} {:>9} {:>8}",
        "vivas", "length", "emitidas", "ms/tick", "ns/linha", "% quadro", "MB"
    );
    for &(side, length) in &[
        (256.0f32, 1.0f32),
        (362.0, 1.0),
        (512.0, 1.0),
        (724.0, 1.0),
        (1024.0, 1.0),
        (512.0, 4.0),
    ] {
        let mut g = Graph::new();
        let seed = grid(&mut g, side);
        let tr = g.add_node("motion.trail");
        g.set_param(tr, "length", length);
        wire(&mut g, seed, 0, tr, 0);
        g.connect(Edge {
            from: (tr, 0),
            to: (tr, 1),
            delayed: true,
        })
        .expect("ring");
        let (rows, ms) = cost_sequential(&g, &reg, tr, 16);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &reg, tr, 1.0).expect("cooks");
        let mb = stream_bytes(out[0].as_stream()) as f64 / 1.0e6;
        println!(
            "{:>8} {length:>7} {rows:>10} {ms:>10.3} {:>10.1} {:>8.1}% {mb:>8.2}",
            (side * side) as usize,
            ms * 1.0e6 / rows.max(1) as f64,
            ms * 100.0 / FRAME_MS
        );
    }
}

/// **Onde o teto de HOJE cai, em milissegundos** — a pergunta que a justificativa escrita
/// (*"131k quads"*) não responde.
#[test]
#[ignore = "sonda: mede, nao afirma"]
fn measure_where_todays_ceiling_lands_on_the_clock() {
    let reg = registry();
    println!("\n== o custo de um tick NO teto de hoje (65_536 linhas emitidas) ==");
    // 2048 vivas × 32 ecos = 65_536 — exactamente o teto, pela rota do rastro.
    let mut g = Graph::new();
    let seed = grid(&mut g, 45.0); // 2025 vivas
    let tr = g.add_node("motion.trail");
    g.set_param(tr, "length", 32.0);
    wire(&mut g, seed, 0, tr, 0);
    g.connect(Edge {
        from: (tr, 0),
        to: (tr, 1),
        delayed: true,
    })
    .expect("ring");
    let (rows, ms) = cost_sequential(&g, &reg, tr, 40);
    println!(
        "motion.trail  {rows:>8} linhas  {ms:>8.3} ms/tick  = {:.1}% de um quadro",
        ms * 100.0 / FRAME_MS
    );

    for ty in ["fx.drop_shadow", "fx.rgb_split"] {
        let mut g = Graph::new();
        let seed = grid(&mut g, 148.0); // 21904 vivas -> 43k/65k linhas
        let fx = g.add_node(ty);
        wire(&mut g, seed, 0, fx, 0);
        let (rows, ms) = cost_pure(&g, &reg, fx, 9);
        println!(
            "{ty:>13}  {rows:>8} linhas  {ms:>8.3} ms       = {:.1}% de um quadro",
            ms * 100.0 / FRAME_MS
        );
    }
}
