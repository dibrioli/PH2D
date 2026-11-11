//! **Prancha B29.1** (missão 4K, decisão 3): Flow baixo em CAMADAS — cinco traços pretos, cada um repassado mais
//! vezes que o de cima, com o pincel de omissão (Strength 1). Com a tinta em `u8` os de baixo ficam IGUAIS: o
//! traço estanca (Flow 2 % ≈ 25/255, Flow 0,2 % ≈ 250/255 no carimbo, `precisao_tests`). 3 fundos × 1080p/4K.
//! DEPOIS (onda 18, item AB): o tecto largo está ligado ao carimbo POR DAB de cor constante; o pincel de omissão
//! vai pela banda (`u8`, por fazer), por isso o DEPOIS é o mesmo traço com Continuous (a cápsula: carimbo por dab).
//!
//! ```text
//! cargo test -p ph2d-tool-painter --profile smoke --test it prancha_b29_1 -- --ignored
//! ```

use ph2d_editor_core::Tool as _;
use ph2d_editor_core::tool::{
    CanvasPaintTool as _, CanvasPointer, PointerPhase, RasterEditTool as _,
};
use ph2d_tool_painter::PainterTool;

const DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/Painter/missao_4k/pranchas/B29.1"
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
        for (i, p) in v.as_chunks_mut::<4>().0.iter_mut().enumerate() {
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

/// Cinco traços horizontais; o `k`-ésimo vai e volta `passes[k]` vezes num só pen-down.
fn cena(flow: f32, passes: [u32; 5], w: u32, h: u32, qual: u8, continuo: bool) -> Vec<u8> {
    let mut t = PainterTool::default();
    if continuo {
        t.toggle_brush_continuous_spacing();
    }
    if qual == 2 {
        t.agenda_papel_texturizado(7);
        t.bind_document(7, fundo(w, h, false), w, h);
    } else {
        t.set_source(fundo(w, h, qual == 1), w, h);
    }
    t.set_brush_size_px(h as f32 / 14.0);
    t.set_brush_hardness(1.0);
    t.set_brush_strength(1.0);
    t.set_brush_flow(flow);
    t.set_brush_color_srgb8([0, 0, 0]);
    let (wf, hf) = (w as f32, h as f32);
    for (k, &n) in passes.iter().enumerate() {
        let y = hf * (0.18 + 0.16 * k as f32);
        let (a, b) = ([0.15 * wf, y], [0.85 * wf, y]);
        t.on_tick(1000.0 / 60.0);
        t.on_canvas_pointer(ponto(a, PointerPhase::Down));
        for perna in 0..n {
            let (de, ate) = if perna % 2 == 0 { (a, b) } else { (b, a) };
            for j in 1..=12 {
                let f = j as f32 / 12.0;
                t.on_canvas_pointer(ponto([de[0] + (ate[0] - de[0]) * f, y], PointerPhase::Move));
            }
            t.on_tick(1000.0 / 60.0);
        }
        t.on_canvas_pointer(ponto(a, PointerPhase::Up));
        t.on_tick(1000.0 / 60.0);
    }
    t.take_preview_arc().expect("o preview").0.to_vec()
}

const FLOWS: [(f32, [u32; 5], &str); 2] = [
    (0.02, [2, 6, 15, 40, 80], "flow2"),
    (0.002, [10, 30, 80, 160, 320], "flow02"),
];
const RES: [(u32, u32, &str); 2] = [(1920, 1080, "1080p"), (3840, 2160, "4k")];
const FUNDOS: [(u8, &str); 3] = [(0, "branco"), (1, "blocos"), (2, "papel")];

#[test]
#[ignore = "prancha B29.1: grava o ANTES (Flow baixo em camadas) em pranchas/B29.1"]
fn prancha_b29_1_flow_baixo_em_camadas() {
    grava(false, "antes");
}

#[test]
#[ignore = "prancha B29.1: grava o DEPOIS (Continuous: o carimbo por dab com o tecto largo) em pranchas/B29.1"]
fn prancha_b29_1_depois_continuous() {
    grava(true, "depois_continuous");
}

fn grava(continuo: bool, quando: &str) {
    std::fs::create_dir_all(DIR).expect("pasta");
    for (flow, passes, fl) in FLOWS {
        for (w, h, res) in RES {
            for (qual, fn_) in FUNDOS {
                let img = cena(flow, passes, w, h, qual, continuo);
                // O centro de cada traço, no meio da tela: a cor pousada (canal mais claro) por camada.
                let niveis: Vec<u8> = (0..5)
                    .map(|k| {
                        let y = (h as f32 * (0.18 + 0.16 * k as f32)) as u32;
                        let i = ((y * w + w / 2) * 4) as usize;
                        img[i..i + 3].iter().copied().max().unwrap_or(0)
                    })
                    .collect();
                println!(
                    "[b29.1 {quando}] {fl:<6} {res:<5} {fn_:<6} passes {passes:?} → centro {niveis:?}"
                );
                // O recorte do meio (largura h/2, altura toda): as 5 camadas sem 10 MB de papel por PNG.
                let rgba = image::RgbaImage::from_raw(w, h, img).expect("imagem");
                image::imageops::crop_imm(&rgba, (w - h / 2) / 2, 0, h / 2, h)
                    .to_image()
                    .save(format!("{DIR}/{res}_{fn_}_{fl}_{quando}.png"))
                    .expect("png");
            }
        }
    }
}
