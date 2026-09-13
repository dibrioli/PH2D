//! ⭐⭐⭐ **O QUE CUSTA UM CARIMBO NUM MILHÃO DE PONTOS** — report do Enio, 2026-09-06:
//! *«no primeiro grafo tentei colocar 1000×1000 no grid e pesou muito. Retirando Shape e
//! Duplicator fica um pouco melhor.»*
//!
//! ⚠️ **A conta que ele fez é a régua certa:** ele bisseccionou sozinho — tirou dois nós e
//! mediu. Esta sonda faz a mesma bissecção com números, e junta a pergunta que ela não responde
//! sozinha: *aquela cadeia chega ao dispositivo?* — porque o tecto do módulo não está num kernel
//! ([doc 98](../../docs/Motion%20Nodes/98_auditoria_de_performance_2026-09-01.md)), está em
//! quantas cenas nunca lá chegam.
//!
//! ```text
//! cargo test -p ph2d-app-motion --lib --release -- --ignored --nocapture measure_the_stamp_at_a_million
//! ```

use crate::motion_state::MotionState;
use ph2d_nodegraph::graph::{Edge, NodeId};

/// Os lados da grade que a sonda varre. O `1000` é o do report.
const LADOS: [f32; 4] = [100.0, 320.0, 640.0, 1000.0];

fn wire(m: &mut MotionState, from: NodeId, fp: u16, to: NodeId, tp: u16) {
    m.doc
        .graph
        .connect(Edge {
            from: (from, fp),
            to: (to, tp),
            delayed: false,
        })
        .expect("liga");
}

/// Monta `grid(n×n) [→ duplicator ← source.shape] → output` e devolve o sink.
fn build(m: &mut MotionState, lado: f32, com_carimbo: bool) -> NodeId {
    let grid = m.doc.graph.add_node("motion.grid");
    m.doc.graph.set_param(grid, "rows", lado);
    m.doc.graph.set_param(grid, "cols", lado);
    let out = m.doc.graph.add_node("motion.output");
    if com_carimbo {
        let shape = m.doc.graph.add_node("source.shape");
        let dup = m.doc.graph.add_node("motion.duplicator");
        wire(m, shape, 0, dup, 0);
        wire(m, grid, 0, dup, 1);
        wire(m, dup, 0, out, 0);
    } else {
        wire(m, grid, 0, out, 0);
    }
    out
}

/// A mediana de `n` cozimentos FRIOS — um `MotionState` novo por corrida, senão o memo do cook
/// responde à segunda e a sonda mede a tabela de hash.
fn cook_ms(lado: f32, com_carimbo: bool) -> (f64, usize, bool) {
    let mut ms: Vec<f64> = Vec::new();
    let (mut n, mut gpu) = (0usize, false);
    for _ in 0..3 {
        let mut m = MotionState::new();
        let sink = build(&mut m, lado, com_carimbo);
        crate::motion_shape_gen::publish(&mut m, 0.0);
        gpu = ph2d_gpu_cook::plan(&m.doc.graph, &m.registry, &m.registry, sink).is_fully_gpu();
        let t = std::time::Instant::now();
        let out = m
            .pump
            .cook
            .cook(&m.doc.graph, &m.registry, sink, 0.0)
            .expect("coze");
        ms.push(t.elapsed().as_secs_f64() * 1000.0);
        n = out[0].as_stream().count();
    }
    ms.sort_by(f64::total_cmp);
    (ms[1], n, gpu)
}

#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE e com a máquina calma"]
fn measure_the_stamp_at_a_million() {
    eprintln!(
        "\n  load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    eprintln!(
        "\n  lado  | objectos  | so' a grade      | grade + carimbo   | o carimbo custa | no device?"
    );
    eprintln!(
        "  ------|-----------|------------------|-------------------|-----------------|-----------"
    );
    for lado in LADOS {
        let (nu, n_nu, gpu_nu) = cook_ms(lado, false);
        let (com, n_com, gpu_com) = cook_ms(lado, true);
        eprintln!(
            "  {lado:>5} | {n_com:>9} | {nu:>10.2} ms {} | {com:>11.2} ms {} | {:>13.1}× | {}",
            if gpu_nu { "🟢" } else { "🔴" },
            if gpu_com { "🟢" } else { "🔴" },
            com / nu.max(1e-9),
            if gpu_com { "sim" } else { "NAO" },
        );
        assert_eq!(n_nu, n_com, "o carimbo de UMA forma preserva a contagem");
    }
    eprintln!(
        "\n  🟢 = o planeador reivindica a cadeia inteira para o dispositivo · 🔴 = ela cai na CPU
  (um quadro de 60 fps tem 16,67 ms)\n"
    );
}
