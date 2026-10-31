//! Os portões da A.1 ([`super`]): a 100 % a vista é a linha de hoje ao byte; a 400 % tem o detalhe que a ampliação
//! bilinear de hoje não tem.

use super::prova::{
    VISUAL, bilinear, camadas, campo, faca, fraccao_outliers, hoje, pico_da_celula, var_laplaciano,
};
use super::{PreparoVista, Vista};
use crate::oleo_real::OleoReal;

/// O recorte de prova: 64 × 64 px do documento sobre a faca.
const RECORTE: (usize, usize, usize, usize) = (16, 16, 64, 64);

fn vista(s: f64) -> Vista {
    let (x0, y0, w, h) = RECORTE;
    Vista {
        x0: x0 as f64,
        y0: y0 as f64,
        w: (w as f64 * s) as usize,
        h: (h as f64 * s) as usize,
        s,
    }
}

/// ⭐ Identidade: a `s = 1` a vista nova é a linha do óleo de hoje AO BYTE (Catmull-Rom nos centros = a amostra).
#[test]
fn a_vista_a_100_por_cento_e_a_linha_de_hoje_ao_byte() {
    let e = faca();
    let o = OleoReal::default();
    let c = campo(&e, &o);
    let antes = hoje(&e, &o, &c, RECORTE);
    assert!(antes.chunks(4).filter(|p| p[3] > 0).count() > 1000, "a faca tem tinta no recorte");
    let nova = PreparoVista::novo(None, &camadas(&e), VISUAL, &o, &c, vista(1.0)).render();
    let dif = antes.iter().zip(&nova).filter(|(a, b)| a != b).count();
    assert_eq!(dif, 0, "{dif} bytes diferem");
}

/// ⭐ Nitidez: a 400 % a variância do laplaciano da vista nova contra a da ampliação BILINEAR do composite de hoje.
/// O alvo do dono é 3×. Medido 30/10: bilinear 54,64 · nova 178,58 = 3,27× com a altura por B-spline interpolante
/// (era 2,47× com Catmull-Rom). Vale só com os portões do pico da célula e dos pontinhos ao lado.
#[test]
fn a_vista_a_400_por_cento_tem_o_detalhe_que_a_ampliacao_nao_tem() {
    let e = faca();
    let o = OleoReal::default();
    let c = campo(&e, &o);
    let (_, _, w, h) = RECORTE;
    let ampliada = bilinear(&hoje(&e, &o, &c, RECORTE), w, h, 4);
    let nova = PreparoVista::novo(None, &camadas(&e), VISUAL, &o, &c, vista(4.0)).render();
    let (vb, vn) = (var_laplaciano(&ampliada, w * 4, h * 4), var_laplaciano(&nova, w * 4, h * 4));
    eprintln!("var laplaciano 400 %: bilinear de hoje {vb:.3} · vista nova {vn:.3} · razão {:.2}", vn / vb);
    assert!(vn >= 3.0 * vb, "bilinear {vb:.3} × nova {vn:.3}");
}

/// As duas imagens a 400 %: a ampliação bilinear de hoje e a vista nova.
fn as_duas_a_400() -> (Vec<u8>, Vec<u8>, usize, usize) {
    let e = faca();
    let o = OleoReal::default();
    let c = campo(&e, &o);
    let (_, _, w, h) = RECORTE;
    let ampliada = bilinear(&hoje(&e, &o, &c, RECORTE), w, h, 4);
    let nova = PreparoVista::novo(None, &camadas(&e), VISUAL, &o, &c, vista(4.0)).render();
    (ampliada, nova, w * 4, h * 4)
}

/// ⭐ Anti-artefacto: no laplaciano da vista nova a 400 % a energia no período da CÉLULA (4 px) não sobressai dos bins
/// vizinhos — pico/vizinhança ≤ 1,5. Um padrão ao passo da célula é detalhe FALSO (e infla a régua da nitidez).
#[test]
fn a_vista_a_400_por_cento_nao_tem_padrao_ao_passo_da_celula() {
    let (ampliada, nova, w, h) = as_duas_a_400();
    let (pb, pn) = (pico_da_celula(&ampliada, w, h, 4), pico_da_celula(&nova, w, h, 4));
    eprintln!("pico da célula 400 %: bilinear de hoje {pb:.3} · vista nova {pn:.3}");
    assert!(pn <= 1.5, "pico da célula {pn:.3} (bilinear {pb:.3})");
}

/// ⭐ Sem pontinhos: a fracção de pixels com |lap| > 6 σ robusto (MAD) da vista nova ≤ 1,5× a da bilinear de hoje.
#[test]
fn a_vista_a_400_por_cento_nao_tem_mais_pontinhos_que_a_ampliacao() {
    let (ampliada, nova, w, h) = as_duas_a_400();
    let (ob, on) = (fraccao_outliers(&ampliada, w, h), fraccao_outliers(&nova, w, h));
    eprintln!("outliers 6σ 400 %: bilinear de hoje {ob:.5} · vista nova {on:.5}");
    assert!(on <= 1.5 * ob, "outliers {on:.5} × bilinear {ob:.5}");
}
