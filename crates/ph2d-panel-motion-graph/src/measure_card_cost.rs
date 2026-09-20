//! **QUANTO CUSTA UM CARTÃO, pintado** — a outra metade do orçamento do ciclo 1
//! ([doc 103](../../../docs/Motion%20Nodes/103_dinamica_dos_ciclos.md)); a irmã mede o custo de
//! uma ROW (`ph2d-panel-motion-params::measure_row_cost`, **13,5 µs/row** a load 2,29).
//!
//! ⚠️ Ela existe porque o ciclo 1 põe os params DENTRO dos cartões: sem os dois números não há
//! como dizer quantos cartões podem mostrar params antes de o quadro estourar — e §0.0 proíbe
//! escrever um limite antes de o medir.
//!
//! ⚠️ **Os cartões sobrepõem-se de propósito.** O pintor descarta o que não toca o viewport
//! (`touches`), então espalhá-los para caberem mediria a CULLING; empilhá-los na tela força as
//! `n` pinturas, que é a variável.
//!
//! `cargo test -p ph2d-panel-motion-graph --release -- --ignored --nocapture measure_card_cost`
use crate::snapshot::{
    GraphNodeView, GraphViewSnapshot, NodeViewKind, PortView, set_current_motion_graph,
};
use crate::{MotionGraphPanel, MotionGraphPanelState};
use ph2d_editor_core::screens::layout::HeroLayout;
use ph2d_editor_core::zones::Rect;
use ph2d_node_registry::{NodeSilhouette, NodeUiCategory};
use ph2d_nodegraph::port::{Clock, Dim, Domain};

/// A tela de MEDIÇÃO (não um desenho) — a mesma do `measure_row_cost`, de propósito: dois
/// tamanhos diferentes mediriam duas telas e os números não se somariam.
const MEASURE_W: f32 = 1200.0; // LITERAL-PX-OK: tela de medicao, nao desenho
const MEASURE_H: f32 = 800.0; // LITERAL-PX-OK: tela de medicao, nao desenho
/// O passo da grelha de cartões dentro da tela — apertado de propósito (ver o doc do módulo).
const STEP_X: f32 = 100.0; // LITERAL-PX-OK: passo da fixtura
const STEP_Y: f32 = 60.0; // LITERAL-PX-OK: passo da fixtura
const COLS: u32 = 12;

/// O mesmo cartão com `k` rows de param — a variável do ciclo 1.
fn card_with_params(id: u32, k: usize) -> GraphNodeView {
    let port = |name: &'static str, dim| PortView {
        name,
        domain: Domain::Instances,
        dim,
        clock: Clock::Frame,
    };
    GraphNodeView {
        primary_input: 0,
        kind: NodeViewKind::Node,
        id,
        // Um nome de comprimento realista: o custo de um cartão é dominado pelo TEXTO, e um
        // nome de uma letra mediria outro programa.
        display_name: format!("Radial Array {id}"),
        category: NodeUiCategory::Source,
        silhouette: NodeSilhouette::Rect,
        x: (id % COLS) as f32 * STEP_X,
        y: (id / COLS) as f32 * STEP_Y,
        inputs: vec![port("in", Dim::Vec2), port("field", Dim::Scalar)],
        outputs: vec![port("out", Dim::Vec2)],
        readout: Some("400 rows".into()),
        count: Some(400),
        hot: false,
        is_sink: false,
        preview: None,
        bypassed: false,
        inert: false,
        thumbnail: None,
        params: (0..k)
            .map(|i| {
                crate::snapshot::CardParam::from_hint(
                    ph2d_node_registry::ParamUiHint {
                        param: "p",
                        // Rótulos realistas e VARIÁVEIS: o custo de uma row é dominado pelo
                        // texto, e medir sempre a mesma string mediria o cache.
                        label: [
                            "Rows", "Columns", "Gap X", "Gap Y", "Seed", "Radius", "Count", "Angle",
                        ][i % 8],
                        min: 0.0,
                        max: 20.0, // LITERAL-PX-OK: valor de FIXTURA de medicao, nao desenho
                        step: 0.1, // LITERAL-PX-OK: valor de FIXTURA de medicao, nao desenho
                        widget: ph2d_node_registry::ParamWidget::Slider,
                    },
                    1.0 + i as f32,
                )
            })
            .collect(),
        sections: Vec::new(),
    }
}

fn paint_ms(cards: u32, n: u32) -> f64 {
    paint_ms_params(cards, 0, n)
}

fn paint_ms_params(cards: u32, k: usize, n: u32) -> f64 {
    set_current_motion_graph(Some(GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
        nodes: (0..cards).map(|i| card_with_params(i, k)).collect(),
        edges: Vec::new(),
        backdrops: Vec::new(),
        probe: None,
        now: 0.0,
    }));
    let viewport = Rect::new(0.0, 0.0, MEASURE_W, MEASURE_H);
    let mut layout = HeroLayout::for_viewport(viewport);
    // ⚠️ `for_viewport` deixa o centro POR PARTIR, e um painel que vive no split recebe um
    // rect de área ZERO e volta antes de desenhar (doc do `paint_with_layout`). Sem esta
    // linha a sonda mediria uma pintura que não acontece.
    layout.motion_graph = viewport;
    let mut melhor = f64::MAX;
    for _ in 0..3 {
        let t0 = std::time::Instant::now();
        for _ in 0..n {
            let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionGraphPanel>();
            let mut state = MotionGraphPanelState::default();
            let _ = host.paint_with_layout::<MotionGraphPanel>(&mut state, layout, viewport);
        }
        melhor = melhor.min(t0.elapsed().as_secs_f64() * 1000.0 / f64::from(n)); // LITERAL-PX-OK: us por ms, conversao de unidade de RELOGIO
    }
    melhor
}

#[test]
#[ignore = "medicao"]
fn measure_card_cost() {
    const N: u32 = 200;
    eprintln!(
        "  load: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    eprintln!(
        "  {:>6} │ {:>10} │ {:>14}",
        "cartoes", "ms/pintura", "us/cartao (marg.)"
    );
    let mut anterior: Option<(u32, f64)> = None;
    for cards in [0u32, 1, 5, 10, 20, 40, 80, 120] {
        let ms = paint_ms(cards, N);
        let marg = anterior.map_or(f64::NAN, |(c0, m0)| {
            (ms - m0) * 1000.0 / f64::from(cards - c0) // LITERAL-PX-OK: us por ms, conversao de unidade de RELOGIO
        });
        eprintln!("  {cards:>6} │ {ms:>10.4} │ {marg:>14.2}");
        anterior = Some((cards, ms));
    }

    // ⭐ A VARIÁVEL DO CICLO 1: o mesmo cartão com rows de param desenhadas.
    eprintln!("  --- 20 cartoes, R rows de param cada (zoom 1 => acima do LOD) ---");
    eprintln!(
        "  {:>5} │ {:>10} │ {:>14}",
        "rows", "ms/pintura", "us/row (marg.)"
    );
    let mut ant: Option<(usize, f64)> = None;
    for k in [0usize, 1, 2, 4, 8, 16] {
        let ms = paint_ms_params(20, k, N);
        const US_POR_MS: f64 = 1000.0; // LITERAL-PX-OK: conversao de unidade de RELOGIO
        let marg = ant.map_or(f64::NAN, |(k0, m0)| {
            (ms - m0) * US_POR_MS / (20 * (k - k0)) as f64
        });
        eprintln!("  {k:>5} │ {ms:>10.4} │ {marg:>14.2}");
        ant = Some((k, ms));
    }
    set_current_motion_graph(None);
    eprintln!("  (N cartoes x R rows tem de caber nos 16,67 ms do quadro — doc 103 §7)");
}
