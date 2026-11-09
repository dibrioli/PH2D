//! Os cinco filtros do PNG: o escritor só usa o 0, mas um preset regravado por outro programa chega
//! com qualquer um (o Krita e o GIMP escolhem por linha).

use super::{Cabecalho, desfiltrar, paeth};

fn imagem(w: usize, h: usize, canais: usize) -> Vec<u8> {
    (0..w * h * canais)
        .map(|i| ((i * 37 + (i / 7) * 11) % 251) as u8)
        .collect()
}

/// O codificador de referência de cada filtro (PNG §9.2), aplicado a TODAS as linhas.
fn filtrar(img: &[u8], w: usize, h: usize, bpp: usize, filtro: u8) -> Vec<u8> {
    let linha = w * bpp;
    let mut out = Vec::with_capacity(h * (linha + 1));
    for y in 0..h {
        out.push(filtro);
        for i in 0..linha {
            let x = img[y * linha + i];
            let a = if i >= bpp {
                img[y * linha + i - bpp]
            } else {
                0
            };
            let b = if y > 0 { img[(y - 1) * linha + i] } else { 0 };
            let c = if y > 0 && i >= bpp {
                img[(y - 1) * linha + i - bpp]
            } else {
                0
            };
            let pred = match filtro {
                0 => 0,
                1 => a,
                2 => b,
                3 => ((u16::from(a) + u16::from(b)) / 2) as u8,
                _ => paeth(a, b, c),
            };
            out.push(x.wrapping_sub(pred));
        }
    }
    out
}

#[test]
fn os_cinco_filtros_desfazem_se_em_rgba_e_rgb() {
    let (w, h) = (13, 9);
    for canais in [3usize, 4] {
        let img = imagem(w, h, canais);
        let esperado: Vec<u8> = if canais == 4 {
            img.clone()
        } else {
            img.as_chunks::<3>()
                .0
                .iter()
                .flat_map(|p| [p[0], p[1], p[2], 255])
                .collect()
        };
        for filtro in 0..=4u8 {
            let cab = Cabecalho {
                largura: w as u32,
                altura: h as u32,
                canais,
            };
            let lida = desfiltrar(&cab, &filtrar(&img, w, h, canais, filtro)).unwrap();
            assert_eq!(lida, esperado, "filtro {filtro}, {canais} canais");
        }
    }
}

#[test]
fn um_filtro_fora_da_norma_e_recusado_com_o_numero() {
    let cab = Cabecalho {
        largura: 1,
        altura: 1,
        canais: 4,
    };
    let e = desfiltrar(&cab, &[5, 1, 2, 3, 4]).unwrap_err();
    assert_eq!(e, crate::Erro::MiniaturaIlegivel("filtro 5".to_owned()));
}

/// ⭐ Um `f64` CALCULADO volta ao bit pela ida e volta do JSON (09/11): a `intensity` que a migração E4 dá ao
/// Wet Wash, `0.011789473684210525`, sem a feature `float_roundtrip` do `serde_json` lia-se `…523` e o
/// «Save as new» recusava o pincel. O mesmo no `brush` (`Value`) e na tabela da largura.
#[test]
fn um_f64_calculado_volta_ao_bit_pelo_json() {
    use crate::modelo::Parametros;
    let x = 0.011_789_473_684_210_525_f64;
    let mut p = Parametros::novo("WetPaint", "Fluid", "Wet Wash 2");
    p.engine.insert("intensity".to_owned(), x);
    p.engine
        .insert("volume".to_owned(), 0.008_151_382_823_871_907);
    p.brush.insert("hardness".to_owned(), serde_json::json!(x));
    p.width_table.push([x, 2.0]);
    let texto = super::json_canonico(&p).expect("o f64 calculado volta igual");
    assert!(texto.contains("0.011789473684210525"), "{texto}");
}
