//! ⭐ **Os ORÁCULOS da saída** (W8, `docs/MiroClone/06_equipa_e_saida.md` §1.4): o nosso SVG e o nosso
//! PDF corridos pelos rasterizadores e validadores que o sistema tem — `resvg` e `rsvg-convert` (o SVG),
//! `pdftoppm`, `pdftotext`, `pdfinfo` e `qpdf --check` (o PDF), `xmllint` — e comparados com o PNG da
//! PLACA (o desenho do ecrã). E o PNG em ladrilhos contra o PNG inteiro (sem costura). Corre à mão
//! (precisa da placa e das ferramentas). Medido 09/10 (`load` 6): ladrilhos de 300 px = a imagem inteira
//! AO BIT; o SVG pelo resvg 0,038 % e pelo rsvg-convert 0,024 % dos píxeis com um canal a mais de 48 da
//! placa; o PDF pelo pdftoppm 0,118 % e 0,018 % (a sombra das notas: a máscara gaussiana contra a
//! aproximação do Vello) — os tectos são ~3× o medido. Antes de tirar o fio de interface dos frames do
//! export a 1.ª página dava 0,407 % (a linha na borda da página).
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-board-export --test it oracle -- --ignored --nocapture
//! ```

use std::path::{Path, PathBuf};
use std::process::Command;

use ph2d_board_export::{
    ExportTarget, PNG_MEGAPIXELS, board_page, frame_page, paint_page, pdf, png, svg,
};
use ph2d_board_render::EXPORT_PAPER;
use ph2d_board_route::RouteCache;
use ph2d_gpu::{GpuContext, TextureFormat};
use ph2d_render::VelloPass;
use ph2d_text::TextSystem;

use crate::sample::sample;

fn dir() -> PathBuf {
    let d = Path::new(env!("CARGO_TARGET_TMPDIR")).join("w8_oracle");
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn run(cmd: &str, args: &[&str]) -> String {
    let out = Command::new(cmd)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("o oráculo `{cmd}` não corre ({e}) — ver §1.4 do estudo"));
    assert!(
        out.status.success(),
        "`{cmd} {args:?}` falhou: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn decode(path: &Path) -> (u32, u32, Vec<u8>) {
    let dec = ::png::Decoder::new(std::io::BufReader::new(std::fs::File::open(path).unwrap()));
    let mut r = dec.read_info().unwrap();
    let mut buf = vec![0; r.output_buffer_size().unwrap()];
    let info = r.next_frame(&mut buf).unwrap();
    let px: Vec<u8> = match info.color_type {
        ::png::ColorType::Rgba => buf[..info.buffer_size()].to_vec(),
        ::png::ColorType::Rgb => buf[..info.buffer_size()]
            .chunks(3)
            .flat_map(|c| [c[0], c[1], c[2], 255])
            .collect(),
        t => panic!("{t:?}"),
    };
    (info.width, info.height, px)
}

/// Sobre papel branco: a fracção de píxeis cuja maior diferença de canal passa `tol`, e a média.
fn diff(a: &[u8], b: &[u8], tol: u8) -> (f64, f64) {
    let over = |p: &[u8]| -> [f64; 3] {
        let al = f64::from(p[3]) / 255.0;
        [0, 1, 2].map(|i| f64::from(p[i]) * al + 255.0 * (1.0 - al))
    };
    let (mut bad, mut sum, mut n) = (0usize, 0.0, 0usize);
    for (p, q) in a.chunks(4).zip(b.chunks(4)) {
        let (p, q) = (over(p), over(q));
        let d = (0..3).map(|i| (p[i] - q[i]).abs()).fold(0.0, f64::max);
        if d > f64::from(tol) {
            bad += 1;
        }
        sum += d;
        n += 1;
    }
    (bad as f64 / n as f64, sum / n as f64)
}

#[test]
#[ignore = "oráculo: corre-se à mão com a placa e as ferramentas do sistema (ver o cabeçalho)"]
fn oracle_svg_pdf_and_tiles_against_the_gpu() {
    let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
        panic!("sem adaptador: o oráculo da placa não corre");
    };
    let mut pass = VelloPass::new(&gpu, TextureFormat::Rgba8Unorm, (64, 64)).expect("VelloPass");
    let (set, [f1, f2]) = sample();
    let b = &set.boards()[0];
    let mut routes = RouteCache::default();
    routes.sync(&b.doc);
    let mut ts = TextSystem::without_system_fonts();
    let d = dir();
    let page = board_page(b, &routes).unwrap();
    let paper = Some(EXPORT_PAPER);

    // 1. A placa: inteira e em ladrilhos de 300 px (com fronteiras a cortar notas e letras).
    let whole = png::render(
        &gpu,
        &mut pass,
        b,
        &page,
        1.0,
        paper,
        8192,
        &std::collections::BTreeSet::new(),
        &mut ts,
    )
    .unwrap();
    let tiled = png::render(
        &gpu,
        &mut pass,
        b,
        &page,
        1.0,
        paper,
        300,
        &std::collections::BTreeSet::new(),
        &mut ts,
    )
    .unwrap();
    let (seam_bad, seam_mean) = diff(&whole.rgba, &tiled.rgba, 2);
    println!(
        "ladrilhos 300 px contra inteira: {:.4} % > 2 · média {seam_mean:.4}",
        seam_bad * 100.0
    );
    std::fs::write(d.join("gpu.png"), png::encode(&whole)).unwrap();
    std::fs::write(d.join("gpu_tiled.png"), png::encode(&tiled)).unwrap();

    // 2. O SVG (vector: o detalhe inteiro) por dois rasterizadores.
    let (mut s, size) = paint_page(
        b,
        &page,
        1.0,
        paper,
        ExportTarget::Svg,
        &std::collections::BTreeSet::new(),
        &mut ts,
    );
    let svg_text = svg::write(&s.take_record(), size);
    let svg_path = d.join("board.svg");
    std::fs::write(&svg_path, &svg_text).unwrap();
    run("xmllint", &["--noout", svg_path.to_str().unwrap()]);
    let resvg_png = d.join("board_resvg.png");
    run(
        "resvg",
        &[svg_path.to_str().unwrap(), resvg_png.to_str().unwrap()],
    );
    let rsvg_png = d.join("board_rsvg.png");
    run(
        "rsvg-convert",
        &[svg_path.to_str().unwrap(), "-o", rsvg_png.to_str().unwrap()],
    );
    for (name, p) in [("resvg", &resvg_png), ("rsvg-convert", &rsvg_png)] {
        let (w, h, px) = decode(p);
        assert_eq!((w, h), (whole.width, whole.height), "{name}: o tamanho");
        let (bad, mean) = diff(&whole.rgba, &px, 48);
        println!(
            "SVG por {name} contra a placa: {:.3} % > 48 · média {mean:.3}",
            bad * 100.0
        );
        assert!(
            bad < 0.002,
            "{name}: {:.3} % dos píxeis diferem (> 48)",
            bad * 100.0
        );
    }

    // 3. O PDF: uma página por frame; texto, estrutura e o desenho de cada página.
    let pages = [frame_page(b, f1).unwrap(), frame_page(b, f2).unwrap()];
    let recs: Vec<_> = pages
        .iter()
        .map(|p| {
            let (mut sc, sz) = paint_page(
                b,
                p,
                1.0,
                paper,
                ExportTarget::Pdf,
                &std::collections::BTreeSet::new(),
                &mut ts,
            );
            (sc.take_record(), sz)
        })
        .collect();
    let pdf_pages: Vec<pdf::PdfPage<'_>> = recs
        .iter()
        .map(|(ops, size)| pdf::PdfPage {
            ops,
            size: *size,
            links: Vec::new(),
        })
        .collect();
    let pdf_path = d.join("board.pdf");
    std::fs::write(&pdf_path, pdf::write(&pdf_pages)).unwrap();
    let p = pdf_path.to_str().unwrap();
    run("qpdf", &["--check", p]);
    let info = run("pdfinfo", &[p]);
    assert!(info.contains("Pages:           2"), "{info}");
    let text = run("pdftotext", &["-layout", p, "-"]);
    println!("pdftotext: {text:?}");
    for t in [
        "Slow builds ação",
        "Sketch",
        "Half",
        "next",
        "Cache",
        "urgent",
    ] {
        assert!(text.contains(t), "o texto «{t}» não se extrai: {text:?}");
    }
    let prefix = d.join("board_pdf");
    run(
        "pdftoppm",
        &["-r", "72", "-png", p, prefix.to_str().unwrap()],
    );
    for (i, pg) in pages.iter().enumerate() {
        let gpu_page = png::render(
            &gpu,
            &mut pass,
            b,
            pg,
            1.0,
            paper,
            8192,
            &std::collections::BTreeSet::new(),
            &mut ts,
        )
        .unwrap();
        let (w, h, px) = decode(&d.join(format!("board_pdf-{}.png", i + 1)));
        assert_eq!((w, h), (gpu_page.width, gpu_page.height), "página {i}");
        let (bad, mean) = diff(&gpu_page.rgba, &px, 48);
        println!(
            "PDF p.{} por pdftoppm contra a placa: {:.3} % > 48 · média {mean:.3}",
            i + 1,
            bad * 100.0
        );
        assert!(bad < 0.005, "página {i}: {:.3} %", bad * 100.0);
    }
    assert!(
        seam_bad < 1e-4,
        "costura entre ladrilhos: {:.4} %",
        seam_bad * 100.0
    );
    let _ = PNG_MEGAPIXELS;
}
