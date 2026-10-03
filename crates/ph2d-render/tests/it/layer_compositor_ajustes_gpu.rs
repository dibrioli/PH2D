//! ADR-0177, P3 — os ajustes na placa: leis ABSOLUTAS (não contra o espelho da CPU do teste, que
//! muda junto com o shader). Irmão de `layer_compositor_gpu` (os ajudantes vêm de lá).

use super::layer_compositor_gpu::{MapProvider, try_headless_gpu};
use ph2d_render::{
    LayerCompositor, LayerOp, Region, SPATIAL_CHROMA, SPATIAL_GAUSSIAN, SPATIAL_MOTION,
    SPATIAL_SHARPEN, gaussian_weights, motion_weights,
};

/// ⭐ Um ajuste a 100 % muda a COR que o píxel tem e nunca a cobertura: um Invert sobre um píxel
/// semitransparente sai o negativo EXACTO, com o alfa intacto. O braço antigo misturava a cor
/// ajustada por `over` com o alfa da base e aplicava só parte (`a = 128` ⇒ ~⅔): 79 degraus contra
/// o GIMP (`ph2d-tool-painter` `oraculo_ajustes_tests`).
#[test]
#[ignore = "needs a GPU device"]
fn gpu_um_ajuste_cheio_sobre_um_pixel_translucido_aplica_se_inteiro() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (16u32, 8u32);
    let mut prov = MapProvider::default();
    prov.insert(1, 1, [200u8, 100, 51, 128].repeat((w * h) as usize));
    let ops = vec![
        LayerOp::Layer {
            mask: None,
            clipping: false,
            key: 1,
            blend_mode: 0,
            opacity: 1.0,
        },
        LayerOp::Adjustment {
            mask: None,
            kind: 2, // Invert
            params: [0.0; 3],
            blend_mode: 0,
            opacity: 1.0,
        },
    ];
    let mut comp = LayerCompositor::new(&gpu);
    comp.composite(&gpu, &ops, &prov, w, h, Region::full(w, h))
        .expect("composite");
    let got = comp.read_output(&gpu).expect("readback");
    assert!(
        got.as_chunks::<4>()
            .0
            .iter()
            .all(|p| *p == [55, 155, 204, 128]),
        "esperado [55, 155, 204, 128] ao byte, primeiro píxel {:?}",
        &got[..4]
    );
}

/// ⭐ O Threshold é a regra de 8 bits do Photoshop — o BYTE da luma `≥ threshold`, i.e. a luma
/// contínua `≥ (t − ½)/255`: no limiar `128` o cinzento `128` sai branco, o `127` preto, e o
/// `127,75` (um `127` sob um `128` a alfa `191`) BRANCO — o corte em `t/255` punha-o no preto. Só
/// o `127,75` distingue as duas regras na placa: ali a luma do `128` exacto não cai abaixo do corte
/// (a mutação G16 sobreviveu ao gate que só tinha `128` e `127`).
#[test]
#[ignore = "needs a GPU device"]
fn gpu_o_threshold_e_o_byte_da_luma_contra_o_limiar() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (15u32, 8u32);
    let n = (w * h) as usize;
    // Por píxel, o alfa do `128` sobre o `127`: opaco → 128, nada → 127, 191 → 127,749.
    const ALFAS: [u8; 3] = [255, 0, 191];
    const QUER: [u8; 3] = [255, 0, 255];
    let mut topo = Vec::with_capacity(n * 4);
    for i in 0..n {
        topo.extend_from_slice(&[128, 128, 128, ALFAS[i % 3]]);
    }
    let mut prov = MapProvider::default();
    prov.insert(0, 1, [127u8, 127, 127, 255].repeat(n));
    prov.insert(1, 1, topo);
    let camada = |key| LayerOp::Layer {
        mask: None,
        clipping: false,
        key,
        blend_mode: 0,
        opacity: 1.0,
    };
    let ops = vec![
        camada(0),
        camada(1),
        LayerOp::Adjustment {
            mask: None,
            kind: 4, // Threshold, p0 = 128/255 (o `gpu_params` do CPU)
            params: [128.0 / 255.0, 0.0, 0.0],
            blend_mode: 0,
            opacity: 1.0,
        },
    ];
    let mut comp = LayerCompositor::new(&gpu);
    comp.composite(&gpu, &ops, &prov, w, h, Region::full(w, h))
        .expect("composite");
    let got = comp.read_output(&gpu).expect("readback");
    for (i, p) in got.as_chunks::<4>().0.iter().enumerate() {
        let q = QUER[i % 3];
        assert_eq!(
            *p,
            [q, q, q, 255],
            "píxel {i} (alfa do 128: {})",
            ALFAS[i % 3]
        );
    }
}

/// Uma faixa `w × h` de duas cores (`esq` antes de `corte`), uma camada só.
fn degrau(w: u32, h: u32, corte: u32, esq: [u8; 4], dir: [u8; 4]) -> Vec<u8> {
    (0..w * h)
        .flat_map(|i| if i % w < corte { esq } else { dir })
        .collect()
}

/// Compõe `px` (uma camada) com `op` por cima e devolve os bytes.
fn compoe(gpu: &ph2d_gpu::GpuContext, w: u32, h: u32, px: Vec<u8>, op: LayerOp) -> Vec<u8> {
    let mut prov = MapProvider::default();
    prov.insert(0, 1, px);
    let ops = vec![
        LayerOp::Layer {
            mask: None,
            clipping: false,
            key: 0,
            blend_mode: 0,
            opacity: 1.0,
        },
        op,
    ];
    let mut comp = LayerCompositor::new(gpu);
    comp.composite(gpu, &ops, &prov, w, h, Region::full(w, h))
        .expect("composite");
    comp.read_output(gpu).expect("readback")
}

fn espacial(kernel: u8, p: [f32; 3]) -> LayerOp {
    LayerOp::SpatialAdjustment {
        kernel,
        params: [p[0], p[1], p[2], 0.0, 0.0, 0.0, 0.0, 0.0],
        blend_mode: 0,
        opacity: 1.0,
    }
}

/// O núcleo simétrico `pesos[0..=half]` sobre a linha `px` (bytes straight), em tons de ecrã
/// PRÉ-MULTIPLICADOS, borda por clamp — `[cor, alfa]` em `0..=1`.
fn nucleo_em_ecra(px: &[[u8; 4]], pesos: &[f32], half: u32) -> Vec<[f32; 2]> {
    let w = px.len() as i32;
    let half = half as i32;
    (0..w)
        .map(|x| {
            let (mut c, mut a) = (0.0f32, 0.0f32);
            for k in -half..=half {
                let s = px[(x + k).clamp(0, w - 1) as usize];
                let p = pesos[k.unsigned_abs() as usize];
                let sa = f32::from(s[3]) / 255.0;
                c += f32::from(s[0]) / 255.0 * sa * p;
                a += sa * p;
            }
            [c / a, a]
        })
        .collect()
}

fn byte(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// ⭐ ADR-0177 P4: os desfoques da placa borram em tons de ecrã pré-multiplicados, como o Krita a
/// 8 bits — o valor de cada píxel é o núcleo sobre os BYTES/255 (em luz o meio de um degrau preto ↔
/// branco clareia dezenas de degraus). Lei ABSOLUTA, ±1 (a placa): não contra o espelho do teste.
#[test]
#[ignore = "needs a GPU device"]
fn gpu_os_desfoques_borram_em_tons_de_ecra() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h, corte) = (32u32, 4u32, 16u32);
    let (preto, branco) = ([0u8, 0, 0, 255], [255u8, 255, 255, 255]);
    let (g64, g191) = ([64u8, 64, 64, 255], [191u8, 191, 191, 255]);
    let branco_140 = [255u8, 255, 255, 140];
    let (gw, gh) = gaussian_weights(4.5);
    let (sw, sh) = gaussian_weights(3.0);
    let (mw, mh) = motion_weights(9.0);
    let casos: [(&str, [u8; 4], [u8; 4], LayerOp); 4] = [
        (
            "gaussian preto|branco",
            preto,
            branco,
            espacial(SPATIAL_GAUSSIAN, [4.5, 0.0, 0.0]),
        ),
        (
            "gaussian branco a 140|preto",
            branco_140,
            preto,
            espacial(SPATIAL_GAUSSIAN, [4.5, 0.0, 0.0]),
        ),
        (
            "motion preto|branco",
            preto,
            branco,
            espacial(SPATIAL_MOTION, [9.0, 0.0, 0.0]),
        ),
        (
            "sharpen 64|191",
            g64,
            g191,
            espacial(SPATIAL_SHARPEN, [0.5, 3.0, 0.0]),
        ),
    ];
    for (nome, esq, dir, op) in casos {
        let linha: Vec<[u8; 4]> = (0..w).map(|x| if x < corte { esq } else { dir }).collect();
        let quer: Vec<[u8; 2]> = match op {
            LayerOp::SpatialAdjustment { kernel, .. } if kernel == SPATIAL_SHARPEN => {
                nucleo_em_ecra(&linha, &sw[..], sh)
                    .iter()
                    .zip(&linha)
                    .map(|(d, b)| {
                        let b = f32::from(b[0]) / 255.0;
                        [byte(b + 0.5 * (b - d[0])), 255]
                    })
                    .collect()
            }
            LayerOp::SpatialAdjustment { kernel, .. } if kernel == SPATIAL_MOTION => {
                nucleo_em_ecra(&linha, &mw[..], mh)
                    .iter()
                    .map(|v| [byte(v[0]), byte(v[1])])
                    .collect()
            }
            _ => nucleo_em_ecra(&linha, &gw[..], gh)
                .iter()
                .map(|v| [byte(v[0]), byte(v[1])])
                .collect(),
        };
        let got = compoe(&gpu, w, h, degrau(w, h, corte, esq, dir), op);
        let mut a_um = 0;
        for (i, p) in got.as_chunks::<4>().0.iter().enumerate() {
            let q = quer[i % w as usize];
            for (k, alvo) in [(0, q[0]), (3, q[1])] {
                let d = p[k].abs_diff(alvo);
                assert!(
                    d <= 1,
                    "{nome}: x={} canal {k}: {} ≠ {alvo} (núcleo em tons de ecrã)",
                    i as u32 % w,
                    p[k]
                );
                a_um += usize::from(d == 1);
            }
        }
        eprintln!("{nome}: ao byte salvo {a_um} canais a 1");
    }
}

/// ⭐ O Chroma junta em tons de ecrã pré-multiplicados: o vermelho do píxel 9 vem do 10, um cinzento
/// `128` a alfa `140`, sobre a cobertura opaca do 9 ⇒ `128 · 140/255 = 70` (em luz seria `97`; com
/// a ida à luz sem volta, `30`).
#[test]
#[ignore = "needs a GPU device"]
fn gpu_o_chroma_junta_em_tons_de_ecra() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (16u32, 2u32);
    let mut px = [0u8, 0, 0, 255].repeat((w * h) as usize);
    for y in 0..h {
        let i = ((y * w + 10) * 4) as usize;
        px[i..i + 4].copy_from_slice(&[128, 128, 128, 140]);
    }
    // `scale = shift / half_diag = −1` ⇒ o píxel x junta o vermelho de `x − (x − 8)·(−1)` = `2x − 8`.
    let half_diag = 0.5 * ((w * w + h * h) as f32).sqrt();
    let got = compoe(
        &gpu,
        w,
        h,
        px,
        espacial(SPATIAL_CHROMA, [-half_diag, 0.0, 0.0]),
    );
    let p9 = &got[(9 * 4) as usize..(9 * 4 + 4) as usize];
    assert!(
        p9[0].abs_diff(70) <= 1,
        "vermelho do píxel 9 = {} ≠ 70",
        p9[0]
    );
    assert_eq!([p9[1], p9[2], p9[3]], [0, 0, 255]);
}
