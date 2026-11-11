//! **Prancha B29.3** (missão 4K): com o filme AA ligado o grão é TECTO ao repassar (antes só TAXA: o repasse apagava o
//! dente). Chalk e Pastel com o Impasto ligado (nenhum pincel de fábrica arma a película com grão: as miniaturas não
//! mudam), um traço que repassa 4× o mesmo sítio, 3 fundos ×
//! 1080p/4K. A lei velha não fica no motor: grava-se a variante com `PH2D_B29_3=<antes|depois>` (a lei de cada build)
//! e o terceiro passo monta `antes | depois | Δ×32`.
//!
//! ```text
//! PH2D_B29_3=depois cargo test -p ph2d-tool-painter --profile smoke --test it prancha_b29_3 -- --ignored
//! ```

use crate::exame_bancada::{Fonte, pinceis};
use ph2d_editor_core::Tool as _;
use ph2d_editor_core::tool::{
    CanvasPaintTool as _, CanvasPointer, PointerPhase, RasterEditTool as _,
};
use ph2d_tool_painter::PainterTool;

const DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/Painter/missao_4k/pranchas/B29"
);

fn fundo(w: u32, h: u32, blocos: bool) -> Vec<u8> {
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

fn ponto(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um traço que vai e volta 4× na horizontal pelo meio da tela; a imagem achatada.
fn cena(nome: &str, w: u32, h: u32, qual: u8) -> Vec<u8> {
    let mut ps = pinceis(nome);
    let Fonte::Preset { preset, .. } = &mut ps[0].fonte else {
        panic!("preset")
    };
    // O pincel de grão com o Impasto ligado e sem Shape: é aí que a película com AA arma (`film_aa_wanted`).
    let b = &mut preset.parametros.brush;
    b.insert("impasto".to_owned(), serde_json::json!(true));
    b.insert("impasto_depth".to_owned(), serde_json::json!(0.6));
    b.insert("impasto_smooth_edges".to_owned(), serde_json::json!(true));
    b.insert("continuous_spacing".to_owned(), serde_json::json!(false));
    let mut t = PainterTool::default();
    if qual == 2 {
        t.agenda_papel_texturizado(7);
        t.bind_document(7, fundo(w, h, false), w, h);
    } else {
        t.set_source(fundo(w, h, qual == 1), w, h);
    }
    ps[0].aplica(&mut t, true);
    t.set_brush_impasto(true);
    t.set_brush_size_px(h as f32 / 12.0);
    t.set_brush_color_srgb8([30, 50, 140]);
    let (wf, hf) = (w as f32, h as f32);
    let (a, b) = ([0.3 * wf, 0.5 * hf], [0.7 * wf, 0.5 * hf]);
    t.on_tick(1000.0 / 60.0);
    t.on_canvas_pointer(ponto(a, PointerPhase::Down));
    let mut n = 0;
    for perna in 0..4 {
        let (de, ate) = if perna % 2 == 0 { (a, b) } else { (b, a) };
        for k in 1..=40 {
            let f = k as f32 / 40.0;
            t.on_canvas_pointer(ponto(
                [de[0] + (ate[0] - de[0]) * f, de[1] + (ate[1] - de[1]) * f],
                PointerPhase::Move,
            ));
            n += 1;
            if n % 3 == 0 {
                t.on_tick(1000.0 / 60.0);
            }
        }
    }
    t.on_canvas_pointer(ponto(a, PointerPhase::Up));
    t.on_tick(1000.0 / 60.0);
    t.take_preview_arc().expect("o preview").0.to_vec()
}

const CASOS: [&str; 2] = ["Chalk", "Pastel"];
const RES: [(u32, u32, &str); 2] = [(1920, 1080, "1080p"), (3840, 2160, "4k")];
const FUNDOS: [(u8, &str); 3] = [(0, "branco"), (1, "blocos"), (2, "papel")];

fn arq(nome: &str) -> String {
    nome.replace(' ', "_").to_lowercase()
}

#[test]
#[ignore = "prancha B29.3: PH2D_B29_3=antes|depois grava a variante; =monta compõe as PNG"]
fn prancha_b29_3_grao_e_tecto_com_o_filme() {
    let modo = std::env::var("PH2D_B29_3").expect("PH2D_B29_3=antes|depois|monta");
    let tmp = format!("{}/../../target/prancha_b29_3", env!("CARGO_MANIFEST_DIR"));
    std::fs::create_dir_all(&tmp).expect("pasta");
    std::fs::create_dir_all(DIR).expect("pasta");
    for nome in CASOS {
        for (w, h, res) in RES {
            for (qual, fn_) in FUNDOS {
                let raw = |v: &str| format!("{tmp}/{v}_{res}_{fn_}_{}.raw", arq(nome));
                if modo != "monta" {
                    std::fs::write(raw(&modo), cena(nome, w, h, qual)).expect("raw");
                    continue;
                }
                let antes = std::fs::read(raw("antes")).expect("antes");
                let depois = std::fs::read(raw("depois")).expect("depois");
                let (mut soma, mut max, mut n) = (0u64, 0u8, 0u64);
                for (a, d) in antes.iter().zip(&depois) {
                    let x = a.abs_diff(*d);
                    soma += u64::from(x);
                    max = max.max(x);
                    n += u64::from(x > 0);
                }
                println!(
                    "[b29.3] {nome:<10} {res:<5} {fn_:<6} Δ médio nos tocados {:.2} · máx {max} · {n} bytes",
                    soma as f64 / n.max(1) as f64
                );
                monta(&antes, &depois, w, h)
                    .save(format!("{DIR}/{res}_{fn_}_{}_antes_depois.png", arq(nome)))
                    .expect("png");
            }
        }
    }
}

/// `antes | depois | Δ×32`, recorte `[w/2 ± h/4] × [h/4, 3h/4]`.
fn monta(antes: &[u8], depois: &[u8], w: u32, h: u32) -> image::RgbaImage {
    let lado = h / 2;
    let (x0, y0) = ((w - lado) / 2, h / 4);
    let gap = 8u32;
    let mut par =
        image::RgbaImage::from_pixel(3 * lado + 2 * gap, lado, image::Rgba([40, 40, 40, 255]));
    for y in 0..lado {
        for x in 0..lado {
            let i = (((y0 + y) * w + x0 + x) * 4) as usize;
            for (k, img) in [antes, depois].iter().enumerate() {
                par.put_pixel(
                    x + k as u32 * (lado + gap),
                    y,
                    image::Rgba([img[i], img[i + 1], img[i + 2], 255]),
                );
            }
            let m = (0..4)
                .map(|c| antes[i + c].abs_diff(depois[i + c]))
                .max()
                .unwrap_or(0);
            let k = 255 - (u16::from(m) * 32).min(255) as u8;
            par.put_pixel(x + 2 * (lado + gap), y, image::Rgba([255, k, k, 255]));
        }
    }
    par
}
