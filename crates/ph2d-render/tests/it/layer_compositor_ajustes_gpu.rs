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

/// ⭐ O Threshold é a regra de 8 bits do Photoshop — o BYTE da luma `≥ threshold`: o cinzento `128`
/// no limiar `128` sai branco, o `127` preto. O corte em `threshold/255` mandava o `128` para o
/// preto (os pesos Rec.601 somam um nadinha abaixo de 1 em `f32`).
#[test]
#[ignore = "needs a GPU device"]
fn gpu_o_threshold_e_o_byte_da_luma_contra_o_limiar() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (16u32, 8u32);
    let mut tela = Vec::new();
    for i in 0..w * h {
        let v = if i % 2 == 0 { 128u8 } else { 127 };
        tela.extend_from_slice(&[v, v, v, 255]);
    }
    let mut prov = MapProvider::default();
    prov.insert(1, 1, tela.clone());
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
    for (i, (p, e)) in got
        .as_chunks::<4>()
        .0
        .iter()
        .zip(tela.as_chunks::<4>().0)
        .enumerate()
    {
        let quer = if e[0] == 128 { 255 } else { 0 };
        assert_eq!(*p, [quer, quer, quer, 255], "píxel {i} (cinzento {})", e[0]);
    }
}
