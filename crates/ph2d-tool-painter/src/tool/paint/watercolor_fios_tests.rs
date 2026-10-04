//! Os gates dos **FIOS na aguada** ([`super::watercolor_fios`], doc 46 §2-7): o Sketchy, o Wire e os
//! degraus do Ribbon chegam à tinta da aquarela.

use super::measure_shape_system::{cp, tool};
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_painter_brush::StrokeMethod;
use ph2d_painter_brush::line_kind::LineKind;

/// Quantos texels a aguada tingiu (o canal vermelho do papel branco desceu).
fn tingidos(t: &PainterTool) -> usize {
    t.canvas_rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[0] < 250)
        .count()
}

/// O zigue-zague dos gates digitais (`thread_deposit_tests`): as pernas mais longe que o rastro e mais
/// perto que o alcance, para a teia atravessar o vão entre elas.
fn zigzag(t: &mut PainterTool) {
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.on_canvas_pointer(cp([88.0, 128.0], PointerPhase::Down));
    for leg in 0..6 {
        #[allow(clippy::cast_precision_loss)]
        let x = 88.0 + (leg as f32) * 16.0;
        let up = if leg % 2 == 0 { 1.0 } else { -1.0 };
        for k in 1..=8 {
            #[allow(clippy::cast_precision_loss)]
            let y = 128.0 + up * (k as f32) * 4.0;
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        }
    }
    t.on_canvas_pointer(cp([160.0, 128.0], PointerPhase::Up));
}

fn aguada(arma: impl Fn(&mut PainterTool)) -> PainterTool {
    let mut t = tool(256, PaintMedia::Watercolor, 4.0);
    arma(&mut t);
    zigzag(&mut t);
    t
}

/// **O SKETCHY COSTURA A AGUADA** — o mesmo gesto com a teia armada tinge texels além do rastro.
#[test]
fn o_sketchy_tinge_a_aguada_alem_do_rastro() {
    let nu = tingidos(&aguada(|t| t.paint.brush.line_kind = LineKind::None));
    let teia = tingidos(&aguada(|t| {
        t.paint.brush.line_kind = LineKind::Sketchy;
        t.paint.brush.sketchy_reach = 3.0;
        t.paint.brush.sketchy_density = 0.4;
        t.paint.brush.thread_width_px = 1.0;
        t.paint.brush.thread_opacity = 0.5;
    }));
    assert!(nu > 200, "controlo: a aguada nua tem de pintar ({nu})");
    assert!(
        teia > nu + nu / 10,
        "a teia não chegou à aguada: {teia} texels contra {nu} do traço nu"
    );
}

/// **O WIRE sem `Connection Line` deixa SÓ o arame na aguada** — a supressão dos dabs é a mesma porta
/// do digital (`wire_suppresses_dabs`), e o arame é tinta.
#[test]
fn o_wire_sem_linha_deixa_so_o_arame_na_aguada() {
    let nu = tingidos(&aguada(|t| t.paint.brush.line_kind = LineKind::None));
    let arame = tingidos(&aguada(|t| {
        t.paint.brush.line_kind = LineKind::Wire;
        t.paint.brush.wire_connection_line = false;
        t.paint.brush.thread_opacity = 0.6;
    }));
    assert!(arame > 50, "o arame não chegou à aguada ({arame})");
    assert!(
        arame < nu,
        "o traço continuou a pintar sob o Wire sem linha ({arame} ≥ {nu})"
    );
}

/// **Os DEGRAUS do Ribbon chegam à aguada** — o número deles muda a tinta (o censo de 2026-10-03 leu
/// o Rungs morto na aquarela: o Ribbon agia, os degraus não).
///
/// ⚠️ A fita é uma MOLA no relógio (`ribbon_lag_s`): sem um tique por quadro ela não abre e não há
/// degraus. O gesto é o da sonda digital (`ribbon_probe`): uma onda, um evento grosso por quadro.
#[test]
fn os_degraus_do_ribbon_chegam_a_aguada() {
    use ph2d_editor_core::Tool as _;
    let fita = |rungs: f32| {
        let mut t = tool(512, PaintMedia::Watercolor, 4.0);
        t.paint.brush.stroke_method = StrokeMethod::Space;
        t.paint.brush.line_kind = LineKind::Ribbon;
        t.paint.brush.ribbon_weight = 0.08;
        t.paint.brush.ribbon_rungs = rungs;
        t.paint.brush.thread_opacity = 0.6;
        let pt = |u: f32| [20.0 + u * 472.0, 256.0 + (u * 18.0).sin() * 160.0];
        t.on_canvas_pointer(cp(pt(0.0), PointerPhase::Down));
        for k in 1..=40 {
            #[allow(clippy::cast_precision_loss)]
            t.on_canvas_pointer(cp(pt(k as f32 / 40.0), PointerPhase::Move));
            t.on_tick(1000.0 / 60.0);
        }
        t.on_canvas_pointer(cp(pt(1.0), PointerPhase::Up));
        t.canvas_rgba.to_vec()
    };
    let difere = |a: &[u8], b: &[u8]| a.chunks(4).zip(b.chunks(4)).filter(|(x, y)| x != y).count();
    let n = difere(&fita(0.0), &fita(1.0));
    assert!(n > 100, "os degraus não mudaram a aguada ({n} texels)");
}

/// **A teia não risca a aguada por dentro** — dentro da tinta molhada o fio deposita só pigmento: os
/// texels que o traço nu já tingia não escurecem com a teia (o aro, que realça toda fronteira da
/// cobertura, riscava-os de preto quando o fio subia a cobertura no ombro do traço).
#[test]
fn a_teia_nao_risca_a_aguada_por_dentro() {
    let nu = aguada(|t| t.paint.brush.line_kind = LineKind::None)
        .canvas_rgba
        .to_vec();
    let teia = aguada(|t| {
        t.paint.brush.line_kind = LineKind::Sketchy;
        t.paint.brush.sketchy_reach = 3.0;
        t.paint.brush.sketchy_density = 0.05;
        t.paint.brush.thread_width_px = 1.0;
        t.paint.brush.thread_opacity = 0.5;
    })
    .canvas_rgba
    .to_vec();
    // O MIOLO do traço nu: todo texel a 3 px tingido — a beira fica de fora, porque ali a orla do
    // traço muda de sítio quando a teia lhe encosta (é a fronteira que se move, não um risco).
    let lado = 256usize;
    let tingido = |x: usize, y: usize| nu[(y * lado + x) * 4] < 250;
    let miolo = |x: usize, y: usize| {
        (x.saturating_sub(3)..=(x + 3).min(lado - 1))
            .all(|xx| (y.saturating_sub(3)..=(y + 3).min(lado - 1)).all(|yy| tingido(xx, yy)))
    };
    let mut no_miolo = 0;
    let mut riscados = 0;
    for y in 0..lado {
        for x in 0..lado {
            if !miolo(x, y) {
                continue;
            }
            no_miolo += 1;
            let i = (y * lado + x) * 4;
            if i32::from(teia[i]) + 40 < i32::from(nu[i]) {
                riscados += 1;
            }
        }
    }
    assert!(
        no_miolo > 300,
        "controlo: o traço nu tem miolo ({no_miolo})"
    );
    assert_eq!(
        riscados, 0,
        "a teia riscou {riscados} texels do miolo da aguada (de {no_miolo})"
    );
}
