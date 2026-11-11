//! B29.1 (missão 4K, decisão 3): a medição da precisão da tinta com Flow baixo — o tecto do traço (`m_buf`) e a
//! tela são u8, e um passo `< ½` LSB arredonda para zero: o texel ESTANCA longe do alvo. Sonda (ignored) que imprime
//! o valor da tela contra o alvo analítico `255·Π(1 − flow)`; o gate red→green nasce com a cura.

use super::*;
use crate::blend::BrushBlend;
use crate::falloff::Falloff;

/// `n` dabs no mesmo centro, preto sobre branco, Strength 1, Accumulate off: (tela no centro, alvo).
fn centro(flow: f32, n: u32) -> (u8, f32) {
    let (w, h) = (16u32, 16u32);
    let spec = BrushSpec {
        radius_px: 5.0,
        color: [0.0, 0.0, 0.0],
        blend: BrushBlend::Mix,
        falloff: Falloff::Constant,
        hardness: 1.0,
        strength: 1.0,
        flow,
        ..Default::default()
    };
    let mut dst = vec![255u8; (w * h * 4) as usize];
    let mut mask = vec![0u8; (w * h) as usize];
    for _ in 0..n {
        let _ = stamp_dab_textured_masked(
            &mut dst,
            w,
            h,
            [8.0, 8.0],
            &spec,
            spec.strength,
            false,
            None,
            None,
            None,
            Some(&mut mask),
            None,
            [1.0, 0.0],
            None,
        );
    }
    let alvo = 255.0 * (1.0 - flow).powi(n as i32);
    (dst[(8 * w as usize + 8) * 4], alvo)
}

#[test]
#[ignore = "sonda B29.1: imprime onde a tinta estanca com Flow baixo"]
fn sonda_b29_1_a_tinta_estanca_com_flow_baixo() {
    for (flow, ns) in [(0.02_f32, [50u32, 150, 400]), (0.002, [500, 2000, 4000])] {
        for n in ns {
            let (tela, alvo) = centro(flow, n);
            println!(
                "[b29.1] flow {flow:<5} {n:>5} dabs: tela {tela:>3} · alvo {alvo:>6.1} · erro {:>6.1} níveis",
                f32::from(tela) - alvo
            );
        }
    }
}
