//! **Os verbos que faltavam à seleção** (Enio, 2026-08-07: *"não temos 3 recursos importantes,
//! Expandir, contrair e CUT, copy e paste"*).
//!
//! O inventário medido antes de escrever qualquer linha: **Copy e Paste JÁ existiam e estavam
//! fiados**; **Expandir/Contrair também**, com quinas CAD, sob o título "OFFSET" — um nome que o
//! artista não procura. O que de fato faltava era **CUT**, o **Intersect** (a 4ª operação booleana do
//! padrão da indústria), o **Select All**, e **todo atalho de teclado** — o painel era a única porta.

use super::*;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use ph2d_painter_brush::Falloff;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Tela branca opaca com pincel preto duro — o mesmo vocabulário da suíte de seleção, local aqui
/// para não alargar a visibilidade de um fixture do arquivo vizinho.
fn white_canvas(size: u32, radius: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let b = BrushSpec {
        radius_px: radius,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.0, 0.0, 0.0],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t
}

fn alpha(t: &PainterTool, stride: u32, x: u32, y: u32) -> u8 {
    t.canvas_rgba[((y * stride + x) * 4 + 3) as usize]
}

/// Duas faixas verticais que se sobrepõem em `x ∈ [24, 40)`, desenhadas pelo GESTO real (o marquee
/// retangular), com `op` aplicado à segunda.
fn two_overlapping_bands(op: u8) -> PainterTool {
    let mut t = white_canvas(64, 4.0);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(2); // Rectangle
    t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([40.0, 56.0], PointerPhase::Up));
    t.set_selection_bool_op(op);
    t.on_canvas_pointer(cp([24.0, 8.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([56.0, 56.0], PointerPhase::Up));
    t
}

/// **Intersect guarda só o que as DUAS cobrem.** A 4ª operação booleana: New/Add/Remove existiam,
/// esta não — e sem ela *"a parte de cima daquela forma"* não é exprimível num gesto.
///
/// ⚠️ **O gate carrega o CONTROLE ao lado**: as mesmas duas faixas com **Add** têm de manter as três
/// zonas. Sem ele, um Intersect que apagasse tudo passaria — zero é subconjunto de qualquer coisa.
///
/// **Mutação que sangra:** o braço `3` do `combine_into` caindo no `_` (que COPIA a região) — a ponta
/// exclusiva da SEGUNDA faixa passaria a estar selecionada.
#[test]
fn intersect_keeps_only_what_both_cover() {
    let t = two_overlapping_bands(3); // Intersect
    assert!(
        t.selection_coverage_at(32, 32) > 128,
        "a faixa COMUM (x=32) continua selecionada"
    );
    assert_eq!(
        t.selection_coverage_at(12, 32),
        0,
        "a parte so da PRIMEIRA (x=12) saiu"
    );
    assert_eq!(
        t.selection_coverage_at(52, 32),
        0,
        "a parte so da SEGUNDA (x=52) nunca entrou"
    );
    // CONTROLE: com Add as tres zonas ficam — e e isso que prova que a fixture contem o fenomeno.
    let add = two_overlapping_bands(1);
    for x in [12u32, 32, 52] {
        assert!(
            add.selection_coverage_at(x, 32) > 128,
            "o controle (Add) tem de cobrir x={x}"
        );
    }
}

/// **Select All seleciona a tela inteira** — incluindo as quinas, que é onde um retângulo
/// mal-arredondado falharia.
///
/// **Mutação que sangra:** semear o crisp com `0` em vez de `255`.
#[test]
fn select_all_covers_every_corner_of_the_canvas() {
    let mut t = white_canvas(32, 4.0);
    t.selection_select_all();
    assert!(t.selection_active(), "ha selecao viva");
    for (x, y) in [(0u32, 0u32), (31, 0), (0, 31), (31, 31), (16, 16)] {
        assert_eq!(
            t.selection_coverage_at(x, y),
            255,
            "({x},{y}) tem de estar selecionado por completo"
        );
    }
}

/// **Cut leva os pixels E os apaga.** As duas metades num gate só: sem a primeira o Paste seguinte
/// não teria o que colar, sem a segunda o Cut seria um Copy com outro nome.
///
/// **Mutação que sangra:** trocar a metade que limpa por um no-op — o alfa fica em 255.
#[test]
fn cut_takes_the_pixels_and_clears_them() {
    let mut t = white_canvas(32, 4.0);
    t.set_rect_selection(0, 0, 16, 32);
    t.selection_color_fill(); // metade esquerda preta, para haver o que cortar
    t.selection_cut();
    assert_eq!(alpha(&t, 32, 4, 16), 0, "o que foi cortado saiu da tela");
    assert_eq!(
        alpha(&t, 32, 24, 16),
        255,
        "fora da selecao nada foi tocado"
    );
}

/// **Um Cut é UM passo de undo.** O Copy não grava undo (é leitura) e a limpeza grava — se as duas
/// metades gravassem, o artista desfaria um Cut pela metade.
///
/// **Mutação que sangra:** um segundo `commit_structural_edit` dentro do `selection_cut`.
#[test]
fn a_cut_is_one_undo_step() {
    let mut t = white_canvas(32, 4.0);
    t.set_rect_selection(0, 0, 16, 32);
    t.selection_color_fill();
    t.selection_cut();
    assert_eq!(alpha(&t, 32, 4, 16), 0, "cortado");
    assert!(t.undo_last(), "o Cut deixou um passo de undo");
    assert_eq!(
        alpha(&t, 32, 4, 16),
        255,
        "UM Ctrl+Z devolve o que o Cut levou"
    );
}

/// **Cut sem seleção não faz nada** — nem apaga a tela, nem enche o clipboard. O caso degenerado que
/// separa *"opera na seleção"* de *"opera na camada"*.
#[test]
fn cut_without_a_selection_is_a_no_op() {
    let mut t = white_canvas(32, 4.0);
    t.selection_cut();
    assert_eq!(
        alpha(&t, 32, 16, 16),
        255,
        "sem selecao o Cut nao pode limpar a camada inteira"
    );
}
