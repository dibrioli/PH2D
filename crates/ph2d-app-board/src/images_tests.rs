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
    let px = decode(&f.bytes.0, None).unwrap();
    assert_eq!(px.rgba, [10, 20, 30, 255, 40, 50, 60, 128]);
}

/// O que não é uma imagem que o app lê é recusado.
#[test]
fn what_is_not_an_image_is_refused() {
    assert!(ingest(b"not an image").is_none());
    assert!(
        ingest(b"<svg xmlns=\"http://www.w3.org/2000/svg\"><g").is_none(),
        "SVG estragado"
    );
}

const SVG_DISC: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20" viewBox="0 0 4 2"><circle cx="2" cy="1" r="1" fill="#00ff00"/><text>oi</text></svg>"##;

/// ⭐ W15 — um SVG entra como IMAGEM: os bytes como chegaram (o SVG exportado e o `.excalidraw`
/// embebem-nos), o tamanho do `<svg>`, e desenha-se ao tamanho pedido (o natural sem pedido).
#[test]
fn an_svg_comes_in_as_a_picture_drawn_at_any_size() {
    let f = ingest(SVG_DISC).expect("um SVG é uma imagem do quadro");
    assert_eq!((&*f.bytes.0, f.size), (SVG_DISC, [40, 20]));
    assert_eq!(f.mime(), Some("image/svg+xml"));
    assert_eq!(decode(SVG_DISC, None).unwrap().size, [40, 20]);
    let big = decode(SVG_DISC, Some([400, 200])).unwrap();
    assert_eq!(big.size, [400, 200]);
    let mid = &big.rgba[(100 * 400 + 200) * 4..][..4];
    assert_eq!(mid, [0, 255, 0, 255]);
}

/// Um `.svgz` entra já descomprimido (um navegador não mostra gzip num `data:`).
#[test]
fn a_svgz_comes_in_as_its_text() {
    use std::io::Write as _;
    let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    e.write_all(SVG_DISC).unwrap();
    let f = ingest(&e.finish().unwrap()).unwrap();
    assert_eq!(&*f.bytes.0, SVG_DISC);
}

/// ⭐ Os textos de um SVG não se desenham — e o aviso di-lo (não só o terminal).
#[test]
fn the_texts_of_an_svg_are_named_in_the_warning() {
    let p = Placed {
        placed: 1,
        refused: 0,
        texts: 1,
    };
    let w = p.warning().expect("um aviso");
    assert!(w.contains("1 text(s) inside the SVG"), "{w}");
    assert!(Placed::default().warning().is_none());
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
    assert!(decode(&b, None).is_none(), "a descodificação inteira falha");
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
        let px = decode(&jpg, None).unwrap();
        inteira = inteira.min(t.elapsed().as_secs_f64() * 1e3);
        assert_eq!(px.size, [w, h]);
    }
    println!(
        "JPEG {} KB: entrada {entrada:.3} ms · descodificação {inteira:.1} ms",
        jpg.len() / 1024
    );
}

/// ⭐ W15 — um `.excalidraw` com uma imagem `image/svg+xml` (a entrada `w15_svg` do oráculo, que o
/// Excalidraw 0.18.1 aceita e devolve igual) abre com a imagem SVG, pela porta de entrada do app.
#[test]
fn an_excalidraw_with_an_svg_picture_opens_with_it() {
    let scene = include_str!(
        "../../../docs/MiroClone/ferramentas/excalidraw_oracle/entradas/w15_svg.excalidraw"
    );
    let imp = ph2d_board_excalidraw::read_with(scene, &ingest).unwrap();
    assert!(!imp.skipped.contains_key("image"), "{:?}", imp.skipped);
    let f = imp.files.values().next().expect("o ficheiro SVG");
    assert_eq!(f.mime(), Some("image/svg+xml"));
    assert_eq!(f.size, [100, 100]);
}

// ── MEDIÇÃO (não é gate): a metade do QUADRO no «Send to board…» de um Flip ──────────────────────
// Uso: `docs/MiroClone/ferramentas/memoria_flip/corre.sh` (`PH2D_MEDE_LADO=<px>`, `--ignored
// --nocapture`): o `encode_png` + o `ingest` de um quadro do Flip `lado²`, em dois conteúdos.

fn rss_mib(campo: &str) -> f64 {
    let s = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    s.lines()
        .find(|l| l.starts_with(campo))
        .and_then(|l| l.split_whitespace().nth(1)?.parse::<f64>().ok())
        .map_or(-1.0, |kb| kb / 1024.0)
}

/// O tempo (mínimo de 3), o tamanho do PNG e o pico de RSS do `encode_png` + `ingest` de `lado²`
/// em dois conteúdos: o da medição do Flip (faixas opacas de uma cor, cantos transparentes) e o
/// pior caso (ruído, incompressível).
#[test]
#[ignore = "medição; corre por docs/MiroClone/ferramentas/memoria_flip/corre.sh"]
fn mede_png_do_quadro_do_flip() {
    let lado: u32 = std::env::var("PH2D_MEDE_LADO")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2048);
    let n = (lado as usize) * (lado as usize);
    let mut faixas = vec![0u8; n * 4];
    for (i, p) in faixas.chunks_exact_mut(4).enumerate() {
        let (x, y) = ((i % lado as usize) as f32, (i / lado as usize) as f32);
        let r = lado as f32 / 20.0;
        let canto = (x < r && y < r && (x - r).hypot(y - r) > r)
            || (x > lado as f32 - r && y < r && (x - lado as f32 + r).hypot(y - r) > r);
        if !canto {
            p.copy_from_slice(&[200, 128, 30, 255]);
        }
    }
    let mut s = 0x2545_F491_4F6C_DD1Du64;
    let ruido: Vec<u8> = (0..n * 4)
        .map(|_| {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            (s >> 24) as u8
        })
        .collect();
    for (nome, rgba) in [("faixas", faixas), ("ruido", ruido)] {
        let px = Pixels {
            size: [lado, lado],
            rgba,
        };
        let (mut te, mut ti, mut bytes) = (f64::INFINITY, f64::INFINITY, 0usize);
        let mut pico = 0.0f64;
        for _ in 0..3 {
            let _ = std::fs::write("/proc/self/clear_refs", "5");
            let rss0 = rss_mib("VmRSS:");
            let t = std::time::Instant::now();
            let png = encode_png(&px).expect("png");
            te = te.min(t.elapsed().as_secs_f64() * 1e3);
            let t = std::time::Instant::now();
            let f = ingest(&png).expect("ingest");
            ti = ti.min(t.elapsed().as_secs_f64() * 1e3);
            pico = pico.max(rss_mib("VmHWM:") - rss0);
            bytes = png.len();
            drop((png, f));
        }
        println!(
            "PNG lado={lado} conteudo={nome} rgba={:.1} MiB png={:.1} MiB encode_min={te:.1} ms ingest_min={ti:.1} ms pico_rss_acima_do_antes={pico:.1} MiB",
            (n * 4) as f64 / 1048576.0,
            bytes as f64 / 1048576.0
        );
    }
}

/// ⭐ W16 — o `Ctrl+V` de ficheiros copiados no Dolphin: cada caminho chega com o `\r` do CRLF no fim
/// (o `arboard` 3.6.1 parte a lista só no `\n`); as imagens entram, o resto é avisado.
#[test]
fn files_copied_in_dolphin_paste_in_with_the_crlf_stripped() {
    let d = std::env::temp_dir().join(format!("ph2d-w16-colar-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    let a = d.join("água teste.png");
    std::fs::write(&a, png(4, 3, [9, 9, 9, 255])).unwrap();
    let t = d.join("nota.txt");
    std::fs::write(&t, b"oi").unwrap();
    let mut h = HeroScreen::new(ph2d_editor_core::NodeId(1));
    assert!(
        paste_files(&mut h, vec![a.clone()]).is_none(),
        "sem quadro não é para aqui"
    );
    h.documents.create("Q".into());
    let cr = |p: &PathBuf| PathBuf::from(format!("{}\r", p.display()));
    let toast = paste_files(&mut h, vec![cr(&a), cr(&t)]).expect("é para o quadro");
    assert!(toast.is_some(), "o .txt avisa-se");
    let board = h.documents.active_board().unwrap();
    assert_eq!(board.doc.live().filter(|e| e.image().is_some()).count(), 1);
    let _ = std::fs::remove_dir_all(&d);
}

/// Um GIF 3×2 de três quadros (vermelho 70 ms, verde 0 ms, azul 40 ms), escrito pelo `image`.
fn gif_3() -> Vec<u8> {
    use image::{Delay, Frame, RgbaImage};
    let mut out = Vec::new();
    {
        let mut enc = image::codecs::gif::GifEncoder::new(&mut out);
        enc.set_repeat(image::codecs::gif::Repeat::Infinite)
            .unwrap();
        for (c, ms) in [
            ([255, 0, 0, 255], 70),
            ([0, 255, 0, 255], 0),
            ([0, 0, 255, 255], 40),
        ] {
            let buf = RgbaImage::from_pixel(3, 2, image::Rgba(c));
            enc.encode_frame(Frame::from_parts(
                buf,
                0,
                0,
                Delay::from_numer_denom_ms(ms, 1),
            ))
            .unwrap();
        }
    }
    out
}

/// ⭐ W16 — os quadros de um GIF saem em SEQUÊNCIA com a duração do ficheiro (a lei dos curtos é do
/// desenho), e o 1.º quadro do desenho é o primeiro dele; um PNG não é animação.
#[test]
fn a_gif_gives_its_frames_in_order_with_their_file_durations() {
    let g = gif_3();
    let got: Vec<([u8; 4], u32)> = frames(&g)
        .expect("um GIF")
        .map(|(px, ms)| {
            assert_eq!(px.size, [3, 2]);
            (px.rgba[..4].try_into().unwrap(), ms)
        })
        .collect();
    assert_eq!(
        got,
        [
            ([255, 0, 0, 255], 70),
            ([0, 255, 0, 255], 0),
            ([0, 0, 255, 255], 40)
        ]
    );
    assert_eq!(decode(&g, None).unwrap().rgba[..4], [255, 0, 0, 255]);
    assert_eq!(ingest(&g).unwrap().size, [3, 2]);
    assert!(frames(&png(2, 2, [1, 2, 3, 255])).is_none());
}
