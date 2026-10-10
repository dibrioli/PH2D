//! **Sonda (ignorada): os 4 pincéis de fábrica do Wet Paint na bancada do Rebelle** (conferência do dono
//! 10/10: «o Wet Wash é seco mesmo? o nome confere?»). A geometria de `ferramentas/gotas.py` (tela 1000×640,
//! traços horizontais de 150 px a 40 px, pressão 1, o azul da bancada, x0 = 60 + 240·i, y = 150), um pincel
//! por coluna — Wet Round · Wet Wash · Wet Dry Brush · Wet Drip —, cada um numa tela SUA (a água de um não
//! corre com os knobs do outro) e as quatro coladas. Grava `presets_<modo>_2s.ppm` e `presets_<modo>.ppm`
//! (30 s) em `PH2D_SONDA_OUT`, para `modo` ∈ `plano` (tela direita) · `inclinado` (anel 8, para baixo) ·
//! `preset` (o Tilt que cada preset guarda). A régua: `ferramentas/presets_regua.py`.
use crate::tool::PainterTool;
use ph2d_brush_presets::{Biblioteca, Volumes};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

const W: usize = 1000;
const H: usize = 640;
const PINCEIS: [&str; 4] = ["Wet Round", "Wet Wash", "Wet Dry Brush", "Wet Drip"];

fn ponto(x: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, 150.0],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um pincel na coluna `i`: as telas a 2 s e a 30 s.
fn coluna(b: &Biblioteca, i: usize, modo: &str) -> (Vec<u8>, Vec<u8>) {
    let p = &b
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == PINCEIS[i])
        .expect("o pincel de fábrica")
        .preset;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(p).is_empty());
    t.wet_preset_aplicar_volumes(&Volumes {
        size_px: Some(40.0),
        ..p.parametros.volumes.clone()
    });
    match modo {
        "plano" => t.paint.wetpaint.tilt_on = false,
        "inclinado" => {
            t.paint.wetpaint.tilt_on = true;
            t.paint.wetpaint.tilt_ring = 8;
            t.paint.wetpaint.tilt_spoke = 3;
        }
        _ => {}
    }
    t.set_brush_color_srgb8([23, 101, 179]);
    assert!(t.ensure_wet_session());
    let x0 = 60.0 + i as f32 * 240.0;
    t.on_canvas_pointer(ponto(x0, PointerPhase::Down));
    for k in 1..=50 {
        t.on_canvas_pointer(ponto(x0 + 3.0 * k as f32, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(ponto(x0 + 150.0, PointerPhase::Up));
    for _ in 0..120 {
        t.on_tick(1000.0 / 60.0);
    }
    let dois = t.canvas_rgba.to_vec();
    for _ in 120..30 * 60 {
        t.on_tick(1000.0 / 60.0);
    }
    (dois, t.canvas_rgba.to_vec())
}

fn grava(dir: &str, nome: &str, px: &[u8]) {
    let mut ppm = format!("P6 {W} {H} 255\n").into_bytes();
    for q in px.as_chunks::<4>().0 {
        ppm.extend_from_slice(&q[..3]);
    }
    std::fs::write(format!("{dir}/{nome}.ppm"), ppm).expect("grava a tela");
}

#[test]
#[ignore = "sonda: corre-se à mão"]
fn sonda_presets() {
    let dir = std::env::var("PH2D_SONDA_OUT").expect("PH2D_SONDA_OUT");
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let b = Biblioteca::carregar(&raiz, None);
    for modo in ["plano", "inclinado", "preset"] {
        let (mut a, mut z) = (vec![255u8; W * H * 4], vec![255u8; W * H * 4]);
        for i in 0..4 {
            let (dois, fim) = coluna(&b, i, modo);
            // Cada tela só tem a sua coluna: cola a faixa x 240·i … 240·(i+1).
            for y in 0..H {
                for x in 240 * i..(240 * (i + 1)).min(W) {
                    let k = (y * W + x) * 4;
                    a[k..k + 4].copy_from_slice(&dois[k..k + 4]);
                    z[k..k + 4].copy_from_slice(&fim[k..k + 4]);
                }
            }
        }
        grava(&dir, &format!("presets_{modo}_2s"), &a);
        grava(&dir, &format!("presets_{modo}"), &z);
    }
}
