//! W14 — a porta de entrada das imagens: o que a cena lê, o quadro lê; o que um navegador não mostra
//! entra em PNG; o resto é recusado.

use super::*;

/// Uma PNG RGBA `w × h` com a cor `c`.
fn png(w: u32, h: u32, c: [u8; 4]) -> Vec<u8> {
    let px = Pixels {
        size: [w, h],
        rgba: c.repeat((w * h) as usize),
    };
    encode_png(&px).unwrap()
}

/// Uma QOI 2×1 escrita à mão (a especificação: cabeçalho `qoif`, um `QOI_OP_RGBA` por píxel, o
/// marcador do fim) — um formato que a cena lê e nenhum navegador mostra.
fn qoi_2x1() -> Vec<u8> {
    let mut b = b"qoif".to_vec();
    b.extend(2u32.to_be_bytes());
    b.extend(1u32.to_be_bytes());
    b.extend([4, 0]);
    b.extend([0xff, 10, 20, 30, 255, 0xff, 40, 50, 60, 128]);
    b.extend([0, 0, 0, 0, 0, 0, 0, 1]);
    b
}

/// Uma PNG entra como chegou (os mesmos bytes, a mesma chave), com o tamanho natural.
#[test]
fn a_png_comes_in_as_it_arrived() {
    let b = png(5, 3, [1, 2, 3, 255]);
    let f = ingest(&b).unwrap();
    assert_eq!((&*f.bytes.0, f.size), (&b[..], [5, 3]));
    assert_eq!(f.mime(), Some("image/png"));
}

/// ⭐ Uma QOI (a cena lê-a; um navegador não) entra em PNG, com os mesmos píxeis.
#[test]
fn a_format_the_browser_does_not_show_comes_in_as_png() {
    let f = ingest(&qoi_2x1()).expect("a cena lê QOI");
    assert_eq!((f.mime(), f.size), (Some("image/png"), [2, 1]));
    let px = decode(&f.bytes.0).unwrap();
    assert_eq!(px.rgba, [10, 20, 30, 255, 40, 50, 60, 128]);
}

/// O que não é uma imagem que o app lê é recusado.
#[test]
fn what_is_not_an_image_is_refused() {
    assert!(ingest(b"not an image").is_none());
    assert!(ingest(b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>").is_none());
}

/// ⭐ Auditoria W14 — o tamanho de uma PNG vem do CABEÇALHO: uns bytes com um cabeçalho válido e os
/// dados estragados entram com o tamanho declarado (a descodificação, que falharia, fica para o
/// desenho, fora do fio da janela).
#[test]
fn the_size_comes_from_the_header_without_decoding() {
    let mut b = png(300, 200, [9, 9, 9, 255]);
    let n = b.len();
    for x in &mut b[n - 20..n - 12] {
        *x ^= 0xa5;
    }
    assert!(decode(&b).is_none(), "a descodificação inteira falha");
    let f = ingest(&b).expect("o cabeçalho chega");
    assert_eq!(f.size, [300, 200]);
}

/// RÉGUA (à mão, máquina calma, perfil `smoke`/release): o custo no fio da janela da entrada de uma
/// fotografia JPEG de 4000 × 3000 (o cabeçalho) contra a descodificação inteira que o desenho faz fora
/// dele (o tempo do cinzento). Imprime o MÍNIMO de 5 corridas intercaladas.
#[test]
#[ignore = "régua de tempo (à mão)"]
fn mede_a_entrada_e_a_descodificacao_de_uma_fotografia() {
    let (w, h) = (4000u32, 3000u32);
    let img = image::RgbImage::from_fn(w, h, |x, y| {
        image::Rgb([
            (x * 7 % 256) as u8,
            (y * 5 % 256) as u8,
            ((x ^ y) % 256) as u8,
        ])
    });
    let mut jpg = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpg, 85)
        .encode_image(&img)
        .unwrap();
    let (mut entrada, mut inteira) = (f64::MAX, f64::MAX);
    for _ in 0..5 {
        let t = std::time::Instant::now();
        let f = ingest(&jpg).unwrap();
        entrada = entrada.min(t.elapsed().as_secs_f64() * 1e3);
        assert_eq!(f.size, [w, h]);
        let t = std::time::Instant::now();
        let px = decode(&jpg).unwrap();
        inteira = inteira.min(t.elapsed().as_secs_f64() * 1e3);
        assert_eq!(px.size, [w, h]);
    }
    println!(
        "JPEG {} KB: entrada {entrada:.3} ms · descodificação {inteira:.1} ms",
        jpg.len() / 1024
    );
}
