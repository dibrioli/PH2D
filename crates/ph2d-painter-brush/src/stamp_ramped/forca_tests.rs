//! B25.5 (missão 4K, N10): a rampa em cache escreve o plano da força das macros — o MESMO que `blit_stamp` escreve
//! com a mesma máscara e cobertura — e a tinta não muda por o ter.

use crate::{BrushSpec, RampAlphaMode, blit_stamp, blit_stamp_ramped};

const W: u32 = 96;

fn mask(spec: &BrushSpec) -> crate::StampMask {
    crate::stamp::render_stamp_mask(spec, None, None, None, 64)
}

#[test]
fn a_rampa_escreve_a_forca_do_carimbo_e_a_mesma_tinta() {
    let n = (W * W) as usize;
    let spec = BrushSpec {
        radius_px: 20.0,
        hardness: 0.4,
        ..BrushSpec::default()
    };
    let m = mask(&spec);
    let rampa = vec![[0.2, 0.4, 0.8, 1.0]; 256];
    let mut comparados = 0usize;
    // Dois toques no mesmo plano (a força compõe por cima) e uma banda única × as bandas da máquina.
    for alpha_locked in [false, true] {
        let (mut tela_c, mut tela_r, mut tela_s) =
            (vec![128u8; n * 4], vec![128u8; n * 4], vec![128u8; n * 4]);
        let (mut f_c, mut f_r) = (vec![0u16; n], vec![0u16; n]);
        for (centro, cobertura) in [([40.3, 47.7], 0.6), ([52.1, 44.9], 0.35)] {
            let _ = blit_stamp(
                &mut tela_c,
                W,
                W,
                centro,
                20.0,
                &m,
                &spec,
                cobertura,
                alpha_locked,
                Some(&mut f_c),
            );
            let (r, s) = (RampAlphaMode::Strength, &rampa);
            let _ = blit_stamp_ramped(
                &mut tela_r,
                W,
                W,
                centro,
                20.0,
                &m,
                &spec,
                cobertura,
                alpha_locked,
                s,
                r,
                Some(&mut f_r),
            );
            let _ = blit_stamp_ramped(
                &mut tela_s,
                W,
                W,
                centro,
                20.0,
                &m,
                &spec,
                cobertura,
                alpha_locked,
                s,
                r,
                None,
            );
        }
        assert!(
            f_c.iter().any(|&f| f > 0),
            "o controlo: o carimbo mediu força"
        );
        let difere = f_c.iter().zip(&f_r).filter(|(a, b)| a != b).count();
        assert_eq!(
            difere, 0,
            "alpha_locked={alpha_locked}: {difere} texels de força diferem do blit_stamp"
        );
        assert_eq!(tela_r, tela_s, "medir a força mudou a tinta da rampa");
        comparados += n;
    }
    assert_eq!(comparados, 2 * (W * W) as usize);
}

/// O Texture Alpha baixa o alfa em vez de pintar por cima: não mede (o plano fica intacto).
#[test]
fn o_texture_alpha_nao_mede_a_forca() {
    let n = (W * W) as usize;
    let spec = BrushSpec {
        radius_px: 20.0,
        ..BrushSpec::default()
    };
    let m = mask(&spec);
    let rampa = vec![[0.2, 0.4, 0.8, 0.5]; 256];
    let (mut tela, mut f) = (vec![128u8; n * 4], vec![0u16; n]);
    let r = RampAlphaMode::TextureAlpha;
    let _ = blit_stamp_ramped(
        &mut tela,
        W,
        W,
        [48.0, 48.0],
        20.0,
        &m,
        &spec,
        0.8,
        false,
        &rampa,
        r,
        Some(&mut f),
    );
    assert!(tela.iter().any(|&b| b != 128), "o controlo: pintou");
    assert!(f.iter().all(|&x| x == 0));
}
