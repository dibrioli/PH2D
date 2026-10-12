//! **Bug #51 (aberto): a imagem no Shape com «Use Texture Colors» cala o Color e o Randomize Color.** A
//! camada pinta o RGB dela e descarta a cor do carimbo (`accumulate.rs` / `accumulate_batch.rs` /
//! `stamp_color.rs`: `Some(img) => sample_shape_rgb_unit(..)`), e o desvio do Randomize é reconstruído de
//! «Color → cor do carimbo», nulo num Color branco. Estes três são os portões VERMELHOS da cura; quando
//! ela entrar sai o `ignore` e o nome perde o `diag_`.

use super::*;

/// Um Shape de UMA camada 16×16 cheia, com o RGB `rgb` capturado e «Use Texture Colors» ligado.
fn com_textura(rgb: [u8; 3]) -> PainterTool {
    let mut t = white_canvas(64, 10.0);
    t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Space;
    t.paint.brush.hardness = 1.0;
    t.paint.brush.falloff = Falloff::Constant;
    t.paint.brush.space_attenuation = false;
    let s = 16u32;
    t.set_brush_shape_layers(vec![(vec![255u8; (s * s) as usize], s, s)]);
    let cor = std::iter::repeat_n(rgb, (s * s) as usize)
        .flatten()
        .collect();
    t.paint
        .shape_layers
        .set_layers_meta(vec![cor], vec![1.0], vec![0], vec![1]);
    t.toggle_brush_shape_per_layer_color();
    assert!(t.paint.shape_layers.is_color_mode());
    t
}

#[test]
#[ignore = "Bug #51 aberto: o portão vermelho da próxima janela"]
fn diag_bug51_o_color_tinge_a_cor_da_textura() {
    // Textura BRANCA × Color vermelho = vermelho (o Color é a tinta, o branco não a muda).
    let mut t = com_textura([255, 255, 255]);
    t.paint.brush.color = [1.0, 0.0, 0.0];
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));
    let p = px(&t, 64, 32, 32);
    assert!(
        p[0] > 180 && p[1] < 80 && p[2] < 80,
        "textura branca × Color vermelho pinta vermelho: {p:?}"
    );
}

/// Um traço horizontal com Randomize Hue no máximo; devolve quantas das 11 amostras saíram do vermelho.
fn fora_do_vermelho(t: &mut PainterTool) -> (usize, Vec<[u8; 4]>) {
    t.paint.brush.color_jitter_hue = 1.0;
    t.paint.brush.spacing = 0.6;
    t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    for x in (10..=56).step_by(2) {
        t.on_canvas_pointer(cp([x as f32, 32.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up));
    let amostras: Vec<[u8; 4]> = (12..=52).step_by(4).map(|x| px(t, 64, x, 32)).collect();
    let n = amostras.iter().filter(|p| p[1] > 120 || p[2] > 120).count();
    (n, amostras)
}

/// O CONTROLO da régua de baixo: sem a cor da textura, o mesmo traço com Color vermelho varia de matiz.
#[test]
fn o_randomize_color_varia_a_silhueta_sem_a_cor_da_textura() {
    let mut t = com_textura([255, 0, 0]);
    t.toggle_brush_shape_per_layer_color();
    assert!(!t.paint.shape_layers.is_color_mode());
    t.paint.brush.color = [1.0, 0.0, 0.0];
    let (n, a) = fora_do_vermelho(&mut t);
    assert!(n >= 3, "o Randomize Color varia o matiz: {a:?}");
}

#[test]
#[ignore = "Bug #51 aberto: o portão vermelho da próxima janela"]
fn diag_bug51_o_randomize_color_muda_a_textura_com_o_color_branco() {
    // Textura VERMELHA, Color BRANCO (o de nascença): os carimbos têm de variar de matiz — o desvio
    // sorteado aplica-se à cor FINAL, não à cor do pincel (num branco o matiz não existe).
    let mut t = com_textura([255, 0, 0]);
    t.paint.brush.color = [1.0, 1.0, 1.0];
    let (n, a) = fora_do_vermelho(&mut t);
    assert!(
        n >= 3,
        "o Randomize Color tira a textura vermelha do vermelho em vários carimbos: {a:?}"
    );
}

#[test]
#[ignore = "Bug #51 aberto: o portão vermelho da próxima janela"]
fn diag_bug51_associar_a_imagem_com_a_cor_da_textura_poe_o_color_a_branco() {
    let mut t = white_canvas(64, 10.0);
    t.paint.brush.color = [0.0, 0.0, 1.0];
    t.toggle_brush_shape_per_layer_color();
    let rgba: Vec<u8> = std::iter::repeat_n([200u8, 40, 40, 255], 64)
        .flatten()
        .collect();
    t.set_brush_shape_image_rgba(&rgba, 8, 8, None);
    assert_eq!(
        t.paint.brush.color,
        [1.0, 1.0, 1.0],
        "com «Use Texture Colors» ligado, a imagem nova entra sem tinta (Color branco)"
    );
}
