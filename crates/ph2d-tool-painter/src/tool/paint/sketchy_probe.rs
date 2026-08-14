//! **O QUE UM TRAÇO DE SKETCHY CUSTA POR EVENTO** — a medição que o doc do
//! `ph2d_painter_brush::line_kind::SKETCHY_DENSITY_MAX` EXIGE de quem construísse o rasterizador:
//!
//! > *"O recurso de verdade é o tempo de rasterização por evento, contra o kill de 8 ms desta casa —
//! > e ele não pode ser medido antes de o rasterizador de fios existir. Este número fica como ponto
//! > de partida declarado; quem construir a rasterização mede e o substitui, com a tabela ao lado."*
//!
//! ⚠️ **Pela porta do PRODUTO** (`on_canvas_pointer`), nunca por um laço próprio: o que decide o teto
//! é o que o artista paga por movimento do dedo, e a lição de que uma sonda com laço próprio fica
//! CEGA à porta já custou duas waves a este módulo (doc 28 §5.11, §5.46).
//!
//! Rodar: `cargo test -p ph2d-tool-painter --release measure_the_sketchy -- --ignored --nocapture`

use super::measure_shape_system::{cp, tool};
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_painter_brush::StrokeMethod;
use ph2d_painter_brush::line_kind::{LineKind, SKETCHY_DENSITY_MAX};
use std::time::Instant;

/// Uma espiral apertada: ela volta para perto de si mesma, que é onde o Sketchy tem vizinhos para
/// costurar. É a fixture do orçamento — um traço reto quase não costura, e mediria o nada.
///
/// Devolve `(ms por evento de Move, ms do pior evento)`.
fn spiral(density: f32, reach: f32, width: f32, radius: f32, turns: usize) -> (f64, f64) {
    let side = 2048u32;
    let mut t = tool(side, PaintMedia::Digital, radius);
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.paint.brush.line_kind = LineKind::Sketchy;
    t.paint.brush.sketchy_reach = reach;
    t.paint.brush.sketchy_density = density;
    t.paint.brush.sketchy_width_px = width;
    t.paint.brush.sketchy_opacity = 0.25;
    #[allow(clippy::cast_precision_loss)]
    let c = (side / 2) as f32;
    let steps = turns * 40;
    let pt = |i: usize| {
        #[allow(clippy::cast_precision_loss)]
        let u = i as f32 / steps as f32;
        // ⚠️ `cos`/`sin` aqui é de SONDA, não de produto: o HR-5 fala do caminho que pinta.
        let ang = u * (turns as f32) * std::f32::consts::TAU;
        let r = 20.0 + u * 180.0;
        [c + r * ang.cos(), c + r * ang.sin()]
    };
    t.on_canvas_pointer(cp(pt(0), PointerPhase::Down));
    let mut total = 0.0f64;
    let mut worst = 0.0f64;
    for i in 1..=steps {
        let t0 = Instant::now();
        t.on_canvas_pointer(cp(pt(i), PointerPhase::Move));
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        total += ms;
        worst = worst.max(ms);
    }
    t.on_canvas_pointer(cp(pt(steps), PointerPhase::Up));
    #[allow(clippy::cast_precision_loss)]
    (total / steps as f64, worst)
}

/// **O TETO DA DENSIDADE** — o número que o doc da constante manda medir e substituir.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_the_sketchy_budget_per_event() {
    println!("[sketchy] espiral de 8 voltas, pincel r=24, 2048², pela porta do produto");
    println!(
        "{:>9} {:>7} {:>7}  {:>10} {:>10}  {:>10}",
        "density", "reach", "width", "ms/evento", "pior ms", "veredito"
    );
    for &(d, reach, w) in &[
        (0.0f32, 1.0f32, 1.0f32),
        (SKETCHY_DENSITY_MAX, 1.0, 1.0),
        (SKETCHY_DENSITY_MAX, 4.0, 1.0),
        (SKETCHY_DENSITY_MAX, 4.0, 4.0),
        (0.10, 1.0, 1.0),
        (0.25, 1.0, 1.0),
        (0.50, 1.0, 1.0),
        (1.00, 1.0, 1.0),
        (1.00, 4.0, 1.0),
        (1.00, 4.0, 4.0),
        // O PIOR CANTO varrido: com o alcance no teto, até onde a densidade cabe?
        (0.10, 4.0, 4.0),
        (0.20, 4.0, 4.0),
        (0.30, 4.0, 4.0),
        (0.40, 4.0, 4.0),
        (0.50, 4.0, 4.0),
        (0.70, 4.0, 4.0),
    ] {
        let (avg, worst) = spiral(d, reach, w, 24.0, 8);
        println!(
            "{d:>9.3} {reach:>7.1} {w:>7.1}  {avg:>10.3} {worst:>10.3}  {:>10}",
            if worst > 8.0 { "ESTOURA" } else { "sob o kill" }
        );
    }
    println!(
        "[sketchy] leitura: o teto é a maior densidade cujo PIOR evento cabe no kill de 8 ms."
    );
}

/// **A LEI É LIVRE DE ESCALA?** — a W0.3 mediu `fios/dab ≈ 8` em qualquer pincel porque o alcance
/// escala com ele. Aqui a pergunta é a do RELÓGIO: o custo por evento também é?
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_the_sketchy_cost_across_brush_sizes() {
    println!("[sketchy] o mesmo traço, pincéis diferentes (density no teto, reach 1, width 1)");
    println!("{:>8}  {:>10} {:>10}", "raio", "ms/evento", "pior ms");
    for r in [6.0f32, 12.0, 24.0, 48.0, 96.0] {
        let (avg, worst) = spiral(SKETCHY_DENSITY_MAX, 1.0, 1.0, r, 8);
        println!("{r:>8.0}  {avg:>10.3} {worst:>10.3}");
    }
    println!("[sketchy] leitura: se o ms/evento for ~plano, o teto vale para todo pincel.");
}
