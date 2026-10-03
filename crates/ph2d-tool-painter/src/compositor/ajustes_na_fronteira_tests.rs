//! ADR-0177, P3: cada ajuste NEUTRO deixa o composto igual AO BIT — o acumulador vive em tons de
//! ecrã e o ajuste definido noutro espaço converte na fronteira dele.

use super::*;
use crate::layers::LayerStack;
use ph2d_painter_effects::adjustments::AdjustmentKind;

const W: u32 = 64;
const H: u32 = 48;

/// Uma tela variada (nenhum píxel igual), com o alfa a variar também se `opaca` for falso.
fn variada(seed: u32, opaca: bool) -> LayerImage {
    let mut v = vec![0u8; (W * H * 4) as usize];
    for (i, px) in v.chunks_exact_mut(4).enumerate() {
        let p = i as u32;
        px[0] = (p.wrapping_mul(37).wrapping_add(seed * 11) % 256) as u8;
        px[1] = (p.wrapping_mul(91).wrapping_add(seed * 29) % 256) as u8;
        px[2] = (p.wrapping_mul(53).wrapping_add(seed * 7) % 256) as u8;
        px[3] = if opaca {
            255
        } else {
            (p.wrapping_mul(17).wrapping_add(seed) % 256) as u8
        };
    }
    LayerImage {
        width: W,
        height: H,
        rgba8: v,
    }
}

/// O composto da pilha base + camada (translúcidas), com um ajuste neutro `kind` por cima (ou sem).
fn compoe_com(kind: Option<AdjustmentKind>, base_opaca: bool) -> Vec<u8> {
    let mut s = LayerStack::new();
    let b = s.add_raster("base", W, H).unwrap();
    let t = s.add_raster("topo", W, H).unwrap();
    s.set_opacity(t, 0.7);
    if let Some(k) = kind {
        s.add_adjustment(k).unwrap();
    }
    let mut src = MapPixelSource::default();
    src.insert(b, variada(1, base_opaca));
    src.insert(t, variada(2, false));
    composite(&s, &src, W, H)
}

/// Os tipos sem ponto neutro: o «padrão» deles já faz algo, por definição.
const SEM_NEUTRO: [AdjustmentKind; 6] = [
    AdjustmentKind::GradientMap,
    AdjustmentKind::Halftone,
    AdjustmentKind::Posterize,
    AdjustmentKind::Threshold,
    AdjustmentKind::Invert,
    AdjustmentKind::BlackAndWhite,
];

/// ⭐ Cada ajuste no seu ponto NEUTRO é um no-op AO BIT sobre o composto em tons de ecrã — com a
/// base opaca e translúcida (medido a 03/10: 0 bytes nos 18 tipos que têm neutro).
#[test]
fn cada_ajuste_neutro_e_um_no_op_ao_bit() {
    let mut vistos = 0;
    for opaca in [true, false] {
        let sem = compoe_com(None, opaca);
        for k in AdjustmentKind::ALL
            .into_iter()
            .filter(|k| !SEM_NEUTRO.contains(k))
        {
            assert_eq!(compoe_com(Some(k), opaca), sem, "{k:?}, base opaca {opaca}");
            vistos += 1;
        }
    }
    assert_eq!(vistos, 2 * 18);
}

/// ⭐ Um ajuste definido EM LUZ vê a luz (a fronteira do ADR-0177): `+1 EV` sobre o cinzento `128`
/// dobra a luz (`0,2158 → 0,4317`) e volta como `176` — não dobra o valor codificado (`255`).
#[test]
fn um_ajuste_definido_em_luz_ve_a_luz() {
    use ph2d_painter_effects::adjustments::{AdjustmentParams, ExposureParams};
    let mut s = LayerStack::new();
    let b = s.add_raster("base", 1, 1).unwrap();
    let a = s.add_adjustment(AdjustmentKind::Exposure).unwrap();
    if let Some(LayerKind::Adjustment(adj)) = s.get_mut(a).map(|l| &mut l.kind) {
        adj.params = AdjustmentParams::Exposure(ExposureParams {
            exposure_ev: 1.0,
            offset: 0.0,
            gamma_correction: 0.0,
        });
    }
    let mut src = MapPixelSource::default();
    src.insert(
        b,
        LayerImage {
            width: 1,
            height: 1,
            rgba8: vec![128, 128, 128, 255],
        },
    );
    let out = composite(&s, &src, 1, 1);
    for (c, &v) in out[..3].iter().enumerate() {
        assert!(v.abs_diff(176) <= 1, "canal {c}: {v} (esperado ~176)");
    }
}

/// Sonda: por tipo, quantos bytes um ajuste NEUTRO move (e o pior), base opaca e translúcida.
#[test]
#[ignore = "sonda: imprime a tabela"]
fn diag_o_ajuste_neutro_por_tipo() {
    for opaca in [true, false] {
        let sem = compoe_com(None, opaca);
        for k in AdjustmentKind::ALL {
            let com = compoe_com(Some(k), opaca);
            let n = sem.iter().zip(&com).filter(|(a, b)| a != b).count();
            let pior = sem
                .iter()
                .zip(&com)
                .map(|(a, b)| a.abs_diff(*b))
                .max()
                .unwrap();
            println!("base opaca {opaca:<5} {k:<24?} {n:>5} bytes (pior {pior})");
        }
    }
}
