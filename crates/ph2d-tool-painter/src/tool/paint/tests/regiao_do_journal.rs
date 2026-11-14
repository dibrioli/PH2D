//! B17.6 (missão 4K): o Blur e o carimbo de cor ASSADO capturam só a região do lote, não a tela.
//!
//! Com `fork_canvas(…, None)` o journal guardava o plano inteiro por lote (33 MB a 4K). A região
//! tem de ser superconjunto do que o lote escreve, com as cópias do Tiling, ou o undo perde a borda.

use super::super::media::PaintMedia;
use super::*;
use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
use ph2d_editor_core::tool::CanvasPaintTool;
use ph2d_painter_brush::texture::{TextureKind, TextureMapping};

const LADO: u32 = 512;

/// Listras de 8 px (o Blur precisa de bordas para mexer) e o pincel duro de raio 10, Tiling ligado.
fn tela() -> PainterTool {
    let mut t = PainterTool::default();
    let mut src = vec![255u8; (LADO * LADO * 4) as usize];
    for (i, p) in src.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        if (i % LADO as usize / 8).is_multiple_of(2) {
            p[..3].copy_from_slice(&[20, 40, 200]);
        }
    }
    t.set_source(src, LADO, LADO);
    t.set_paint_media(PaintMedia::Digital);
    t.paint.tiling = [true, true];
    t
}

fn pincel(t: &mut PainterTool) {
    t.paint.brush.radius_px = 10.0;
    t.paint.brush.hardness = 1.0;
    t.paint.brush.falloff = Falloff::Constant;
    t.paint.brush.strength = 1.0;
    t.paint.brush.space_attenuation = false;
    t.paint.brush.spacing = 0.3;
    let seed = t.paint.brush;
    t.paint.brush_by_mode.fill(seed);
}

/// Um salto colado à borda esquerda (as cópias do Tiling caem à direita), o traço ABERTO; depois os
/// controlos, a região do journal e o undo ao byte.
fn exige_a_regiao(nome: &str, mut t: PainterTool) {
    let before: Vec<u8> = t.canvas_rgba.to_vec();
    t.on_canvas_pointer(cp([4.0, 200.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([4.0, 260.0], PointerPhase::Move));
    let (mut changed, mut undescribed, mut wrapped) = (0usize, 0usize, 0usize);
    for (i, (&a, &b)) in before.iter().zip(t.canvas_rgba.iter()).enumerate() {
        if a == b {
            continue;
        }
        changed += 1;
        if t.undo.write_state.canvas_before(i) != Some(a) {
            undescribed += 1;
        }
        if (i / 4) % LADO as usize > LADO as usize * 3 / 4 {
            wrapped += 1;
        }
    }
    assert!(
        changed > 100,
        "{nome}: controlo, o traço mal mexeu ({changed})"
    );
    assert!(
        wrapped > 0,
        "{nome}: controlo, o Tiling não copiou para a borda oposta"
    );
    assert_eq!(
        undescribed, 0,
        "{nome}: {undescribed}/{changed} texels fora do journal"
    );
    // A região é UM retângulo: com a cópia do Tiling ele abarca a largura toda (≈ 1/4 desta tela);
    // `None` retinha a tela inteira.
    let retido = t.undo.write_state.journal_heap_bytes();
    assert!(
        retido * 2 < before.len(),
        "{nome}: o journal reteve {retido} B de uma tela de {} B (a tela inteira)",
        before.len()
    );
    t.on_canvas_pointer(cp([4.0, 260.0], PointerPhase::Up));
    assert!(t.undo_last(), "{nome}: sem passo de undo");
    assert!(
        t.canvas_rgba[..] == before[..],
        "{nome}: o undo não devolveu a tela ao byte"
    );
}

/// As três rotas do Blur (falloff redondo, máscara em cache, grão por pixel) partilham um fork.
#[test]
fn o_blur_captura_so_a_regiao_do_lote() {
    for (nome, forma) in [
        ("blur redondo", None),
        ("blur máscara", Some(TextureMapping::ViewPlane)),
        ("blur grão", Some(TextureMapping::Tiled)),
    ] {
        let mut t = tela();
        t.set_paint_tool_mode("blur");
        if let Some(m) = forma {
            t.set_brush_shape_kind(TextureKind::Checker.to_u8());
            t.paint.brush.shape.mapping = m;
        }
        pincel(&mut t);
        assert!(matches!(t.paint.paint_mode, PaintMode::Blur));
        exige_a_regiao(nome, t);
    }
}

/// O Clone (redondo, máscara em cache, grão por pixel): a fonte é lida da própria tela, a escrita é a pegada do lote.
#[test]
fn o_clone_captura_so_a_regiao_do_lote() {
    for (nome, forma) in [
        ("clone redondo", None),
        ("clone máscara", Some(TextureMapping::ViewPlane)),
        ("clone grão", Some(TextureMapping::Tiled)),
    ] {
        let mut t = tela();
        t.set_paint_tool_mode("clone");
        if let Some(m) = forma {
            t.set_brush_shape_kind(TextureKind::Checker.to_u8());
            t.paint.brush.shape.mapping = m;
        }
        pincel(&mut t);
        t.set_clone_source([9.0, 200.0]); // a fonte 5 px à direita do toque
        assert!(matches!(t.paint.paint_mode, PaintMode::Clone));
        exige_a_regiao(nome, t);
    }
}

/// O carimbo de cor assado (Texture Color sobre o xadrez em View): a rota `stamp_dabs_cached_ramp_color`.
#[test]
fn o_carimbo_de_cor_assado_captura_so_a_regiao_do_lote() {
    let rampa = ColorRamp::new(
        vec![
            RampStop::new(0.0, [0.0, 1.0, 0.0, 1.0]),
            RampStop::new(1.0, [1.0, 0.0, 0.0, 1.0]),
        ],
        RampColorMode::Rgb,
        RampInterp::Linear,
    );
    let mut t = tela();
    t.set_brush_shape_kind(TextureKind::Checker.to_u8());
    t.paint.brush.shape.mapping = TextureMapping::ViewPlane;
    t.set_shape_color_ramp(rampa.clone());
    t.set_shape_falloff_ramp(rampa.clone());
    t.set_shape_ramp_enabled(true);
    t.paint.shape_color_ramp_bw = false;
    t.paint.shape_camadas.textura = true;
    t.paint.shape_camadas.queda = false;
    t.set_shape_falloff_ramp(rampa);
    pincel(&mut t);
    t.on_canvas_pointer(cp([300.0, 300.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([300.0, 300.0], PointerPhase::Up));
    assert!(
        t.paint.ramp_color_stamp_cache.is_some() && t.paint.stamp_cache.is_none(),
        "controlo: o traço não foi pela rota do carimbo de cor assado"
    );
    exige_a_regiao("carimbo de cor assado", t);
}
