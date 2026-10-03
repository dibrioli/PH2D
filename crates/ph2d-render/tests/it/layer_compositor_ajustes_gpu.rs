//! ADR-0177, P3 — os ajustes na placa: leis ABSOLUTAS (não contra o espelho da CPU do teste, que
//! muda junto com o shader). Irmão de `layer_compositor_gpu` (os ajudantes vêm de lá).

use super::layer_compositor_gpu::{MapProvider, try_headless_gpu};
use ph2d_render::{LayerCompositor, LayerOp, Region};

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
