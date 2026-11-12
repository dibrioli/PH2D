//! T1-13 (B19.2): a dirty window under a SPATIAL adjustment is the crop of the whole-canvas recompose,
//! to the byte — per spatial `AdjustmentKind`, one at a time and stacked (the halos add up), at an
//! interior window and at windows that touch the canvas edge.

use super::*;
use ph2d_painter_effects::adjustments::{
    AdjustmentKind, AdjustmentParams, BloomParams, ChromaticAberrationParams, GaussianBlurParams,
    MotionBlurParams, ShadowsHighlightsParams, SharpenParams,
};

const W: u32 = 61;
const H: u32 = 47;

fn imagem(semente: u32, alfa: bool) -> LayerImage {
    let mut img = LayerImage::transparent(W, H);
    for (i, p) in img.rgba8_mut().chunks_mut(4).enumerate() {
        let (x, y) = (i as u32 % W, i as u32 / W);
        let v = (x * 37 + y * 101 + semente * 53) ^ (x * y + semente);
        p[0] = (v % 251) as u8;
        p[1] = ((v / 7) % 253) as u8;
        p[2] = ((v / 13) % 241) as u8;
        p[3] = if alfa { ((v / 3) % 256) as u8 } else { 255 };
    }
    img
}

fn ajustes() -> Vec<AdjustmentParams> {
    vec![
        AdjustmentParams::GaussianBlur(GaussianBlurParams { radius: 4.5 }),
        AdjustmentParams::Sharpen(SharpenParams {
            amount: 1.5,
            radius: 3.0,
            mask_edges: false,
        }),
        AdjustmentParams::MotionBlur(MotionBlurParams {
            distance: 9.0,
            angle: 0.6,
        }),
        AdjustmentParams::ChromaticAberration(ChromaticAberrationParams {
            red_shift: 5.0,
            green_shift: -2.0,
            blue_shift: -6.0,
            falloff_center: 0.0,
        }),
        AdjustmentParams::Bloom(BloomParams {
            threshold: 0.4,
            intensity: 0.8,
            radius: 5.0,
            falloff: 0.2,
        }),
        // Radius past the low-res cap: the downsample grid is the canvas's, never a window's.
        AdjustmentParams::Bloom(BloomParams {
            threshold: 0.4,
            intensity: 0.8,
            radius: 40.0,
            falloff: 0.2,
        }),
        AdjustmentParams::ShadowsHighlights(ShadowsHighlightsParams {
            shadows_amount: 0.6,
            shadows_tonal_width: 0.5,
            shadows_radius: 6.0,
            highlights_amount: 0.4,
            highlights_tonal_width: 0.5,
            highlights_radius: 3.0,
            color_correction: 0.3,
            midtone_contrast: 0.2,
        }),
    ]
}

fn pilha(params: &[AdjustmentParams]) -> (LayerStack, MapPixelSource) {
    let mut s = LayerStack::new();
    let mut src = MapPixelSource::default();
    let fundo = s.add_raster("fundo", W, H).unwrap();
    src.insert(fundo, imagem(1, false));
    let tinta = s.add_raster("tinta", W, H).unwrap();
    src.insert(tinta, imagem(2, true));
    for p in params {
        let a = s.add_adjustment(p.kind()).unwrap();
        s.adjustment_mut(a).unwrap().params = p.clone();
    }
    let topo = s.add_raster("topo", W, H).unwrap();
    s.get_mut(topo).unwrap().opacity = 0.5;
    src.insert(topo, imagem(3, true));
    (s, src)
}

fn janelas() -> [Region; 4] {
    [
        Region {
            x: 23,
            y: 17,
            w: 9,
            h: 7,
        },
        Region {
            x: 0,
            y: 0,
            w: 5,
            h: 4,
        },
        Region {
            x: 50,
            y: 40,
            w: 11,
            h: 7,
        },
        Region {
            x: 1,
            y: 30,
            w: 60,
            h: 2,
        },
    ]
}

fn confere(params: &[AdjustmentParams]) -> Vec<String> {
    let (s, src) = pilha(params);
    let tela = composite(&s, &src, W, H);
    let mut falhas = Vec::new();
    for r in janelas() {
        let janela = composite_region(&s, &src, W, H, r);
        let mut recorte = Vec::with_capacity(janela.len());
        for y in r.y..r.y + r.h {
            let i = ((y * W + r.x) * 4) as usize;
            recorte.extend_from_slice(&tela[i..i + (r.w * 4) as usize]);
        }
        let n = janela
            .chunks(4)
            .zip(recorte.chunks(4))
            .filter(|(a, b)| a != b)
            .count();
        if n > 0 {
            falhas.push(format!(
                "{:?} em {r:?}: {n} píxeis",
                params.iter().map(|p| p.kind()).collect::<Vec<_>>()
            ));
        }
    }
    falhas
}

#[test]
fn a_janela_suja_sob_um_ajuste_espacial_e_o_recorte_da_tela() {
    let mut falhas = Vec::new();
    for p in ajustes() {
        assert!(p.kind().reads_the_image_layout(), "{:?}", p.kind());
        falhas.extend(confere(std::slice::from_ref(&p)));
    }
    // Stacked: the halos ADD UP (each kernel reads the previous kernel's output).
    let todos = ajustes();
    falhas.extend(confere(&[
        todos[0].clone(),
        todos[2].clone(),
        todos[3].clone(),
    ]));
    assert!(falhas.is_empty(), "{falhas:#?}");
}

/// Every spatial kind is in the census above (a new one must enter the gate).
#[test]
fn o_censo_da_janela_cobre_todo_ajuste_espacial() {
    let cobertos: std::collections::BTreeSet<_> = ajustes()
        .iter()
        .map(|p| format!("{:?}", p.kind()))
        .collect();
    let espaciais: std::collections::BTreeSet<_> = AdjustmentKind::ALL
        .iter()
        .filter(|k| k.gpu_spatial_code().is_some())
        .map(|k| format!("{k:?}"))
        .collect();
    assert_eq!(cobertos, espaciais);
}
