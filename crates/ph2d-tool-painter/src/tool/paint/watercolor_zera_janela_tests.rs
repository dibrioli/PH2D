//! B44.14 (N10 F8): a figura de aguada (Drag Dot) refaz a cobertura a cada quadro e zerava os planos de TELA
//! INTEIRA — memset proporcional à tela, não à figura. A cura zera só a `wet_cum_dirty`; aqui: o contador
//! `bytes_zerados` por quadro não cresce com a tela, e os planos e a tela saem AO BIT do caminho de antes.

use super::diag_auditoria_da_pilha::cp;
use super::watercolor_accum::{bytes_zerados, forca_zerar_a_janela};
use super::*;

const QUADROS: usize = 12;

fn cena(w: u32, h: u32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
    t.paint.composite_enabled = false;
    t.paint.brush.watercolor = true;
    t.paint.brush.radius_px = 60.0;
    t.paint.brush.edge_spread = 7.0;
    t.paint.brush.wet_charge = 0.5; // o mixer armado: os planos de reserva vivem
    t.paint.brush.color = [0.15, 0.35, 0.75];
    t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::DragDot;
    t
}

/// Um Drag Dot que anda `QUADROS` quadros; devolve (bytes zerados por quadro de movimento, a ferramenta).
fn drag_dot(w: u32, h: u32, janela: bool) -> (u64, PainterTool) {
    let (b, _, t) = drag_dot_cronometrado(w, h, janela);
    (b, t)
}

/// O mesmo, com o tempo MÍNIMO de um quadro de movimento (ms).
fn drag_dot_cronometrado(w: u32, h: u32, janela: bool) -> (u64, f64, PainterTool) {
    forca_zerar_a_janela(Some(janela));
    let mut t = cena(w, h);
    let c = [512.0f32, 512.0];
    t.on_canvas_pointer(cp(c, PointerPhase::Down));
    let antes = bytes_zerados();
    let mut minimo = f64::MAX;
    for k in 0..QUADROS {
        let t0 = std::time::Instant::now();
        t.on_canvas_pointer(cp(
            [c[0] + 8.0 * k as f32, c[1] + 3.0 * k as f32],
            PointerPhase::Move,
        ));
        minimo = minimo.min(t0.elapsed().as_secs_f64() * 1e3);
    }
    let por_quadro = (bytes_zerados() - antes) / QUADROS as u64;
    t.on_canvas_pointer(cp([c[0] + 96.0, c[1] + 36.0], PointerPhase::Up));
    forca_zerar_a_janela(None);
    (por_quadro, minimo, t)
}

fn planos(t: &PainterTool) -> [Vec<u8>; 6] {
    let p = &t.paint;
    [
        p.stroke_coverage.clone(),
        p.stroke_density.clone(),
        p.stroke_deplete.clone(),
        p.stroke_deplete_prox.clone(),
        p.stroke_color.clone(),
        t.canvas_rgba.to_vec(),
    ]
}

#[test]
fn a_figura_de_aguada_zera_a_janela_e_nao_a_tela() {
    let (pequena, _) = drag_dot(1024, 1024, true);
    let (grande, _) = drag_dot(3840, 2160, true);
    let (antes, _) = drag_dot(3840, 2160, false);
    eprintln!(
        "[B44.14] bytes zerados/quadro: 1024² {pequena} · 4K {grande} · 4K tela inteira (antes) {antes}"
    );
    // A×B intercalado no MESMO processo, o mínimo de 3 rodadas (régua informativa, sem limiar de relógio).
    let (mut ms_a, mut ms_b) = (f64::MAX, f64::MAX);
    for _ in 0..3 {
        ms_a = ms_a.min(drag_dot_cronometrado(3840, 2160, false).1);
        ms_b = ms_b.min(drag_dot_cronometrado(3840, 2160, true).1);
    }
    eprintln!(
        "[B44.14] quadro Drag Dot 4K (mín.): tela inteira {ms_a:.2} ms · janela {ms_b:.2} ms"
    );
    assert!(pequena > 0, "a fixtura tem de zerar algo (população)");
    assert!(
        antes >= 5 * (3840 * 2160) as u64,
        "o controlo (tela inteira) tem de zerar ≥ 5 planos de 4K por quadro: {antes}"
    );
    assert!(
        grande as f64 <= 1.02 * pequena as f64,
        "a 4K a figura zera {grande} B/quadro contra {pequena} a 1024² — o memset segue a TELA"
    );
}

#[test]
fn zerar_a_janela_da_os_mesmos_planos_e_a_mesma_tela_que_a_tela_inteira() {
    let (_, a) = drag_dot(1024, 1024, true);
    let (_, b) = drag_dot(1024, 1024, false);
    let (pa, pb) = (planos(&a), planos(&b));
    assert!(
        pa[0].iter().any(|&v| v > 0),
        "a figura tem de pintar cobertura (população)"
    );
    assert!(
        !pa[2].is_empty(),
        "o mixer armado tem de criar os planos de reserva"
    );
    for (k, (x, y)) in pa.iter().zip(&pb).enumerate() {
        let d = x.iter().zip(y).filter(|(u, v)| u != v).count();
        assert_eq!(
            (x.len(), d),
            (y.len(), 0),
            "plano {k}: a janela difere da tela inteira em {d} bytes"
        );
    }
}
