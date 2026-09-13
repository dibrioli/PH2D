//! **A TEE CONTRA O NÓ FUNDIDO** — as sondas do estudo dos outputs
//! ([doc 100](../../../docs/Motion%20Nodes/100_estudo_dos_outputs_2026-09-04.md)).
//!
//! O Mini Cavalry dá ao oscilador três pinos (forma · número · disparo); a casa dá um, e
//! entrega o número por OUTRO nó (`value.lfo → motion.drive`). Três perguntas, medidas:
//! os dois caminhos dão os MESMOS bits? · os dois chegam ao device? · o que custa cada um
//! na CPU a um milhão de elementos?

use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

const AMPLITUDE: f32 = 37.0;
const STAGGER: f32 = 0.013;

fn wire(g: &mut Graph, from: (NodeId, u16), to: (NodeId, u16)) {
    g.connect(Edge {
        from,
        to,
        delayed: false,
    })
    .expect("as portas encaixam");
}

fn grid(g: &mut Graph, rows: f32, cols: f32) -> NodeId {
    let id = g.add_node("motion.grid");
    g.set_param(id, "rows", rows);
    g.set_param(id, "cols", cols);
    id
}

/// `grid -> motion.oscillator(Y) -> output`: o nó FUNDIDO, um pino.
fn fused(g: &mut Graph, rows: f32, cols: f32, frequency: f32) -> NodeId {
    let src = grid(g, rows, cols);
    let osc = g.add_node("motion.oscillator");
    g.set_param(osc, "channel", 1.0);
    g.set_param(osc, "frequency", frequency);
    g.set_param(osc, "amplitude", AMPLITUDE);
    g.set_param(osc, "phase_stagger", STAGGER);
    let out = g.add_node("motion.output");
    wire(g, (src, 0), (osc, 0));
    wire(g, (osc, 0), (out, 0));
    out
}

/// `grid -> value.lfo(in = grid) -> motion.drive(in = grid, value = lfo, Add, Y) -> output`:
/// a TEE — a mesma onda, num pino que qualquer outro nó também pode ler.
fn tee(g: &mut Graph, rows: f32, cols: f32, period: f32) -> NodeId {
    let src = grid(g, rows, cols);
    let lfo = g.add_node("value.lfo");
    g.set_param(lfo, "period", period);
    g.set_param(lfo, "amplitude", AMPLITUDE);
    g.set_param(lfo, "phase_stagger", STAGGER);
    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", 1.0);
    g.set_param(drive, "mode", 0.0); // Add
    g.set_param(drive, "scale", 1.0);
    let out = g.add_node("motion.output");
    wire(g, (src, 0), (lfo, 0));
    wire(g, (src, 0), (drive, 0));
    wire(g, (lfo, 0), (drive, 1));
    wire(g, (drive, 0), (out, 0));
    out
}

fn positions(motion: &MotionState, g: &Graph, out: NodeId, t: f64) -> (Vec<[f32; 2]>, f64) {
    let mut cook = Cook::new();
    let clock = std::time::Instant::now();
    let r = cook.cook(g, &motion.registry, out, t).expect("cozinha");
    let ms = clock.elapsed().as_secs_f64() * 1000.0;
    let p = match r.first().map(|v| v.as_stream()).and_then(|s| s.get("P")) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    };
    (p, ms)
}

fn compare(motion: &MotionState, frequency: f32, period: f32, rotulo: &str) {
    eprintln!("  --- {rotulo} ---");
    eprintln!(
        "  {:>8} │ {:>9} │ {:>12} │ {:>8}",
        "t", "diferem", "max |dy|", "max ULP"
    );
    for t in [0.0_f64, 0.1, 0.37, 1.234, 7.5, 33.3] {
        let mut a = Graph::new();
        let oa = fused(&mut a, 100.0, 100.0, frequency);
        let mut b = Graph::new();
        let ob = tee(&mut b, 100.0, 100.0, period);
        let (pa, _) = positions(motion, &a, oa, t);
        let (pb, _) = positions(motion, &b, ob, t);
        assert_eq!(pa.len(), pb.len(), "as duas grelhas tem a mesma contagem");
        assert!(!pa.is_empty(), "a grelha nao esta vazia");
        let mut differ = 0usize;
        let mut max_dy = 0.0f32;
        let mut max_ulp = 0u32;
        for (x, y) in pa.iter().zip(&pb) {
            if x[1].to_bits() != y[1].to_bits() {
                differ += 1;
                max_dy = max_dy.max((x[1] - y[1]).abs());
                max_ulp = max_ulp.max(x[1].to_bits().abs_diff(y[1].to_bits()));
            }
        }
        eprintln!(
            "  {t:>8.3} │ {differ:>5}/{:<3} │ {max_dy:>12.3e} │ {max_ulp:>8}",
            pa.len()
        );
    }
}

/// `cargo test -p ph2d-app-motion --release --lib -- --ignored --nocapture the_tee_against_the_fused_node`
#[test]
#[ignore = "sonda de medicao, nao um gate"]
fn the_tee_against_the_fused_node() {
    let motion = MotionState::new();
    compare(
        &motion,
        2.0,
        0.5,
        "os MESMOS bits? (100x100 · 2 Hz = periodo 0,5 s, exactos em binario)",
    );
    compare(
        &motion,
        3.0,
        1.0 / 3.0,
        "e a 3 Hz = periodo 1/3 s (t*f contra t/(1/f), que o f32 nao iguala)",
    );

    eprintln!("  --- chegam ao device? ---");
    let mut a = Graph::new();
    let oa = fused(&mut a, 100.0, 100.0, 2.0);
    let mut b = Graph::new();
    let ob = tee(&mut b, 100.0, 100.0, 0.5);
    for (rotulo, g, out) in [("fundido", &a, oa), ("tee", &b, ob)] {
        let plan = ph2d_gpu_cook::plan(g, &motion.registry, &motion.registry, out);
        eprintln!(
            "  {rotulo:<8} │ fully_gpu = {:<5} │ {} estagios │ {} fronteiras │ {} passes",
            plan.is_fully_gpu(),
            plan.stages.len(),
            plan.boundaries.len(),
            plan.dispatching_stages(&motion.registry)
        );
    }

    eprintln!(
        "  --- custo na CPU a 1 000 000 (1000x1000): cozimento frio, depois um segundo instante ---"
    );
    let mut a = Graph::new();
    let oa = fused(&mut a, 1000.0, 1000.0, 2.0);
    let mut b = Graph::new();
    let ob = tee(&mut b, 1000.0, 1000.0, 0.5);
    for (rotulo, g, out) in [("fundido", &a, oa), ("tee", &b, ob)] {
        let (p0, frio) = positions(&motion, g, out, 0.3);
        let (p1, quente) = positions(&motion, g, out, 0.6);
        eprintln!(
            "  {rotulo:<8} │ {:>9} linhas │ frio {frio:>8.2} ms │ quente {quente:>8.2} ms",
            p0.len().min(p1.len())
        );
    }
}
