//! **Prancha B34** (missão 4K, decisão 6): o papel à escala do documento. Chalk (o papel como Grain) e Wet Round
//! (o papel do Wet), 3 fundos × 1080p/4K; o pincel é r40 a 1080p e r80 a 4K (o mesmo traço relativo). Grava
//! `fator 1 | fator do documento | Δ×32` (a 1080p os dois são o fator 1: Δ = 0) e imprime o Δ médio/máx.
//!
//! ```text
//! cargo test -p ph2d-tool-painter --profile smoke --test it prancha_b34 -- --ignored --nocapture
//! ```

use crate::exame_bancada::pinceis;
use ph2d_editor_core::Tool as _;
use ph2d_editor_core::tool::{
    CanvasPaintTool as _, CanvasPointer, PointerPhase, RasterEditTool as _,
};
use ph2d_tool_painter::PainterTool;

pub(crate) fn fundo(w: u32, h: u32, blocos: bool) -> Vec<u8> {
    let cores = [
        [200u8, 60, 50],
        [60, 150, 210],
        [240, 200, 60],
        [90, 170, 90],
    ];
    let mut v = vec![255u8; (w * h * 4) as usize];
    if blocos {
        for (i, p) in v.chunks_exact_mut(4).enumerate() {
            let (x, y) = (i as u32 % w, i as u32 / w);
            p[..3].copy_from_slice(&cores[usize::from(x * 2 >= w) + 2 * usize::from(y * 2 >= h)]);
        }
    }
    v
}

pub(crate) fn ponto(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// A imagem achatada de um zigue-zague; `fator1` força o «antes» (o motor sem a escala do documento).
fn cena(nome: &str, w: u32, h: u32, qual: u8, fator1: bool) -> Vec<u8> {
    let ps = pinceis(nome);
    let mut t = PainterTool::default();
    if qual == 2 {
        t.agenda_papel_texturizado(7);
        t.bind_document(7, fundo(w, h, false), w, h);
    } else {
        t.set_source(fundo(w, h, qual == 1), w, h);
    }
    if fator1 {
        t.fixa_fator_aparencia_para_sonda(1.0);
    }
    ps[0].aplica(&mut t, true);
    t.set_brush_size_px(40.0 * h as f32 / 1080.0);
    t.set_brush_color_srgb8([30, 50, 140]);
    let (wf, hf) = (w as f32, h as f32);
    let pernas = [
        ([0.3 * wf, 0.35 * hf], [0.7 * wf, 0.35 * hf]),
        ([0.7 * wf, 0.35 * hf], [0.3 * wf, 0.65 * hf]),
        ([0.3 * wf, 0.65 * hf], [0.7 * wf, 0.65 * hf]),
    ];
    t.on_tick(1000.0 / 60.0);
    t.on_canvas_pointer(ponto(pernas[0].0, PointerPhase::Down));
    let mut n = 0;
    for (a, b) in pernas {
        for k in 1..=40 {
            let f = k as f32 / 40.0;
            t.on_canvas_pointer(ponto(
                [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f],
                PointerPhase::Move,
            ));
            n += 1;
            if n % 3 == 0 {
                t.on_tick(1000.0 / 60.0);
            }
        }
    }
    t.on_canvas_pointer(ponto(pernas[2].1, PointerPhase::Up));
    for _ in 0..30 {
        t.on_tick(1000.0 / 60.0);
    }
    t.take_preview_arc().expect("o preview").0.to_vec()
}

/// `antes | depois | Δ×32`, recorte `[w/2 ± h/4] × [h/4, 3h/4]`, amostrado a 540 px de lado.
pub(crate) fn monta(antes: &[u8], depois: &[u8], w: u32, h: u32) -> image::RgbaImage {
    let lado = h / 2;
    let passo = lado / 540;
    let out = lado / passo;
    let (x0, y0) = ((w - lado) / 2, h / 4);
    let gap = 8u32;
    let mut par =
        image::RgbaImage::from_pixel(3 * out + 2 * gap, out, image::Rgba([40, 40, 40, 255]));
    for y in 0..out {
        for x in 0..out {
            let i = (((y0 + y * passo) * w + x0 + x * passo) * 4) as usize;
            for (k, img) in [antes, depois].iter().enumerate() {
                par.put_pixel(
                    x + k as u32 * (out + gap),
                    y,
                    image::Rgba([img[i], img[i + 1], img[i + 2], 255]),
                );
            }
            let m = (0..4)
                .map(|c| antes[i + c].abs_diff(depois[i + c]))
                .max()
                .unwrap_or(0);
            let k = 255 - (u16::from(m) * 32).min(255) as u8;
            par.put_pixel(x + 2 * (out + gap), y, image::Rgba([255, k, k, 255]));
        }
    }
    par
}

#[test]
#[ignore = "prancha B34: grava as PNG em docs/Painter/missao_4k/pranchas/B34/"]
fn prancha_b34_papel_a_escala_do_documento() {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/Painter/missao_4k/pranchas/B34"
    );
    std::fs::create_dir_all(dir).expect("pasta");
    for nome in ["Chalk", "Wet Round"] {
        for (w, h, res) in [(1920u32, 1080u32, "1080p"), (3840, 2160, "4k")] {
            for (qual, fundo_nome) in [(0u8, "branco"), (1, "blocos"), (2, "papel")] {
                let antes = cena(nome, w, h, qual, true);
                let depois = cena(nome, w, h, qual, false);
                let (mut soma, mut max) = (0u64, 0u8);
                for (a, d) in antes.iter().zip(&depois) {
                    let x = a.abs_diff(*d);
                    soma += u64::from(x);
                    max = max.max(x);
                }
                println!(
                    "[b34] {nome:<10} {res:<5} {fundo_nome:<6} Δ médio {:.4} máx {max}",
                    soma as f64 / antes.len() as f64
                );
                let arq = nome.replace(' ', "_").to_lowercase();
                monta(&antes, &depois, w, h)
                    .save(format!("{dir}/{res}_{fundo_nome}_{arq}_antes_depois.png"))
                    .expect("png");
            }
        }
    }
}
