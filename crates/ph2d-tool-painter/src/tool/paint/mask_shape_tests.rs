//! B40.3/B40.4 (DF-45, DF-59d): as FORMAS no modo Mask protegem, e o Spacing do Blur/Smear não fica.

use super::mask_probe::{coverage, cp, mask_tool, vstroke};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, PanelEvent, PointerPhase, Tool};
use ph2d_painter_brush::StrokeMethod;

const S: u32 = 256;
const Y: f32 = 128.0;

fn click(t: &mut PainterTool, p: [f32; 2]) {
    t.on_canvas_pointer(cp(p, PointerPhase::Down));
    t.on_canvas_pointer(cp(p, PointerPhase::Up));
}

fn drag(t: &mut PainterTool, a: [f32; 2], b: [f32; 2]) {
    t.on_canvas_pointer(cp(a, PointerPhase::Down));
    for i in 1..=8 {
        let k = i as f32 / 8.0;
        t.on_canvas_pointer(cp(
            [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k],
            PointerPhase::Move,
        ));
    }
    t.on_canvas_pointer(cp(b, PointerPhase::Up));
}

/// Desenha a forma `method` atravessando `x = 128` em `y ≈ 128` e confirma-a.
fn shape(t: &mut PainterTool, method: StrokeMethod) {
    t.paint.brush.stroke_method = method;
    match method {
        StrokeMethod::Line => {
            click(t, [40.0, Y]);
            click(t, [216.0, Y]);
            assert!(t.line_commit(), "fixture: a linha confirma");
        }
        StrokeMethod::Ellipse => {
            drag(t, [128.0, Y], [188.0, Y]);
            assert!(t.ellipse_commit(), "fixture: a elipse confirma");
        }
        _ => {
            drag(t, [40.0, Y], [216.0, Y]);
            assert!(t.commit_open_shape(), "fixture: a curva confirma");
        }
    }
    let _ = t.take_preview_arc();
}

/// O ponto que a forma cobre: a linha e a curva em (128, 128); a elipse no lado esquerdo.
fn ponto(method: StrokeMethod, cov: &[f32]) -> (usize, usize) {
    match method {
        StrokeMethod::Ellipse => (68, 128),
        StrokeMethod::Line => (128, 128),
        // O Arc curva-se: o texel mais protegido da coluna do meio.
        _ => {
            let y = (0..S as usize)
                .max_by(|&a, &b| cov[a * S as usize + 128].total_cmp(&cov[b * S as usize + 128]))
                .unwrap_or(128);
            (128, y)
        }
    }
}

#[test]
fn uma_forma_no_modo_mask_protege_o_que_cobre() {
    for method in [StrokeMethod::Line, StrokeMethod::Ellipse, StrokeMethod::Arc] {
        let mut t = mask_tool(S);
        shape(&mut t, method);
        let cov = coverage(&t, S);
        let (x, y) = ponto(method, &cov);
        let c = cov[y * S as usize + x];
        assert!(
            c > 0.9,
            "{method:?}: a forma no modo Mask não protegeu nada (cobertura {c} em ({x},{y}))"
        );
        // A área protegida fica intacta quando o pincel normal passa por cima.
        let antes = t.canvas_rgba.clone();
        t.handle_panel_event(PanelEvent::SelectOption(
            ph2d_editor_core::ids::PAINTER_PAINT_MODE,
            "paint".to_string(),
        ));
        t.paint.brush.stroke_method = StrokeMethod::Space;
        t.paint.brush.color = [0.1, 0.9, 0.1];
        vstroke(&mut t, x as f32, y as f32 - 30.0, y as f32 + 30.0, 20);
        let i = (y * S as usize + x) * 4;
        assert_eq!(
            &t.canvas_rgba[i..i + 4],
            &antes[i..i + 4],
            "{method:?}: o pincel pintou por cima da área que a forma protegeu"
        );
        let j = ((y - 25) * S as usize + x) * 4;
        assert_ne!(
            &t.canvas_rgba[j..j + 4],
            &antes[j..j + 4],
            "{method:?}: controlo — fora da forma o pincel pinta"
        );
    }
}

/// Arrastar o 2.º ponto da linha por vários sítios não deixa rasto na máscara: o resultado é o de
/// pôr o ponto direto no destino.
#[test]
fn arrastar_a_forma_no_modo_mask_nao_deixa_rasto() {
    let mut direto = mask_tool(S);
    direto.paint.brush.stroke_method = StrokeMethod::Line;
    click(&mut direto, [40.0, Y]);
    click(&mut direto, [216.0, Y]);
    assert!(direto.line_commit());
    let mut arrastado = mask_tool(S);
    arrastado.paint.brush.stroke_method = StrokeMethod::Line;
    click(&mut arrastado, [40.0, Y]);
    click(&mut arrastado, [216.0, Y]);
    drag(&mut arrastado, [216.0, Y], [216.0, 40.0]);
    drag(&mut arrastado, [216.0, 40.0], [216.0, Y]);
    assert!(arrastado.line_commit());
    assert!(
        coverage(&direto, S)[128 * S as usize + 128] > 0.9,
        "fixture: a linha protege"
    );
    assert_eq!(
        coverage(&direto, S),
        coverage(&arrastado, S),
        "o arrasto deixou rasto na máscara"
    );
}

/// B40.4: Blur/Smear adensam o Spacing a 0,05; voltar a Paint/Erase devolve o do artista.
#[test]
fn entrar_e_sair_do_blur_da_mascara_devolve_o_spacing() {
    for sub in [2u8, 3] {
        for volta in [0u8, 1] {
            let mut t = mask_tool(S);
            t.paint.brush.spacing = 0.3;
            t.set_mask_brush(sub);
            assert!(
                (t.paint.brush.spacing - 0.05).abs() < 1e-6,
                "fixture: adensa"
            );
            t.set_mask_brush(5 - sub); // Blur ⇄ Smear mantém o denso
            assert!((t.paint.brush.spacing - 0.05).abs() < 1e-6);
            t.set_mask_brush(volta);
            assert!(
                (t.paint.brush.spacing - 0.3).abs() < 1e-6,
                "sair de {sub} para {volta} deixou o Spacing em {}",
                t.paint.brush.spacing
            );
        }
    }
}
