//! **O Blend no Wet Paint** (doc 46 §1-§2, item 6): a tinta molhada é uma CAMADA, e o modo de Blend
//! entra onde ela se compõe sobre a tela (`composite.rs`). O modo é da SESSÃO — congelado quando ela
//! nasce; trocá-lo com tinta molhada na tela fixa a sessão antes do traço seguinte.

use super::*;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use ph2d_painter_brush::{BrushBlend, Falloff};

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Uma tela VERMELHA opaca e o pincel da água com a cor `cor`, no modo `blend`.
fn tela_vermelha(cor: [f32; 3], blend: BrushBlend) -> PainterTool {
    let mut t = PainterTool::default();
    let mut px = vec![0u8; 200 * 120 * 4];
    for p in px.as_chunks_mut::<4>().0 {
        p.copy_from_slice(&[200, 60, 40, 255]);
    }
    t.set_source(px, 200, 120);
    let b = BrushSpec {
        radius_px: 10.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: cor,
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.set_paint_tool_mode("wetpaint");
    t.set_brush_blend(blend as u8);
    t
}

fn risca(t: &mut PainterTool, y: f32) {
    t.on_canvas_pointer(cp([30.0, y], PointerPhase::Down));
    for k in 1..=20 {
        t.on_canvas_pointer(cp([30.0 + 7.0 * k as f32, y], PointerPhase::Move));
        t.paint_tick(1.0 / 60.0);
    }
    t.on_canvas_pointer(cp([170.0, y], PointerPhase::Up));
    for _ in 0..10 {
        t.paint_tick(1.0 / 60.0);
    }
}

/// `(texels que mudaram, |Δ| máximo)` contra a tela vermelha de partida.
fn mudanca(t: &PainterTool) -> (usize, u8) {
    let mut n = 0;
    let mut max = 0u8;
    for p in t.canvas_rgba.as_chunks::<4>().0 {
        let d = [
            p[0].abs_diff(200),
            p[1].abs_diff(60),
            p[2].abs_diff(40),
            p[3].abs_diff(255),
        ];
        let m = d.into_iter().max().unwrap_or(0);
        n += usize::from(m > 0);
        max = max.max(m);
    }
    (n, max)
}

/// ⭐⭐⭐ **Tinta BRANCA em Multiply (e PRETA em Screen) não pinta**: é a identidade do modo. Em Mix a
/// mesma tinta cobre o vermelho. Mutação que sangra: o composite ignorar o modo da sessão.
#[test]
fn a_identidade_do_modo_nao_pinta_sobre_a_tela() {
    let mut mix = tela_vermelha([1.0, 1.0, 1.0], BrushBlend::Mix);
    risca(&mut mix, 60.0);
    let (n_mix, max_mix) = mudanca(&mix);
    assert!(
        n_mix > 500 && max_mix > 60,
        "a régua: em Mix a tinta branca cobre o vermelho ({n_mix}, Δ {max_mix})"
    );
    for (cor, modo) in [
        ([1.0, 1.0, 1.0], BrushBlend::Multiply),
        ([0.0, 0.0, 0.0], BrushBlend::Screen),
    ] {
        let mut t = tela_vermelha(cor, modo);
        risca(&mut t, 60.0);
        let (n, max) = mudanca(&t);
        assert!(
            max <= 3,
            "{modo:?} com a tinta que é a identidade dele mudou {n} texels até Δ {max} — o Blend não chega à água"
        );
    }
}

/// ⭐⭐ **O modo é da SESSÃO.** Trocar o Blend com tinta molhada na tela fixa a sessão (a tinta fica
/// como está, ao byte) e o traço seguinte nasce numa sessão nova, no modo novo. Mutação que sangra:
/// o setter não fechar a sessão (a tinta já pintada re-compor-se-ia no modo novo).
#[test]
fn trocar_o_blend_com_tinta_molhada_fixa_a_sessao() {
    let mut t = tela_vermelha([0.1, 0.2, 0.9], BrushBlend::Mix);
    risca(&mut t, 40.0);
    assert!(
        t.paint.wetpaint.session.is_some(),
        "a régua: a sessão molhada está viva"
    );
    let antes = t.canvas_rgba.as_ref().clone();
    t.set_brush_blend(BrushBlend::Multiply as u8);
    assert!(
        t.paint.wetpaint.session.is_none(),
        "trocar o modo com tinta na tela fecha a sessão"
    );
    assert_eq!(
        *t.canvas_rgba, antes,
        "fechar a sessão não muda um byte da tinta pintada"
    );
    risca(&mut t, 90.0);
    let s = t
        .paint
        .wetpaint
        .session
        .as_ref()
        .expect("o traço novo abriu uma sessão");
    assert_eq!(
        s.blend,
        BrushBlend::Multiply,
        "a sessão nova nasce no modo novo"
    );
}
