//! ⭐⭐ **A AUDITORIA DO GRUPO DOS TRANSFORMES & DEFORMADORES** (ciclo 3, passo 2 — doc 106).
//!
//! Mesma régua dos ciclos 1 e 2: a auditoria começa por **medir o que existe**, nunca por uma
//! lista do que eu acho que falta.
//!
//! ⚠️ **A coluna do dispositivo lê-se do `register_gpu_kernel`, NUNCA do `lowerings`** — o
//! ciclo 2 pagou essa: o `lowerings` é o que o nó declara saber baixar **sozinho**, e imprimir
//! essa coluna fabrica uma tabela de dívida com oito kernels que já existem.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --bins --release -- --ignored --nocapture audit_the_deformer_group
//! ```

use crate::motion_state::MotionState;
use ph2d_nodegraph::graph::{Edge, NodeId};

/// Os treze do ciclo 3 (doc 103 §5).
pub(crate) const GRUPO: [&str; 13] = [
    "motion.move",
    "motion.rotate",
    "motion.scale",
    "motion.transform",
    "motion.mirror",
    "motion.look_at",
    "motion.bend",
    "motion.twist",
    "motion.spherize",
    "motion.four_point_warp",
    "motion.bezier_warp",
    "motion.kaleidoscope",
    "motion.spline_wrap",
];

#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn audit_the_deformer_group() {
    let base = MotionState::new();
    eprintln!("\n  nó                     | params | no cartão | device | portas | efeito");
    eprintln!("  -----------------------|--------|-----------|--------|--------|--------");
    for nome in GRUPO {
        let mut m = MotionState::new();
        let id = m.doc.graph.add_node(nome.to_string());
        let tid = m.doc.graph.node(id).expect("no'").type_id();
        let man = {
            use ph2d_nodegraph::cook::OpResolver;
            base.registry.resolve(tid).map(|op| op.manifest())
        };
        let Some(man) = man else {
            eprintln!("  {nome:<23} | (nao registado)");
            continue;
        };
        let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
        crate::render_loop::motion_bridge::params::card::stamp_card_params(
            &m,
            ph2d_editor::ProjectSettings::default(),
            &mut snap,
        );
        let no_cartao = snap
            .nodes
            .iter()
            .find(|v| v.id == id.0)
            .map_or(0, |v| v.params.len());
        let device = {
            use ph2d_nodegraph::gpu::KernelResolver;
            m.registry.gpu_kernel(tid).is_some()
        };
        eprintln!(
            "  {nome:<23} | {:>6} | {no_cartao:>9} | {:^6} | {}->{:<4} | {:?}",
            man.params.len(),
            if device { "sim" } else { "NAO" },
            man.inputs.len(),
            man.outputs.len(),
            man.effect,
        );
    }
    eprintln!();
}

/// **OS PARAMS DE CADA NÓ DO GRUPO, um a um** — o que a auditoria compara contra as
/// referências. ⚠️ Sem esta lista, «falta X» é um palpite.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins -- --ignored --nocapture what_each_deformer_offers
/// ```
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn what_each_deformer_offers() {
    let m = MotionState::new();
    for nome in GRUPO {
        let tid = ph2d_nodegraph::node::NodeTypeId::of(nome);
        let man = {
            use ph2d_nodegraph::cook::OpResolver;
            let Some(op) = m.registry.resolve(tid) else {
                continue;
            };
            op.manifest()
        };
        eprintln!("\n  === {nome} ===");
        eprintln!(
            "  portas: [{}] -> [{}]",
            man.inputs
                .iter()
                .map(|p| p.name)
                .collect::<Vec<_>>()
                .join(", "),
            man.outputs
                .iter()
                .map(|p| p.name)
                .collect::<Vec<_>>()
                .join(", ")
        );
        let hints = m.registry.param_ui(tid).unwrap_or(&[]);
        for spec in man.params {
            let h = hints.iter().find(|h| h.param == spec.name);
            let w = h.map_or("(sem hint)".to_string(), |h| match h.widget {
                ph2d_node_registry::ParamWidget::Enum { labels } => {
                    format!("Enum[{}]", labels.join("|"))
                }
                outro => format!("{outro:?}"),
            });
            eprintln!(
                "    {:<18} default {:>9.3}  {:<16}  {}",
                spec.name,
                spec.default,
                h.map_or("—".to_string(), |h| format!("{}..{}", h.min, h.max)),
                w
            );
        }
    }
    eprintln!();
}

// ---------------------------------------------------------------------------------------------
// ⭐⭐⭐ O PREÇO DE CADA NÓ DO GRUPO — e, sobretudo, o preço de ele NÃO chegar ao dispositivo.
// ---------------------------------------------------------------------------------------------

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

/// Monta `motion.grid(lado × lado) [→ <nó>] → motion.output` e devolve o sink.
fn build_com(m: &mut MotionState, lado: f32, no: Option<&str>) -> NodeId {
    let grid = m.doc.graph.add_node("motion.grid");
    m.doc.graph.set_param(grid, "rows", lado);
    m.doc.graph.set_param(grid, "cols", lado);
    let out = m.doc.graph.add_node("motion.output");
    match no {
        Some(nome) => {
            let d = m.doc.graph.add_node(nome.to_string());
            wire(m, grid, 0, d, 0);
            wire(m, d, 0, out, 0);
        }
        None => wire(m, grid, 0, out, 0),
    }
    out
}

/// A mediana de 3 cozimentos **FRIOS** — um `MotionState` novo por corrida, senão o memo do cook
/// responde à segunda e a sonda mede a tabela de hash. Devolve `(ms, n, no_device)`.
fn cook_com(lado: f32, no: Option<&str>) -> (f64, usize, bool) {
    let mut ms: Vec<f64> = Vec::new();
    let (mut n, mut gpu) = (0usize, false);
    for _ in 0..3 {
        let mut m = MotionState::new();
        let sink = build_com(&mut m, lado, no);
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

/// ⭐⭐⭐ **O PREÇO DO GRUPO, e o 🔴 é o achado — nunca a razão** (doc 103 §5.1).
///
/// Um nó que cai na CPU **no meio de uma cadeia** não custa o que ele custa: custa o
/// dispositivo inteiro ([doc 98](../../docs/Motion%20Nodes/98_auditoria_de_performance_2026-09-01.md)
/// mediu `50,9×`). Por isso a coluna que decide é *«a cadeia inteira é reivindicada?»*, e o
/// relógio está ao lado só para dizer quanto.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins --release -- --ignored --nocapture measure_the_deformer_group
/// ```
#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE e com a máquina calma"]
fn measure_the_deformer_group() {
    let lado: f32 = std::env::var("PH2D_LADO")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(320.0);
    eprintln!(
        "\n  load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    let (nu, n_nu, gpu_nu) = cook_com(lado, None);
    eprintln!(
        "\n  grade {lado}×{lado} = {n_nu} objectos · so' a grade: {nu:.2} ms {}",
        if gpu_nu { "🟢" } else { "🔴" }
    );
    eprintln!("\n  nó                     | objectos  | cozimento  | vs. so' a grade | cadeia no device?");
    eprintln!("  -----------------------|-----------|------------|-----------------|------------------");
    for nome in GRUPO {
        let (ms, n, gpu) = cook_com(lado, Some(nome));
        eprintln!(
            "  {nome:<23} | {n:>9} | {ms:>7.2} ms | {:>14.2}× | {}",
            ms / nu.max(1e-9),
            if gpu { "🟢 sim" } else { "🔴 NAO" },
        );
    }
    eprintln!(
        "\n  🟢 = o planeador reivindica a cadeia inteira · 🔴 = ela cai na CPU
  (um quadro de 60 fps tem 16,67 ms)\n"
    );
}
