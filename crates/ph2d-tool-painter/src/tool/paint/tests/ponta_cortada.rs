//! **A ponta da Shape cortada com Jitter Rotate** (smoke do dono 12/10). Sonda: um carimbo de uma imagem
//! QUADRADA cheia, pela rota real (`stamp_dabs`), rodado e com Size; mede até onde a tinta chega.

use super::*;
use ph2d_painter_brush::{Dab, StrokeMethod};

/// Um Shape de imagem 16×16 cheia (alpha 255, RGB cinza), raio 10, numa tela 96².
fn quadrado(cor_da_textura: bool, size: f32) -> PainterTool {
    let mut t = white_canvas(96, 10.0);
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.paint.brush.color = [0.0, 0.0, 0.0];
    t.paint.brush.shape.size = [size, size];
    let s = 16u32;
    let rgba: Vec<u8> = std::iter::repeat_n([40u8, 40, 40, 255], (s * s) as usize)
        .flatten()
        .collect();
    t.set_brush_shape_image_rgba(&rgba, s, s, None);
    // A silhueta pelo ALPHA (o quadrado cheio), não pela luminância.
    t.toggle_brush_shape_alpha_from_image();
    if cor_da_textura {
        t.toggle_brush_shape_per_layer_color();
        t.paint.brush.color = [1.0, 1.0, 1.0];
    }
    t
}

/// Carimba UM dab rodado de `graus` no centro e devolve `(meia-largura x, meia-altura y, píxeis)`.
fn mede(t: &mut PainterTool, graus: f32, dinamica: bool) -> (i32, i32, usize) {
    let r = graus.to_radians();
    if dinamica {
        t.paint.brush.jitter_rotate = 1.0; // a rota dinâmica (rotação por dab)
    }
    let d = Dab {
        center: [48.0, 48.0],
        radius_px: 10.0,
        coverage: 1.0,
        color: t.paint.brush.color,
        color_shift: [0.0; 3],
        rotation: [r.cos(), r.sin()],
        dir: [0.0, 0.0],
        arc_len: 0.0,
        stroke_radius_px: 10.0,
    };
    t.stamp_dabs(&[d]);
    let (mut ex, mut ey, mut n) = (0i32, 0i32, 0usize);
    for y in 0..96u32 {
        for x in 0..96u32 {
            if px(t, 96, x, y)[0] < 200 {
                n += 1;
                ex = ex.max((x as i32 - 48).abs());
                ey = ey.max((y as i32 - 48).abs());
            }
        }
    }
    (ex, ey, n)
}

#[test]
#[ignore = "sonda: a tabela do corte"]
fn sonda_ponta_cortada() {
    for cor in [false, true] {
        for size in [1.0f32, 0.6] {
            for graus in [0.0f32, 30.0, 45.0, -1.0] {
                let mut t = quadrado(cor, size);
                // `-1` = SEM Jitter Rotate (a rota do carimbo cacheado), 0°.
                let cache = graus < 0.0;
                let (ex, ey, n) = mede(&mut t, graus.max(0.0), !cache);
                // Sem corte: lado 2r/size, área (2r/size)², meia-diagonal r·√2/size.
                let lado = 20.0 / size;
                println!(
                    "cor_textura={cor} size={size} {graus:>4}° cache={cache} → meia-larg {ex} meia-alt {ey} píxeis {n} (esperado ≈ {:.0}, alcance a 45° ≈ {:.1})",
                    lado * lado,
                    lado * 0.5 * std::f32::consts::SQRT_2
                );
            }
        }
    }
}

/// BUGS #52 (aberto): os portões VERMELHOS da cura — quando ela entrar sai o `ignore` e o `diag_`.
/// A ponta rodada a 45° chega à meia-diagonal (≈ 14,1 px) e conserva a área (400 px). Hoje: ±10, 356.
#[test]
#[ignore = "BUGS #52 aberto: o portão vermelho da próxima janela"]
fn diag_bug52_a_ponta_rodada_nao_e_cortada() {
    for cor in [false, true] {
        let (ex, ey, n) = mede(&mut quadrado(cor, 1.0), 45.0, true);
        assert!(ex >= 13 && ey >= 13 && n >= 380, "cor={cor}: {ex} {ey} {n}");
    }
}

/// Size < 1 aumenta a ponta além do raio: 0,6 ⇒ lado 33 px (≈ 1111 px). Hoje: cortada ao quadrado da caixa
/// (441 na rota dinâmica, 400 na do carimbo cacheado — duas respostas para a mesma ponta).
#[test]
#[ignore = "BUGS #52 aberto: o portão vermelho da próxima janela"]
fn diag_bug52_size_menor_que_um_nao_corta_a_ponta_nas_duas_rotas() {
    for cor in [false, true] {
        let (_, _, dinamica) = mede(&mut quadrado(cor, 0.6), 0.0, true);
        let (_, _, cache) = mede(&mut quadrado(cor, 0.6), 0.0, false);
        assert!(
            dinamica >= 1000 && cache >= 1000,
            "cor={cor}: dinâmica {dinamica} · cache {cache}"
        );
    }
}
