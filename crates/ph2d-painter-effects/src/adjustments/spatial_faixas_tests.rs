//! B19b: os kernels espaciais em faixas (`par_rows`) e o blur de `k` por fora dão o caminho antigo
//! ([`antigo`], verbatim) AO BIT. Tamanhos acima de `1 << 14` px para as faixas correrem de facto.

use super::tonal::{BORROES_ESCALARES, downsample_box};
use super::*;

#[path = "spatial_antigo_tests.rs"]
mod antigo;

/// Pixel determinístico: alfa 0, 1 e intermédio, canais fora de `[0, 1]` de vez em quando.
fn janela(w: u32, h: u32, sal: u32) -> Vec<[f32; 4]> {
    let mut s = 0x9E37_79B9u32 ^ sal;
    let mut r = || {
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        (s >> 8) as f32 / (1u32 << 24) as f32
    };
    (0..w * h)
        .map(|_| {
            let a = match (r() * 5.0) as u32 {
                0 => 0.0,
                1 | 2 => 1.0,
                _ => r(),
            };
            let fora = if r() < 0.05 { 1.3 } else { 1.0 };
            [r() * fora, r(), r() - 0.02, a]
        })
        .collect()
}

fn bits(v: &[[f32; 4]]) -> Vec<[u32; 4]> {
    v.iter().map(|p| p.map(f32::to_bits)).collect()
}

/// `(janela, rótulo)`: tela inteira pequena, sem faixas, e duas com faixas (uma sub-janela).
fn janelas() -> Vec<AdjustWindow> {
    vec![
        AdjustWindow::full(1, 1),
        AdjustWindow::full(7, 3),
        AdjustWindow::full(160, 120),
        AdjustWindow {
            width: 150,
            height: 131,
            origin_x: 37,
            origin_y: 11,
            canvas_width: 400,
            canvas_height: 300,
        },
    ]
}

fn compara(
    novo: impl Fn(&mut [[f32; 4]], AdjustWindow),
    velho: impl Fn(&mut [[f32; 4]], AdjustWindow),
    rotulo: &str,
) {
    for (i, win) in janelas().into_iter().enumerate() {
        let base = janela(win.width, win.height, i as u32 * 7 + 1);
        let (mut a, mut b) = (base.clone(), base);
        novo(&mut a, win);
        velho(&mut b, win);
        assert!(
            bits(&a) == bits(&b),
            "{rotulo}: {win:?} difere do caminho antigo"
        );
    }
}

#[test]
fn o_blur_separavel_de_k_por_fora_e_o_antigo_ao_bit() {
    for radius in [0.4, 1.0, 3.0, 20.0, 300.0] {
        let p = GaussianBlurParams { radius };
        compara(
            |a, w| apply_gaussian(&p, a, w),
            |a, w| antigo::apply_gaussian(&p, a, w),
            &format!("Gaussian r{radius}"),
        );
        for win in janelas() {
            let base: Vec<f32> = janela(win.width, win.height, 3)
                .iter()
                .map(|p| p[1])
                .collect();
            let (mut a, mut b) = (base.clone(), base);
            separable_blur_scalar(radius, &mut a, win);
            antigo::separable_blur_scalar(radius, &mut b, win);
            let (a, b): (Vec<u32>, Vec<u32>) = (
                a.iter().map(|v| v.to_bits()).collect(),
                b.iter().map(|v| v.to_bits()).collect(),
            );
            assert!(a == b, "blur escalar r{radius}: {win:?} difere");
        }
    }
}

#[test]
fn motion_e_chromatic_em_faixas_dao_o_serial_ao_bit() {
    for distance in [0.5, 7.0, 40.0, 600.0] {
        for angle in [0.0, 0.3, 1.570_796_4, 2.7, -1.0] {
            let p = MotionBlurParams { distance, angle };
            compara(
                |a, w| apply_motion_blur(&p, a, w),
                |a, w| antigo::apply_motion_blur(&p, a, w),
                &format!("Motion d{distance} a{angle}"),
            );
        }
    }
    for (r, g, b) in [(4.0, 0.0, -4.0), (0.0, 30.0, 0.0), (-80.0, 12.5, 200.0)] {
        let p = ChromaticAberrationParams {
            red_shift: r,
            green_shift: g,
            blue_shift: b,
            falloff_center: 0.0,
        };
        compara(
            |a, w| apply_chromatic_aberration(&p, a, w),
            |a, w| antigo::apply_chromatic_aberration(&p, a, w),
            &format!("CA {r}/{g}/{b}"),
        );
    }
}

#[test]
fn bloom_downsample_em_faixas_ao_bit() {
    for (w, h, f) in [(7, 3, 2), (320, 240, 2), (641, 483, 4), (333, 251, 32)] {
        let mut src = janela(w as u32, h as u32, 5);
        for p in &mut src {
            p[3] = p[3].clamp(0.0, 1.0);
        }
        let (dw, dh) = ((w + f - 1) / f, (h + f - 1) / f);
        let (a, b) = (
            downsample_box(&src, w, h, dw, dh, f),
            antigo::downsample_box_antigo(&src, w, h, dw, dh, f),
        );
        assert!(bits(&a) == bits(&b), "downsample {w}×{h}/{f} difere");
    }
    for radius in [10.0, 40.0, 100.0] {
        let p = BloomParams {
            threshold: 0.4,
            intensity: 1.5,
            radius,
            falloff: 0.2,
        };
        compara(
            |a, w| apply_bloom(&p, a, w),
            |a, w| antigo::apply_bloom(&p, a, w),
            &format!("Bloom r{radius}"),
        );
    }
}

#[test]
fn s_h_com_raios_iguais_borra_uma_vez() {
    for (rs, rh, borroes) in [(6.0, 6.0, 1), (6.0, 9.0, 2), (0.0, 0.0, 0)] {
        let p = ShadowsHighlightsParams {
            shadows_amount: 0.5,
            shadows_tonal_width: 0.4,
            shadows_radius: rs,
            highlights_amount: 0.3,
            highlights_tonal_width: 0.4,
            highlights_radius: rh,
            color_correction: 0.2,
            midtone_contrast: 0.1,
        };
        compara(
            |a, w| apply_shadows_highlights(&p, a, w),
            |a, w| antigo::apply_shadows_highlights(&p, a, w),
            &format!("S/H {rs}/{rh}"),
        );
        let win = AdjustWindow::full(160, 120);
        let mut a = janela(160, 120, 9);
        BORROES_ESCALARES.with(|c| c.set(0));
        apply_shadows_highlights(&p, &mut a, win);
        let n = BORROES_ESCALARES.with(std::cell::Cell::get);
        assert_eq!(n, borroes, "S/H {rs}/{rh}: {n} borrões escalares");
    }
}

/// Sonda B19b (4K, mesmo processo, ABBA, o MÍNIMO): `--profile smoke -- --ignored --nocapture`.
#[test]
#[ignore = "sonda de velocidade"]
fn mede_os_kernels_espaciais_4k() {
    type K = Box<dyn Fn(&mut [[f32; 4]], AdjustWindow)>;
    let g = GaussianBlurParams { radius: 30.0 };
    let m = MotionBlurParams {
        distance: 200.0,
        angle: 0.5,
    };
    let ca = ChromaticAberrationParams {
        red_shift: 12.0,
        green_shift: 0.0,
        blue_shift: -12.0,
        falloff_center: 0.0,
    };
    let b = BloomParams {
        threshold: 0.4,
        intensity: 1.0,
        radius: 40.0,
        falloff: 0.2,
    };
    let sh = ShadowsHighlightsParams {
        shadows_amount: 0.5,
        shadows_tonal_width: 0.4,
        shadows_radius: 20.0,
        highlights_amount: 0.3,
        highlights_tonal_width: 0.4,
        highlights_radius: 20.0,
        color_correction: 0.2,
        midtone_contrast: 0.1,
    };
    let casos: Vec<(&str, K, K)> = vec![
        (
            "Gaussian r30",
            Box::new(move |a, w| apply_gaussian(&g, a, w)),
            Box::new(move |a, w| antigo::apply_gaussian(&g, a, w)),
        ),
        (
            "Motion d200",
            Box::new(move |a, w| apply_motion_blur(&m, a, w)),
            Box::new(move |a, w| antigo::apply_motion_blur(&m, a, w)),
        ),
        (
            "Chromatic",
            Box::new(move |a, w| apply_chromatic_aberration(&ca, a, w)),
            Box::new(move |a, w| antigo::apply_chromatic_aberration(&ca, a, w)),
        ),
        (
            "Bloom r40",
            Box::new(move |a, w| apply_bloom(&b, a, w)),
            Box::new(move |a, w| antigo::apply_bloom(&b, a, w)),
        ),
        (
            "S/H r20=r20",
            Box::new(move |a, w| apply_shadows_highlights(&sh, a, w)),
            Box::new(move |a, w| antigo::apply_shadows_highlights(&sh, a, w)),
        ),
    ];
    let win = AdjustWindow::full(3840, 2160);
    let base = janela(3840, 2160, 11);
    for (nome, novo, velho) in &casos {
        let mut min = [f64::MAX; 2];
        for ordem in [[1, 0], [0, 1], [0, 1], [1, 0]] {
            for v in ordem {
                let mut a = base.clone();
                let t = std::time::Instant::now();
                if v == 0 {
                    velho(&mut a, win)
                } else {
                    novo(&mut a, win)
                }
                min[v] = min[v].min(t.elapsed().as_secs_f64() * 1e3);
            }
        }
        println!(
            "B19b {nome}: antigo {:.1} ms → novo {:.1} ms",
            min[0], min[1]
        );
    }
}
