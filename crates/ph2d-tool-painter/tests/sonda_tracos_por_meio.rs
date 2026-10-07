//! **Sonda (ignorada): o MESMO gesto nos quatro meios, gravado em PNG** — a metade NOSSA da comparação
//! com o oráculo Rebelle 8 (`docs/Painter/NewFeaturesWetPaint/`, 2026-10-07).
//!
//! Cada meio recebe uma tela 1000×640 branca e cinco gestos iguais aos que o Rebelle recebe pela
//! porta WebSocket (`ferramentas/gera_eventos.py` → `ferramentas/ws_eventos.mjs`, um evento por
//! mensagem; a injecção de rato não pinta): um traço recto a passo de rato (3 px por evento), uma
//! curva em S, um traço com rampa de pressão, um traço RÁPIDO (25 px por evento) e um cruzamento com
//! outra cor. A régua é a mesma dos dois lados:
//! `docs/Painter/NewFeaturesWetPaint/ferramentas/mede_traco.py`.
//!
//! ```text
//! PH2D_SONDA_OUT=/tmp/x cargo test -p ph2d-tool-painter --profile smoke --test sonda_tracos_por_meio -- --ignored --nocapture
//! ```
//!
//! Escreve `<meio>.ppm` (P6) — o PNG faz-se fora, com `magick`. Sem pacote novo.

use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::{PaintMedia, PainterTool};
use std::io::Write;

const W: u32 = 1000;
const H: u32 = 640;
/// O tamanho de fábrica da sonda; `PH2D_SONDA_SIZE` sobrepõe-no (o Rebelle recebe `size_px` 40 e mede 42 px de
/// largura; o nosso 40 mede 58–72 — a comparação iguala a LARGURA MEDIDA, não o número).
const BRUSH_PX: f32 = 40.0;
const AZUL: [u8; 3] = [23, 101, 179];
const VERMELHO: [u8; 3] = [191, 0, 18];

fn cp(pos: [f32; 2], pressure: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um gesto por uma polilinha, com `passo` px entre eventos e a pressão dada por `p(t)` em `t ∈ [0, 1]`.
fn gesto(t: &mut PainterTool, pts: &[[f32; 2]], passo: f32, p: impl Fn(f32) -> f32) {
    let total: f32 = pts
        .windows(2)
        .map(|w| ((w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2)).sqrt())
        .sum();
    let mut andado = 0.0;
    t.on_canvas_pointer(cp(pts[0], p(0.0), PointerPhase::Down));
    for w in pts.windows(2) {
        let dx = w[1][0] - w[0][0];
        let dy = w[1][1] - w[0][1];
        let len = (dx * dx + dy * dy).sqrt();
        let n = (len / passo).ceil().max(1.0) as u32;
        for k in 1..=n {
            let f = k as f32 / n as f32;
            andado += len / n as f32;
            t.on_canvas_pointer(cp(
                [w[0][0] + dx * f, w[0][1] + dy * f],
                p(andado / total),
                PointerPhase::Move,
            ));
            t.on_tick(8.0);
        }
    }
    let fim = pts[pts.len() - 1];
    t.on_canvas_pointer(cp(fim, p(1.0), PointerPhase::Up));
}

fn curva_s(x0: f32, x1: f32, cy: f32, amp: f32) -> Vec<[f32; 2]> {
    (0..=120)
        .map(|i| {
            let f = i as f32 / 120.0;
            [
                x0 + (x1 - x0) * f,
                cy + amp * (f * std::f32::consts::TAU * 1.5).sin(),
            ]
        })
        .collect()
}

fn pinta(meio: PaintMedia) -> (Vec<u8>, u32, u32) {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(meio);
    let size = std::env::var("PH2D_SONDA_SIZE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(BRUSH_PX);
    t.set_brush_size_px(size);
    t.set_brush_strength(1.0);
    t.set_brush_color_srgb8(AZUL);
    let cheio = |_: f32| 1.0f32;
    // Linha 1 — recto a passo de rato · curva em S.
    gesto(&mut t, &[[60.0, 120.0], [560.0, 120.0]], 3.0, cheio);
    gesto(&mut t, &curva_s(600.0, 960.0, 120.0, 25.0), 3.0, cheio);
    // Linha 2 — rampa de pressão (0,15 → 1 → 0,15) · traço RÁPIDO (25 px por evento).
    gesto(&mut t, &[[60.0, 300.0], [560.0, 300.0]], 3.0, |f| {
        0.15 + 0.85 * (f * std::f32::consts::PI).sin()
    });
    gesto(&mut t, &[[600.0, 300.0], [960.0, 300.0]], 25.0, cheio);
    // Linha 3 — recto, e um cruzamento vermelho (a mistura / o «dirty brush»).
    gesto(&mut t, &[[60.0, 480.0], [560.0, 480.0]], 3.0, cheio);
    t.set_brush_color_srgb8(VERMELHO);
    gesto(&mut t, &[[300.0, 400.0], [300.0, 560.0]], 3.0, cheio);
    gesto(&mut t, &[[700.0, 480.0], [960.0, 480.0]], 3.0, cheio);
    // O tempo passa (a água corre e seca; nos outros meios é inócuo): 8 s a 60 Hz.
    for _ in 0..480 {
        t.on_tick(1000.0 / 60.0);
    }
    t.run_full()
}

fn escreve_ppm(path: &std::path::Path, rgba: &[u8], w: u32, h: u32) {
    assert_eq!(rgba.len(), (w * h * 4) as usize, "run_full devolve RGBA8");
    let mut f = std::fs::File::create(path).expect("ppm");
    write!(f, "P6\n{w} {h}\n255\n").unwrap();
    let mut rgb = Vec::with_capacity((w * h * 3) as usize);
    for px in rgba.chunks_exact(4) {
        rgb.extend_from_slice(&px[..3]);
    }
    f.write_all(&rgb).unwrap();
}

#[test]
#[ignore = "sonda: grava PNGs para a comparação com o oráculo; corre-se à mão"]
fn os_mesmos_gestos_nos_quatro_meios() {
    let out = std::env::var("PH2D_SONDA_OUT").unwrap_or_else(|_| {
        std::env::temp_dir()
            .join("ph2d_sonda_tracos")
            .display()
            .to_string()
    });
    std::fs::create_dir_all(&out).unwrap();
    for (nome, meio) in [
        ("digital", PaintMedia::Digital),
        ("watercolor", PaintMedia::Watercolor),
        ("impasto", PaintMedia::Impasto),
        ("wetpaint", PaintMedia::WetPaint),
    ] {
        let t0 = std::time::Instant::now();
        let (px, w, h) = pinta(meio);
        let p = std::path::Path::new(&out).join(format!("{nome}.ppm"));
        escreve_ppm(&p, &px, w, h);
        println!(
            "[sonda] {nome:<10} {w}x{h} → {} ({:.1} s)",
            p.display(),
            t0.elapsed().as_secs_f32()
        );
    }
}
