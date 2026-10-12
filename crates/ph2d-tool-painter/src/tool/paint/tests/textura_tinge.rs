//! **Bug #51: a imagem no Shape com «Use Texture Colors» calava o Color e o Randomize Color.** A camada
//! pintava o RGB dela e descartava a cor do carimbo, e o desvio do Randomize era reconstruído de «Color →
//! cor do carimbo» — nulo num Color branco. A lei agora é `textura × tinta`, deslocada pelo desvio
//! SORTEADO do carimbo (`Dab::color_shift`). E o Randomize Color «a cada traço» mora aqui também: o mesmo
//! sorteio, uma vez por traço.

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
fn o_color_tinge_a_cor_da_textura() {
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
fn o_randomize_color_muda_a_textura_com_o_color_branco() {
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
fn associar_a_imagem_com_a_cor_da_textura_poe_o_color_a_branco() {
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

#[test]
fn ligar_a_cor_da_textura_com_a_imagem_ja_la_poe_o_color_a_branco() {
    let mut t = com_textura([200, 40, 40]);
    t.toggle_brush_shape_per_layer_color(); // desliga: o Color volta a ser a tinta do pincel
    t.paint.brush.color = [0.0, 0.0, 1.0];
    t.toggle_brush_shape_per_layer_color(); // liga: a textura passa a contar
    assert_eq!(t.paint.brush.color, [1.0, 1.0, 1.0]);
    // ⚠️ E o Color é a tinta do artista DAQUI em diante: escolhida depois, fica.
    t.paint.brush.color = [1.0, 0.0, 0.0];
    assert!(
        !t.brush_settings().color_inactive,
        "uma camada: o Color é a tinta, activo"
    );
}

/// Duas camadas com cor de textura («Per-Layer Color»): cada uma é dona da cor dela, a tinta é BRANCA e o
/// Color fica inactivo — e o traço pinta a textura como é, seja qual for o Color.
#[test]
fn em_per_layer_color_o_color_fica_inativo_e_nao_tinge() {
    let pinta = |color: [f32; 3]| {
        let mut t = white_canvas(64, 10.0);
        t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Space;
        t.paint.brush.falloff = Falloff::Constant;
        t.paint.brush.space_attenuation = false;
        let s = 16u32;
        let cheia = vec![255u8; (s * s) as usize];
        t.set_brush_shape_layers(vec![(cheia.clone(), s, s), (cheia, s, s)]);
        let cor = |rgb: [u8; 3]| -> Vec<u8> {
            std::iter::repeat_n(rgb, (s * s) as usize)
                .flatten()
                .collect()
        };
        t.paint.shape_layers.set_layers_meta(
            vec![cor([20, 200, 20]), cor([20, 200, 20])],
            vec![1.0, 1.0],
            vec![0, 0],
            vec![1, 2],
        );
        t.toggle_brush_shape_per_layer_color();
        assert!(
            t.brush_settings().color_inactive,
            "Per-Layer Color: o Color é inactivo"
        );
        t.paint.brush.color = color;
        t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));
        px(&t, 64, 32, 32)
    };
    let vermelho = pinta([1.0, 0.0, 0.0]);
    assert!(
        vermelho[1] > 180,
        "a textura verde pinta verde: {vermelho:?}"
    );
    assert_eq!(
        vermelho,
        pinta([0.0, 0.0, 1.0]),
        "o Color não age em Per-Layer Color"
    );
}

/// Um traço horizontal com Randomize Hue no máximo e «a cada traço»; devolve as 11 amostras.
fn traco_por_traco(t: &mut PainterTool, y: f32) -> Vec<[u8; 4]> {
    t.paint.brush.color_jitter_hue = 1.0;
    t.set_brush_color_jitter_scope(1);
    t.paint.brush.spacing = 0.6;
    t.on_canvas_pointer(cp([8.0, y], PointerPhase::Down));
    for x in (10..=56).step_by(2) {
        t.on_canvas_pointer(cp([x as f32, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([56.0, y], PointerPhase::Up));
    (12..=52)
        .step_by(4)
        .map(|x| px(t, 64, x, y as u32))
        .collect()
}

/// (a) «a cada traço»: todos os carimbos de um traço saem na MESMA cor, e o traço seguinte noutra.
#[test]
fn a_cada_traco_o_traco_inteiro_sai_numa_cor_e_o_seguinte_noutra() {
    let mut t = white_canvas(64, 6.0);
    t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Space;
    t.paint.brush.color = [1.0, 0.0, 0.0];
    let a = traco_por_traco(&mut t, 16.0);
    let b = traco_por_traco(&mut t, 48.0);
    assert!(a.iter().all(|p| *p == a[0]), "um traço, uma cor: {a:?}");
    assert!(b.iter().all(|p| *p == b[0]), "um traço, uma cor: {b:?}");
    assert_ne!(a[0], b[0], "o traço seguinte sai noutra cor");
}

/// (b) e com a textura do #51 (Color branco): o traço inteiro numa cor, o seguinte noutra.
#[test]
fn a_cada_traco_vale_tambem_para_a_cor_da_textura() {
    let mut t = com_textura([255, 0, 0]);
    t.paint.brush.radius_px = 6.0;
    t.paint.brush.color = [1.0, 1.0, 1.0];
    let a = traco_por_traco(&mut t, 16.0);
    let b = traco_por_traco(&mut t, 48.0);
    assert!(a.iter().all(|p| *p == a[0]), "um traço, uma cor: {a:?}");
    assert!(b.iter().all(|p| *p == b[0]), "um traço, uma cor: {b:?}");
    assert_ne!(a[0], b[0], "o traço seguinte sai noutra cor");
}

/// Um método que REFAZ o traço inteiro a cada Move (a Elipse: o editor re-carimba com a semente dele)
/// mantém a MESMA cor durante o gesto — a cor é do traço, não do Move.
#[test]
fn a_cada_traco_a_elipse_editada_mantem_a_cor() {
    let mut t = white_canvas(128, 3.0);
    t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Ellipse;
    t.set_shape_grab_tol_px(6.0);
    t.paint.brush.color = [1.0, 0.0, 0.0];
    t.paint.brush.color_jitter_hue = 1.0;
    t.set_brush_color_jitter_scope(1);
    t.on_canvas_pointer(cp([64.0, 64.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([84.0, 64.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([84.0, 64.0], PointerPhase::Up));
    // O topo do anel (64, 44) não sai do sítio quando o eixo horizontal cresce.
    let antes = px(&t, 128, 64, 44);
    assert_ne!(antes, [255, 255, 255, 255], "o anel passa no topo");
    t.on_canvas_pointer(cp([84.0, 64.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([94.0, 64.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([94.0, 64.0], PointerPhase::Up));
    assert_eq!(
        px(&t, 128, 64, 44),
        antes,
        "a elipse editada mantém a cor do traço"
    );
}

/// No Wet Paint a cor da textura não pinta (a tinta é o Color chapado): associar a imagem com «Use
/// Texture Colors» ligado NÃO apaga a tinta, e o Color não fica inactivo.
#[test]
fn no_wet_paint_a_imagem_nao_apaga_a_tinta() {
    let mut t = white_canvas(64, 10.0);
    t.set_paint_media(super::super::media::PaintMedia::WetPaint);
    assert_eq!(t.paint_media(), super::super::media::PaintMedia::WetPaint);
    t.paint.brush.color = [0.0, 0.0, 1.0];
    t.toggle_brush_shape_per_layer_color();
    let rgba: Vec<u8> = std::iter::repeat_n([200u8, 40, 40, 255], 64)
        .flatten()
        .collect();
    t.set_brush_shape_image_rgba(&rgba, 8, 8, None);
    assert_eq!(
        t.paint.brush.color,
        [0.0, 0.0, 1.0],
        "a tinta do Wet Paint fica"
    );
    assert!(!t.brush_settings().color_inactive);
}
