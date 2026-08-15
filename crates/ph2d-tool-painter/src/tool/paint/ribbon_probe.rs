//! **O QUE A FITA CUSTA POR EVENTO** — a medição que decide se os tetos do
//! `ph2d_painter_brush::line_kind::RIBBON_*` são de RECURSO ou de LOOK (plano 38 W6).
//!
//! ⚠️ **Pela porta do PRODUTO** (`on_canvas_pointer` + `on_tick`), nunca por um laço próprio: a
//! lição de que uma sonda com laço próprio fica CEGA à porta já custou duas waves a este módulo
//! (doc 28 §5.11, §5.46), e a de que *duas fixtures "grandes" diferentes dão números incomparáveis*
//! custou outra (§5.40).
//!
//! ⚠️ **E a fita é o PRIMEIRO tipo cujo custo NÃO se concentra no evento de movimento:** ela
//! percorre caminho no TIQUE, e a cauda dela cai toda no **pen-up**. Uma sonda que medisse só o
//! `Move` diria que a fita é de graça — e diria a verdade sobre o evento errado.
//!
//! Rodar: `cargo test -p ph2d-tool-painter --release measure_the_ribbon -- --ignored --nocapture --test-threads=1`

use super::measure_shape_system::{cp, tool};
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, Tool};
use ph2d_painter_brush::StrokeMethod;
use ph2d_painter_brush::line_kind::LineKind;
use std::time::Instant;

/// Um traço reto de `secs` a `speed` px/s, com o tique do produto entre os eventos.
/// Devolve `(pior Move ms, pior tique ms, pen-up ms)`.
fn straight(kind: LineKind, weight: f32, friction: f32, radius: f32, speed: f32) -> (f64, f64, f64) {
    let side = 2048u32;
    let dt = 1.0 / 60.0;
    let mut t = tool(side, PaintMedia::Digital, radius);
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.paint.brush.line_kind = kind;
    t.paint.brush.ribbon_weight = weight;
    t.paint.brush.ribbon_friction = friction;
    let y = f32::from(u16::try_from(side / 2).unwrap_or(u16::MAX));
    let mut x = 200.0f32;
    t.on_canvas_pointer(cp([x, y], PointerPhase::Down));
    let (mut worst_move, mut worst_tick) = (0.0f64, 0.0f64);
    for _ in 0..120 {
        x += speed * dt;
        let a = Instant::now();
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        worst_move = worst_move.max(a.elapsed().as_secs_f64() * 1e3);
        let b = Instant::now();
        t.on_tick(dt * 1e3);
        worst_tick = worst_tick.max(b.elapsed().as_secs_f64() * 1e3);
    }
    let c = Instant::now();
    t.on_canvas_pointer(cp([x, y], PointerPhase::Up));
    (worst_move, worst_tick, c.elapsed().as_secs_f64() * 1e3)
}

/// **O ORÇAMENTO DA FITA** — o pior Move, o pior tique e o PEN-UP, contra o kill de 8 ms.
///
/// ⚠️ **A primeira linha é o CONTROLE** (`None`, a fita desarmada): sem ela a tabela não diz se um
/// número é da fita ou do pincel.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_the_ribbon_budget_per_event() {
    println!("[ribbon] traco reto de 2 s a 2400 px/s, 2048^2, pela porta do produto");
    println!(
        "{:>9} {:>7} {:>8} {:>6}  {:>10} {:>10} {:>10} {:>10}",
        "tipo", "weight", "friction", "raio", "pior move", "pior tick", "pen-up", "veredito"
    );
    // A corrida de aquecimento paga o *first-touch* dos planos: a lição do doc 28 §5.13.
    let _ = straight(LineKind::None, 0.0, 0.0, 24.0, 2400.0);
    for radius in [24.0f32, 100.0, 200.0] {
        for (kind, w, f) in [
            (LineKind::None, 0.0, 0.0),
            (LineKind::Ribbon, 0.25, 0.30),
            (LineKind::Ribbon, 0.45, 0.30),
            (LineKind::Ribbon, 1.0, 0.30),
            (LineKind::Ribbon, 1.0, 0.05),
            (LineKind::Ribbon, 1.0, 1.0),
        ] {
            let (mv, tk, up) = straight(kind, w, f, radius, 2400.0);
            let worst = mv.max(tk).max(up);
            println!(
                "{:>9} {w:>7.2} {f:>8.2} {radius:>6.0}  {mv:>10.3} {tk:>10.3} {up:>10.3} {:>10}",
                format!("{kind:?}"),
                if worst > 8.0 { "ESTOURA" } else { "sob o kill" }
            );
        }
    }
    println!("[ribbon] leitura: o teto e o maior peso cujo PIOR evento cabe nos 8 ms.");
}
