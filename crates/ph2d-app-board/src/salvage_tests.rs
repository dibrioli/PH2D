//! W18 — o que se lê de uma imagem cortada, contra o Chromium (`excalidraw_oracle/saidas/w17`, sonda
//! `D01_cena_*_truncado`, a coluna x=520: a imagem 400×300 desenhada a 1:1 em y=180).

use super::*;

const ENT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/MiroClone/ferramentas/excalidraw_oracle/entradas/"
);

/// Quantas linhas de cima têm tinta (alfa) na coluna `x`.
fn rows_read(px: &Pixels, x: u32) -> u32 {
    (0..px.size[1])
        .take_while(|&y| px.rgba[((y * px.size[0] + x) * 4 + 3) as usize] != 0)
        .count() as u32
}

/// ⭐⭐ O PNG cortado a 60 %: as linhas de cima até onde o IDAT chega — o Chromium mostra 173
/// (y = 180..=352 no ecrã), o resto transparente; as cores das linhas lidas são as do ficheiro bom.
#[test]
fn a_truncated_png_shows_the_rows_it_has_like_the_browser() {
    let bytes = std::fs::read(format!("{ENT}w17_png_truncado.png")).unwrap();
    assert!(crate::images::decode(&bytes, None).is_none(), "o inteiro recusa-o");
    let px = salvage(&bytes).expect("lê as linhas de cima");
    assert_eq!(px.size, [400, 300]);
    let n = rows_read(&px, 120);
    assert!((172..=174).contains(&n), "Chromium: 173 linhas; nós {n}");
    assert_eq!(&px.rgba[(120 * 4)..(120 * 4 + 4)], &[255, 0, 0, 255], "o quadrante de cima é vermelho");
    let below = ((n + 5) * 400 + 120) as usize * 4;
    assert_eq!(px.rgba[below + 3], 0, "abaixo do corte, transparente");
}

/// ⭐⭐ O JPEG cortado a 50 %: o Chromium mostra 142 linhas; um JPEG descodifica-se por blocos (16
/// linhas no 4:2:0) — a mesma ordem de grandeza, nunca a imagem inteira nem nada.
#[test]
fn a_truncated_jpeg_shows_the_rows_it_has_like_the_browser() {
    let bytes = std::fs::read(format!("{ENT}w17_jpeg_truncado.jpg")).unwrap();
    assert!(crate::images::decode(&bytes, None).is_none(), "o inteiro recusa-o");
    let px = salvage(&bytes).expect("lê as linhas de cima");
    let n = rows_read(&px, 120);
    assert!((126..=158).contains(&n), "Chromium: 142 linhas; nós {n}");
    let top = &px.rgba[(10 * 400 + 120) * 4..(10 * 400 + 120) * 4 + 4];
    assert!(top[0] > 230 && top[1] < 25 && top[2] < 25, "vermelho em cima: {top:?}");
}

/// O que não é PNG/JPEG, ou não tem nada legível, não se salva (o cinzento riscado continua).
#[test]
fn garbage_and_other_formats_are_not_salvaged() {
    assert!(salvage(b"not an image").is_none());
    let lixo = std::fs::read(format!("{ENT}w17_png_idat_lixo.png")).unwrap();
    let r = salvage(&lixo);
    assert!(r.is_none_or(|p| rows_read(&p, 120) == 0), "o IDAT de lixo não dá linhas");
}

