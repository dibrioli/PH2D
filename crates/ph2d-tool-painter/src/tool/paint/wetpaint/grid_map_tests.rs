//! Os gates da porta de conversão pixel↔célula ([`super`]).
//!
//! O que eles provam, em ordem de importância:
//!
//! 1. **razão 1 é o mundo de sempre, AO BIT** — não "aproximadamente", não
//!    "dentro de um épsilon": a mesma expressão de `f64`. É isto que autoriza
//!    a wave a não re-pinar o fingerprint do ADR-0134.
//! 2. **as portas são INVERSAS** — dab e silhueta têm de concordar sobre onde
//!    uma célula está, senão o carimbo sai deslocado de meia célula e ninguém
//!    vê o número que erra (a doença `seed == sample`).
//! 3. **nenhuma tinta cai fora da grade** — o `div_ceil` é correção, não
//!    arredondamento de gosto.
//! 4. **o upsample não inventa nem vaza** — campo constante volta constante, e
//!    a cor de um vizinho TRANSPARENTE não entra na tinta.

use super::*;

/// A razão 1 é a identidade nas QUATRO portas, e a comparação é exata.
///
/// Mutação: trocar o early-out de `px_to_cell` pela divisão geral (`px / 1.0 +
/// 1.0`) mantém este gate verde — a divisão por 1,0 é exata em IEEE-754 —,
/// então ele NÃO prova o early-out, prova a LEI. O early-out é performance, e
/// está dito assim no doc.
#[test]
fn ratio_one_is_the_world_that_shipped() {
    for px in [0.0f64, 0.5, 1.0, 123.25, 4095.5] {
        assert_eq!(px_to_cell(px, 1), px + 1.0, "dab em {px}");
    }
    for len in [0.0f64, 1.0, 10.5, 100.0] {
        assert_eq!(px_len_to_cell(len, 1), len, "raio {len}");
    }
    for c in [1i32, 2, 137, 4096] {
        assert_eq!(
            cell_center_px(c, 1),
            (c - 1) as f32 + 0.5,
            "silhueta em {c}"
        );
        assert_eq!(cell_center_texel(c, 1), i64::from(c - 1), "texel em {c}");
    }
    assert_eq!(grid_dims(4096, 2048, 1), (4096, 2048));
    assert_eq!(
        cell_rect_to_px((1, 1, 4096, 2048), 1, 4096, 2048),
        (0, 0, 4096, 2048)
    );
}

/// A silhueta e o dab concordam sobre onde uma célula está, em TODA razão.
///
/// O centro da célula `c`, levado de volta ao espaço do motor, tem de dar
/// exatamente `c + 0,5` (a convenção que o `+ 1.0` da rota sempre significou).
/// Mutação: `cell_center_px` sem o `+ r*0.5` (o canto em vez do centro) faz
/// isto dar `c` em vez de `c + 0,5` — meia célula de deslocamento, que a
/// razão 1 esconde por ser meio pixel.
#[test]
fn the_dab_door_and_the_silhouette_door_are_inverses() {
    for ratio in [1u8, 2, 3, 4, 7, 16, 30] {
        for c in [1i32, 2, 5, 50] {
            let px = f64::from(cell_center_px(c, ratio));
            let back = px_to_cell(px, ratio);
            let want = f64::from(c) + 0.5;
            assert!(
                (back - want).abs() < 1e-4,
                "ratio {ratio} cell {c}: centro {px} px voltou como {back}, esperado {want}"
            );
        }
    }
}

/// Todo pixel do documento tem célula — o `div_ceil`.
///
/// Mutação: `w / r` (floor) deixa a última fatia sem célula e este gate
/// nomeia o pixel que ficaria sem água.
#[test]
fn every_canvas_pixel_has_a_cell() {
    // 4096 nao e divisivel por 30, 7 ou 3 — as razoes que expoem o floor.
    for ratio in [1u8, 2, 3, 7, 30] {
        let (gw, gh) = grid_dims(4096, 2048, ratio);
        for (px, dim, g, axis) in [(4095usize, 4096usize, gw, "x"), (2047, 2048, gh, "y")] {
            let cell = px_to_cell(px as f64 + 0.5, ratio).floor() as usize;
            assert!(
                cell <= g,
                "ratio {ratio} eixo {axis}: o pixel {px} de {dim} cai na celula {cell}, \
                 fora de uma grade de {g}"
            );
        }
    }
}

/// Um canvas menor que uma célula ainda tem grade (o `max(1)`).
#[test]
fn a_canvas_smaller_than_one_cell_still_has_a_grid() {
    assert_eq!(grid_dims(10, 10, 30), (1, 1));
}

/// A janela de células cobre o canvas e nunca passa dele.
#[test]
fn a_cell_window_covers_its_pixels_and_stops_at_the_document() {
    let (w, h) = (100usize, 50usize);
    let (gw, gh) = grid_dims(w, h, 7);
    let (x0, y0, x1, y1) = cell_rect_to_px((1, 1, gw, gh), 7, w, h);
    assert_eq!(
        (x0, y0, x1, y1),
        (0, 0, w, h),
        "a grade inteira e o canvas inteiro"
    );
    // Uma celula do meio cobre exatamente sua fatia.
    assert_eq!(cell_rect_to_px((3, 2, 3, 2), 7, w, h), (14, 7, 21, 14));
}

/// Constrói um plano de pigmento `gw × gh` com uma cor/alpha por célula.
fn plane(gw: usize, gh: usize, f: impl Fn(usize, usize) -> [u8; 4]) -> Vec<u8> {
    let mut v = vec![0u8; gw * gh * 4];
    for cy in 0..gh {
        for cx in 0..gw {
            v[(cy * gw + cx) * 4..(cy * gw + cx) * 4 + 4].copy_from_slice(&f(cx, cy));
        }
    }
    v
}

/// Na razão 1 o amostrador devolve **o próprio pixel**, premultiplicado — a
/// prova de que a rota de identidade e a bilinear não podem divergir.
///
/// Mutação: `u()` sem o `+ 0.5` (ou com `px as f64` em vez do centro) desloca
/// a amostra um pixel e este gate nomeia o valor do vizinho.
#[test]
fn at_ratio_one_the_sampler_returns_that_very_pixel() {
    let (gw, gh) = (8usize, 4usize);
    let pig = plane(gw, gh, |cx, cy| [(cx * 30) as u8, (cy * 60) as u8, 7, 200]);
    let s = SampleU::new(1, gw, gh);
    assert!(s.is_identity());
    for py in 0..gh {
        for px in 0..gw {
            let got = s.at(&pig, px, py);
            let o = (py * gw + px) * 4;
            let a = f64::from(pig[o + 3]);
            let want = [
                f64::from(pig[o]) * a,
                f64::from(pig[o + 1]) * a,
                f64::from(pig[o + 2]) * a,
                a,
            ];
            assert_eq!(got, want, "pixel ({px},{py})");
        }
    }
}

/// Um campo CONSTANTE volta constante em qualquer razão — o upsample não
/// inventa estrutura (nem escurece a borda por peso perdido).
#[test]
fn the_upsample_of_a_flat_field_is_flat() {
    for ratio in [2u8, 4, 30] {
        let (gw, gh) = grid_dims(64, 64, ratio);
        let pig = plane(gw, gh, |_, _| [200, 100, 50, 255]);
        let s = SampleU::new(ratio, gw, gh);
        for py in (0..64).step_by(7) {
            for px in (0..64).step_by(5) {
                let g = s.at(&pig, px, py);
                let a = g[3];
                assert!(a > 0.0);
                let (r, b) = (g[0] / a, g[2] / a);
                assert!(
                    (r - 200.0).abs() < 0.01 && (b - 50.0).abs() < 0.01 && (a - 255.0).abs() < 0.01,
                    "ratio {ratio} px ({px},{py}): {g:?}"
                );
            }
        }
    }
}

/// A cor de um vizinho **transparente** não entra na tinta — a prova de que a
/// interpolação é premultiplicada.
///
/// Fixture: uma célula opaca VERMELHA ao lado de uma transparente cuja cor
/// straight é VERDE puro. Em straight-alpha o pixel do meio sairia com verde
/// dentro (o halo); em premultiplicado ele é vermelho com alpha caído.
///
/// Mutação: interpolar sem multiplicar por `a` (straight) faz o verde
/// aparecer e este gate o nomeia.
#[test]
fn a_transparent_neighbour_does_not_bleed_its_colour() {
    let ratio = 8u8;
    let (gw, gh) = (4usize, 1usize);
    // celula 1 = vermelho opaco; celula 2 = verde TRANSPARENTE.
    let pig = plane(gw, gh, |cx, _| match cx {
        0 => [255, 0, 0, 255],
        _ => [0, 255, 0, 0],
    });
    let s = SampleU::new(ratio, gw, gh);
    // ⚠️ **Nenhum pixel cai no meio de duas celulas** com r par: os centros
    // estao em px 4,0 e 12,0, e o meio (8,0) e a FRONTEIRA entre os pixels 7 e
    // 8. A 1a versao deste gate afirmava "px 7 esta a meio caminho" e falhou
    // sobre codigo correto (143,4 em vez de 127,5) — a fixture errava a
    // aritmetica. A propriedade verdadeira e a SIMETRIA em torno do meio.
    let (a7, a8) = (s.at(&pig, 7, 0), s.at(&pig, 8, 0));
    for (px, g) in [(7usize, a7), (8, a8)] {
        assert!(g[3] > 0.0, "px {px}: alguma cobertura: {g:?}");
        let green = g[1] / g[3];
        let red = g[0] / g[3];
        assert!(
            green < 0.01 && red > 254.0,
            "px {px}: verde transparente vazou: r={red} g={green} \
             (premultiplicado nao pode vazar)"
        );
    }
    // Simetria: os dois pixels que abracam o meio somam a cobertura cheia, e
    // o de dentro carrega mais que o de fora.
    assert!(
        (a7[3] + a8[3] - 255.0).abs() < 0.01,
        "os dois lados do meio somam a cobertura cheia: {} + {}",
        a7[3],
        a8[3]
    );
    assert!(a7[3] > a8[3], "o pixel mais perto da tinta carrega mais");
}

/// A borda do documento não amostra o pad ring (a célula 0 é do motor).
#[test]
fn the_document_edge_clamps_into_the_live_grid() {
    let ratio = 4u8;
    let (gw, gh) = grid_dims(32, 32, ratio);
    let pig = plane(gw, gh, |_, _| [10, 20, 30, 255]);
    // O pixel (0,0) tem u < 1 nos dois eixos: o clamp e o que o mantem vivo.
    let g = SampleU::new(ratio, gw, gh).at(&pig, 0, 0);
    assert!(
        (g[3] - 255.0).abs() < 0.01,
        "canto superior-esquerdo: {g:?}"
    );
    let g = SampleU::new(ratio, gw, gh).at(&pig, 31, 31);
    assert!((g[3] - 255.0).abs() < 0.01, "canto inferior-direito: {g:?}");
}

/// O clamp da faixa do slider é a faixa que o painel oferece.
#[test]
fn the_slider_range_is_the_clamp() {
    assert_eq!(clamp_ratio(0), MIN_RATIO);
    assert_eq!(clamp_ratio(255), MAX_RATIO);
    assert_eq!(DEFAULT_RATIO, 1, "a fabrica e a grade de sempre");
}
