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
use ph2d_painter_brush::line_kind::{
    LineKind, SKETCHY_DENSITY_MAX, WIRE_CURVES_PER_DAB, WIRE_HISTORY_MAX,
};
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
    t.paint.brush.thread_width_px = width;
    t.paint.brush.thread_opacity = 0.25;
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

/// A MESMA espiral, com o Wire armado. Devolve `(ms por Move, ms do pior Move)`.
fn wire_spiral(history: f32, width: f32, radius: f32, turns: usize) -> (f64, f64) {
    let side = 2048u32;
    let mut t = tool(side, PaintMedia::Digital, radius);
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.paint.brush.line_kind = LineKind::Wire;
    t.paint.brush.wire_history = history;
    t.paint.brush.thread_width_px = width;
    t.paint.brush.thread_opacity = 0.25;
    #[allow(clippy::cast_precision_loss)]
    let c = (side / 2) as f32;
    let steps = turns * 40;
    let pt = |i: usize| {
        #[allow(clippy::cast_precision_loss)]
        let u = i as f32 / steps as f32;
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

/// **O TETO DO `History` E A CONTAGEM DE CORDAS DO WIRE** — os dois números que o plano 38 W4 exige
/// medidos, contra o mesmo kill de 8 ms do irmão.
///
/// ⚠️ **Os dois governam o MESMO orçamento e por isso são medidos juntos:** o custo de rasterizar é
/// linear em `contagem × comprimento médio de corda × largura`, e o comprimento médio é metade da
/// janela ⇒ `custo ∝ WIRE_CURVES_PER_DAB × history × width`. Medir um sem o outro daria um teto que
/// o produto não sustenta assim que alguém mexesse no que ficou de fora.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_the_wire_budget_per_event() {
    println!("[wire] espiral de 8 voltas, pincel r=24, 2048², pela porta do produto");
    println!("[wire] cordas/dab = {WIRE_CURVES_PER_DAB} (a constante de hoje)");
    println!(
        "{:>9} {:>7}  {:>10} {:>10}  {:>10}",
        "history", "width", "ms/evento", "pior ms", "veredito"
    );
    for &(h, w) in &[
        (0.0f32, 1.0f32),
        (1.0, 1.0),
        (3.0, 1.0),
        (WIRE_HISTORY_MAX, 1.0),
        (1.0, 4.0),
        (3.0, 4.0),
        (WIRE_HISTORY_MAX, 4.0),
        (8.0, 4.0),
        (12.0, 4.0),
        (24.0, 4.0),
    ] {
        let (avg, worst) = wire_spiral(h, w, 24.0, 8);
        println!(
            "{h:>9.1} {w:>7.1}  {avg:>10.3} {worst:>10.3}  {:>10}",
            if worst > 8.0 { "ESTOURA" } else { "sob o kill" }
        );
    }
    println!(
        "[wire] leitura: o teto é a maior janela cujo PIOR evento cabe no kill, com a espessura NO TETO."
    );
}

/// **O PIOR CASO DO WIRE É UM TRAÇO RETO, e a espiral o SUB-MEDE.**
///
/// ⚠️ Numa espiral o traço enrola sobre si mesmo, então uma corda de 1 152 px **de arco** liga dois
/// pontos que estão a ~200 px um do outro no canvas — e o que custa a rasterizar é o COMPRIMENTO da
/// corda, não o arco que ela pula. Num traço reto os dois coincidem, e é ali que a janela cobra o
/// preço cheio. Sem esta tabela o teto sairia de uma medição que o produto pode dobrar.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_the_wire_worst_case_on_a_straight_stroke() {
    let side = 2048u32;
    println!("[wire-reto] uma diagonal de ~2700 px, pincel r=24, 2048², pela porta do produto");
    println!(
        "{:>9} {:>7}  {:>10} {:>10}  {:>10}",
        "history", "width", "ms/evento", "pior ms", "veredito"
    );
    for &(h, w) in &[
        (1.0f32, 4.0f32),
        (3.0, 4.0),
        (6.0, 4.0),
        (12.0, 4.0),
        (24.0, 4.0),
        (48.0, 4.0),
    ] {
        let mut t = tool(side, PaintMedia::Digital, 24.0);
        t.paint.brush.stroke_method = StrokeMethod::Space;
        t.paint.brush.line_kind = LineKind::Wire;
        t.paint.brush.wire_history = h;
        t.paint.brush.thread_width_px = w;
        t.paint.brush.thread_opacity = 0.25;
        let steps = 320usize;
        #[allow(clippy::cast_precision_loss)]
        let pt = |i: usize| {
            let u = i as f32 / steps as f32;
            [60.0 + u * 1920.0, 60.0 + u * 1920.0]
        };
        t.on_canvas_pointer(cp(pt(0), PointerPhase::Down));
        let (mut total, mut worst) = (0.0f64, 0.0f64);
        for i in 1..=steps {
            let t0 = Instant::now();
            t.on_canvas_pointer(cp(pt(i), PointerPhase::Move));
            let ms = t0.elapsed().as_secs_f64() * 1e3;
            total += ms;
            worst = worst.max(ms);
        }
        t.on_canvas_pointer(cp(pt(steps), PointerPhase::Up));
        #[allow(clippy::cast_precision_loss)]
        let avg = total / steps as f64;
        println!(
            "{h:>9.1} {w:>7.1}  {avg:>10.3} {worst:>10.3}  {:>10}",
            if worst > 8.0 { "ESTOURA" } else { "sob o kill" }
        );
    }
}

/// **A CONTAGEM é livre de escala?** — e quanto ela custa, para o número não ser escolhido.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_the_wire_cost_across_curve_counts() {
    println!("[wire] o custo por corda: history no teto, width no teto, pincel r=24");
    println!("[wire] (a contagem é const; esta tabela é a LEI de como ela escala)");
    println!("{:>8}  {:>10} {:>10}", "history", "ms/evento", "pior ms");
    for h in [1.0f32, 2.0, 4.0, 6.0, 8.0] {
        let (avg, worst) = wire_spiral(h, 4.0, 24.0, 8);
        println!("{h:>8.1}  {avg:>10.3} {worst:>10.3}");
    }
    println!("[wire] o mesmo traço, pincéis diferentes (history no teto, width 1)");
    println!("{:>8}  {:>10} {:>10}", "raio", "ms/evento", "pior ms");
    for r in [6.0f32, 12.0, 24.0, 48.0, 96.0] {
        let (avg, worst) = wire_spiral(WIRE_HISTORY_MAX, 1.0, r, 8);
        println!("{r:>8.0}  {avg:>10.3} {worst:>10.3}");
    }
}
