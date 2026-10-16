//! **A ponta da Shape cortada com Jitter Rotate** (smoke do dono 12/10, BUGS #52). Um carimbo de uma imagem
//! QUADRADA cheia, pela rota real (`stamp_dabs`), rodado e com Size; mede até onde a tinta chega. A ponta é
//! `|tex| ≤ 1` no referencial DELA — rodada chega à meia-diagonal, com Size < 1 passa do raio.

use super::*;
use ph2d_painter_brush::{Dab, StrokeMethod};

/// Um Shape de imagem 16×16 cheia (alpha 255, RGB cinza), raio 10, numa tela 96².
fn quadrado(cor_da_textura: bool, size: f32) -> PainterTool {
    quadrado_em(96, cor_da_textura, size)
}

fn quadrado_em(n: u32, cor_da_textura: bool, size: f32) -> PainterTool {
    let mut t = white_canvas(n, 10.0);
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
    let mut r = graus.to_radians();
    if dinamica {
        t.paint.brush.jitter_rotate = 1.0; // a rota dinâmica (rotação por dab)
    } else {
        // A rota do carimbo cacheado: o ângulo é o Angle ESTÁTICO da Shape, não o do dab.
        t.paint.brush.shape.angle_deg = graus as u16;
        r = 0.0;
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
        step: [0.0, 0.0],
        ante: None,
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
            for (graus, cache) in [
                (0.0f32, false),
                (30.0, false),
                (45.0, false),
                (0.0, true),
                (45.0, true),
            ] {
                let mut t = quadrado(cor, size);
                // `cache` = SEM Jitter Rotate: a rota do carimbo cacheado, rodada pelo Angle da Shape.
                let (ex, ey, n) = mede(&mut t, graus, !cache);
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

/// BUGS #52: a ponta rodada a 45° chega à meia-diagonal (≈ 14,1 px) e conserva a área (400 px) — pelo Jitter
/// Rotate (rota dinâmica) e pelo Angle estático da Shape (carimbo cacheado). Antes da cura: ±10, 356 nas duas.
#[test]
fn a_ponta_rodada_nao_e_cortada() {
    for cor in [false, true] {
        for cache in [false, true] {
            let (ex, ey, n) = mede(&mut quadrado(cor, 1.0), 45.0, !cache);
            assert!(
                // A orla bilinear da aresta diagonal soma ~20–36 px ao quadrado de 400.
                ex >= 13 && ey >= 13 && (380..=450).contains(&n),
                "cor={cor} cache={cache}: {ex} {ey} {n}"
            );
        }
    }
}

/// Size < 1 aumenta a ponta além do raio: 0,6 ⇒ lado 33 px (≈ 1111 px), e as DUAS rotas dão a mesma ponta.
/// Antes da cura: 441 na dinâmica, 400 na do carimbo cacheado — duas respostas, as duas cortadas.
#[test]
fn size_menor_que_um_nao_corta_a_ponta_nas_duas_rotas() {
    for cor in [false, true] {
        let (dx, dy, dinamica) = mede(&mut quadrado(cor, 0.6), 0.0, true);
        let (cx, cy, cache) = mede(&mut quadrado(cor, 0.6), 0.0, false);
        assert!(
            (1050..=1180).contains(&dinamica) && (1050..=1180).contains(&cache),
            "cor={cor}: dinâmica {dinamica} · cache {cache}"
        );
        assert!(
            dx >= 16 && dy >= 16 && cx >= 16 && cy >= 16,
            "cor={cor}: alcance {dx}/{dy} · {cx}/{cy} (meia-largura ≈ 16,7)"
        );
    }
}

/// **O CONTROLO do ao-bit:** a 0°, Size 1, a ponta É a caixa — 400 px, meia-largura 10, nas duas rotas.
#[test]
fn a_zero_graus_size_um_a_ponta_e_a_caixa() {
    for cor in [false, true] {
        for cache in [false, true] {
            let (ex, ey, n) = mede(&mut quadrado(cor, 1.0), 0.0, !cache);
            assert_eq!((ex, ey, n), (10, 10, 400), "cor={cor} cache={cache}");
        }
    }
}

/// **Re-carimbo sem rasto (a lição do #24):** uma Linha com a ponta de imagem rodada e de Size < 1, o
/// extremo arrastado à volta — cada Move descasca o traço anterior pela região salva. Uma região do tamanho
/// do RAIO deixa a orla da ponta (entre `r` e o alcance dela) por restaurar.
#[test]
fn a_linha_com_a_ponta_alargada_editada_nao_deixa_rasto() {
    for cor in [false, true] {
        let mut t = quadrado(cor, 0.6);
        t.paint.brush.radius_px = 6.0;
        t.paint.brush.shape.angle_deg = 30;
        t.paint.brush.stroke_method = StrokeMethod::Line;
        let a = [14.0, 48.0];
        t.on_canvas_pointer(cp(a, PointerPhase::Down));
        t.on_canvas_pointer(cp(a, PointerPhase::Up));
        t.on_canvas_pointer(cp([80.0, 14.0], PointerPhase::Down));
        for b in [[80.0, 82.0], [80.0, 48.0], [24.0, 48.0]] {
            t.on_canvas_pointer(cp(b, PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([24.0, 48.0], PointerPhase::Up));
        // O traço final é o segmento (14..24, 48) com a ponta de alcance ≤ 6·√2/0,6 ≈ 14 px.
        let (mut perto, mut rasto) = (0usize, Vec::new());
        for y in 0..96u32 {
            for x in 0..96u32 {
                let pintado = px(&t, 96, x, y) != [255, 255, 255, 255];
                let longe = !(32..=64).contains(&y) || x > 40;
                if pintado && longe {
                    rasto.push((x, y));
                } else if pintado {
                    perto += 1;
                }
            }
        }
        assert!(
            perto > 200,
            "cor={cor}: CONTROLO — o traço final tem de pintar ({perto})"
        );
        assert!(
            rasto.is_empty(),
            "cor={cor}: rasto em {} px, ex. {:?}",
            rasto.len(),
            &rasto[..rasto.len().min(8)]
        );
    }
}

/// **Desfazer apaga a ponta INTEIRA** — o diário de desfazer guarda os blocos de 128 px que a região
/// declarada (`region::dabs_bounds`) toca, ANTES de os escrever. O carimbo mora a 12 px da fronteira de um
/// bloco: com a região do tamanho do RAIO (10) o bloco vizinho não era guardado e a orla da ponta
/// (até ~24 px) ficava pintada depois do Ctrl+Z.
#[test]
fn desfazer_apaga_a_ponta_alargada_inteira() {
    for cor in [false, true] {
        for jitter in [false, true] {
            let mut t = quadrado_em(256, cor, 0.6);
            t.paint.brush.shape.angle_deg = 30;
            if jitter {
                t.paint.brush.jitter_rotate = 1.0;
            }
            let antes = (*t.canvas_rgba).clone();
            t.on_canvas_pointer(cp([116.0, 116.0], PointerPhase::Down));
            t.on_canvas_pointer(cp([116.0, 116.0], PointerPhase::Up));
            assert_ne!(
                *t.canvas_rgba, antes,
                "cor={cor} jitter={jitter}: CONTROLO — pintou"
            );
            assert!(
                t.undo_last(),
                "cor={cor} jitter={jitter}: um traço, uma entrada"
            );
            let sobra = t
                .canvas_rgba
                .chunks(4)
                .zip(antes.chunks(4))
                .filter(|(a, b)| a != b)
                .count();
            assert_eq!(
                sobra, 0,
                "cor={cor} jitter={jitter}: {sobra} px ficaram depois do desfazer"
            );
        }
    }
}

/// **O Drag Dot com a ponta alargada não deixa rasto** — o ponto que segue o cursor descasca a posição
/// anterior pela região salva (`caixa_do_lote`).
#[test]
fn o_drag_dot_com_a_ponta_alargada_nao_deixa_rasto() {
    for cor in [false, true] {
        let mut t = quadrado(cor, 0.6);
        t.paint.brush.shape.angle_deg = 30;
        t.paint.brush.stroke_method = StrokeMethod::DragDot;
        t.on_canvas_pointer(cp([24.0, 24.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([72.0, 72.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([72.0, 72.0], PointerPhase::Up));
        // O ponto final mora à volta de (72, 72) com alcance ≤ 10·√2/0,6 ≈ 24 px.
        let mut rasto = 0usize;
        for y in 0..96u32 {
            for x in 0..96u32 {
                let longe = x < 46 && y < 46;
                if longe && px(&t, 96, x, y) != [255, 255, 255, 255] {
                    rasto += 1;
                }
            }
        }
        assert!(
            px(&t, 96, 72, 72)[0] < 200,
            "cor={cor}: CONTROLO — o ponto final pinta"
        );
        assert_eq!(rasto, 0, "cor={cor}: o ponto de partida deixou {rasto} px");
    }
}

/// O Wet Paint com a mesma ponta quadrada: o motor da tinta molhada percorre as células `r · reach`
/// (`ph2d_wet_paint::trail::Dab::reach`), e a janela do rasto cabe a ponta inteira.
fn mede_molhado(size: f32, graus: u16) -> (i32, i32, usize) {
    let mut t = white_canvas(96, 10.0);
    t.set_paint_media(super::super::media::PaintMedia::WetPaint);
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.paint.brush.color = [0.1, 0.1, 0.6];
    t.paint.brush.shape.size = [size, size];
    t.paint.brush.shape.angle_deg = graus;
    let s = 16u32;
    let rgba: Vec<u8> = std::iter::repeat_n([40u8, 40, 40, 255], (s * s) as usize)
        .flatten()
        .collect();
    t.set_brush_shape_image_rgba(&rgba, s, s, None);
    t.toggle_brush_shape_alpha_from_image();
    t.on_canvas_pointer(cp([48.0, 48.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([48.0, 48.0], PointerPhase::Up));
    let (mut ex, mut ey, mut n) = (0i32, 0i32, 0usize);
    for y in 0..96u32 {
        for x in 0..96u32 {
            if px(&t, 96, x, y)[0] < 235 {
                n += 1;
                ex = ex.max((x as i32 - 48).abs());
                ey = ey.max((y as i32 - 48).abs());
            }
        }
    }
    (ex, ey, n)
}

#[test]
#[ignore = "sonda: o Wet Paint com a ponta alargada"]
fn sonda_wet_paint_ponta() {
    for (size, graus) in [(1.0f32, 0u16), (1.0, 45), (0.6, 0), (0.6, 30)] {
        let (ex, ey, n) = mede_molhado(size, graus);
        println!("wet size={size} {graus}° → meia-larg {ex} meia-alt {ey} píxeis {n}");
    }
}

/// **O Wet Paint também não corta a ponta.** Medido: sem o alcance no motor, Size 0,6 e 30° davam a MESMA
/// mancha da Size 1 a 0° (meia-largura 18 = o raio 10 + o espalhar da água, 90 px); com ele, 30 e 37–39.
#[test]
fn o_wet_paint_nao_corta_a_ponta() {
    let (bx, by, bn) = mede_molhado(1.0, 0);
    for (size, graus) in [(1.0f32, 45u16), (0.6, 0), (0.6, 30)] {
        let (ex, ey, n) = mede_molhado(size, graus);
        assert!(
            ex >= bx + 4 && ey >= by + 4,
            "size={size} {graus}°: alcance {ex}/{ey} contra {bx}/{by} da ponta a 0°/Size 1"
        );
        if size < 1.0 {
            assert!(n > bn * 3 / 2, "size={size} {graus}°: {n} px contra {bn}");
        }
    }
}
