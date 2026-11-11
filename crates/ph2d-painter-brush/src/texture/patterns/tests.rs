//! Tests for the procedural texture samplers: per-kind boundedness + determinism (HR-5), the shape
//! knobs, and the image-backed bilinear sampler. Moved to a sibling file to keep `patterns.rs` under
//! the LOC cap.

use super::*;

#[test]
fn paper_presets_are_distinct_and_tooth_like() {
    // Sample each paper preset over a patch at its spec defaults; a paper tooth must sit mid-band (not a
    // flat wash of black or white) and the three surfaces must be genuinely different fields.
    fn stats(kind: TextureKind) -> (f32, f32, Vec<f32>) {
        let mut params = [0.5f32; 6];
        for (slot, s) in param_specs(kind).iter().enumerate() {
            params[slot] = s.default;
        }
        let mut vals = Vec::new();
        for i in 0..48 {
            for j in 0..48 {
                vals.push(sample_kind(
                    kind,
                    [i as f32 * 0.21, j as f32 * 0.23],
                    params,
                    None,
                ));
            }
        }
        let mean = vals.iter().sum::<f32>() / vals.len() as f32;
        let var = vals.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / vals.len() as f32;
        (mean, var, vals)
    }
    let kinds = [
        TextureKind::PaperCold,
        TextureKind::PaperRough,
        TextureKind::PaperHot,
    ];
    let all: Vec<_> = kinds.into_iter().map(stats).collect();
    for (kind, (mean, var, _)) in kinds.iter().zip(&all) {
        assert!(
            (0.2..0.8).contains(mean),
            "{} tooth should sit mid-band, mean {mean}",
            kind.name_key()
        );
        assert!(
            *var > 1e-4,
            "{} must have tooth relief (variance), got {var}",
            kind.name_key()
        );
    }
    // The three surfaces differ: at least one sampled pixel diverges between each pair.
    for a in 0..all.len() {
        for b in (a + 1)..all.len() {
            let diff = all[a]
                .2
                .iter()
                .zip(&all[b].2)
                .any(|(x, y)| (x - y).abs() > 1e-3);
            assert!(
                diff,
                "papers {} and {} are identical",
                kinds[a].name_key(),
                kinds[b].name_key()
            );
        }
    }
}

#[test]
fn checker_alternates_by_cell_parity() {
    // Hard checker (Softness 0) reproduces the integer-cell parity.
    let hard = &[0.0f32];
    assert_eq!(checker(0.5, 0.5, hard), 0.0);
    assert_eq!(checker(1.5, 0.5, hard), 1.0);
    assert_eq!(checker(1.5, 1.5, hard), 0.0);
    assert_eq!(checker(-0.5, 0.5, hard), 1.0); // floor (not truncation): cell -1 has parity 1
}

#[test]
fn stripes_width_controls_coverage() {
    // k = [Width, Frequency(0.35 ≈ 1×), Softness].
    let s = |u: f32, w: f32| stripes(u, 0.0, &[w, 0.35, 0.3]);
    assert!(s(0.5, 0.5) > 0.9, "band centre is covered");
    let cov = |w: f32| (0..100).map(|i| s(i as f32 / 100.0, w)).sum::<f32>();
    assert!(
        cov(0.8) > cov(0.3),
        "a wider Width paints more of the period"
    );
    assert!((s(0.25, 0.5) - s(1.25, 0.5)).abs() < 1e-6, "periodic");
}

#[test]
fn voronoi_randomness_and_edges_change_the_field() {
    let cells = &[1.0, 0.0, 0.0, 0.0]; // F1 cells
    let cracks = &[1.0, 0.0, 0.0, 1.0]; // F2−F1 edges
    let (mut diff, mut n) = (0.0f32, 0.0f32);
    for i in 0..60 {
        let (u, v) = (i as f32 * 0.41, i as f32 * 0.29);
        let a = voronoi_t(u, v, cells, 0, 0);
        let b = voronoi_t(u, v, cracks, 0, 0);
        assert!(
            (0.0..=1.0).contains(&a) && (0.0..=1.0).contains(&b),
            "bounded"
        );
        diff += (a - b).abs();
        n += 1.0;
    }
    assert!(diff / n > 0.05, "the Edges knob produces a different field");
}

#[test]
fn value_noise_is_bounded_and_deterministic() {
    for i in 0..50 {
        let (u, v) = (i as f32 * 0.37, i as f32 * -0.21);
        let a = value_noise(u, v);
        assert!((0.0..=1.0).contains(&a), "noise out of range: {a}");
        assert_eq!(a, value_noise(u, v), "noise must be a pure function");
    }
    assert_ne!(value_noise(0.5, 0.5), value_noise(10.5, 10.5));
}

#[test]
fn sample_image_is_centred_bilinear_and_tiles() {
    let lum = [0u8, 255, 128, 64]; // 2×2: row0 [0,255], row1 [128,64]
    let img = ImageMask {
        lum: &lum,
        width: 2,
        height: 2,
    };
    // Centred convention (Enio 2026-06-24): `u = 0` is the image CENTRE; the footprint −1..1 spans one
    // image at size 1. texel (0,0) centre sits at `u=v=-0.5`, texel (1,0) at `u=0.5, v=-0.5`.
    assert!(
        (sample_image(&img, -0.5, -0.5) - 0.0).abs() < 1e-6,
        "texel (0,0)=0"
    );
    assert!(
        (sample_image(&img, 0.5, -0.5) - 1.0).abs() < 1e-6,
        "texel (1,0)=255"
    );
    // The origin is the image centre → bilinear blend of all four texels (their mean).
    let mean = (0.0 + 255.0 + 128.0 + 64.0) / 4.0 / 255.0;
    assert!(
        (sample_image(&img, 0.0, 0.0) - mean).abs() < 1e-6,
        "centre = 4-texel mean"
    );
    // Tiles with period 2 in footprint units (one image spans 2 units).
    assert!((sample_image(&img, 1.5, -0.5) - sample_image(&img, -0.5, -0.5)).abs() < 1e-6);
    for k in 0..30 {
        assert!((0.0..=1.0).contains(&sample_image(&img, k as f32 * 0.17, -(k as f32) * 0.17)));
    }
}

#[test]
fn every_kind_is_bounded_and_deterministic() {
    // Neutral params + two non-neutral sets (all six slots), across every kind.
    for params in [
        [0.5; 6],
        [0.9, 0.3, 0.8, 0.5, 0.2, 0.7],
        [0.1, 0.7, 0.2, 0.5, 0.9, 0.3],
    ] {
        for k in 0..TextureKind::COUNT {
            let kind = TextureKind::from_u8(k);
            for i in 0..40 {
                let (u, v) = (i as f32 * 0.31 - 3.0, i as f32 * -0.27 + 1.5);
                let a = sample_kind(kind, [u, v], params, None);
                assert!(
                    (0.0..=1.0).contains(&a),
                    "{} out of range: {a}",
                    kind.name_key()
                );
                assert_eq!(
                    a,
                    sample_kind(kind, [u, v], params, None),
                    "{} must be pure",
                    kind.name_key()
                );
            }
        }
    }
}

#[test]
fn every_procedural_kind_varies_across_the_plane() {
    // Each procedural (not None / Image) must produce a non-flat field at its defaults.
    for k in 1..TextureKind::COUNT {
        let kind = TextureKind::from_u8(k);
        if matches!(kind, TextureKind::Image) {
            continue;
        }
        // Seed each slot from the kind's spec defaults (slots beyond the specs stay neutral).
        let mut params = [0.5f32; 6];
        for (slot, s) in param_specs(kind).iter().enumerate() {
            params[slot] = s.default;
        }
        let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
        for i in 0..72 {
            for j in 0..72 {
                let a = sample_kind(kind, [i as f32 * 0.17, j as f32 * 0.19], params, None);
                lo = lo.min(a);
                hi = hi.max(a);
            }
        }
        assert!(
            hi - lo > 0.05,
            "{} should vary (lo={lo} hi={hi})",
            kind.name_key()
        );
    }
}

#[test]
fn knob_defaults_to_neutral_past_the_slice() {
    assert_eq!(knob(&[], 0), 0.5);
    assert_eq!(knob(&[0.2], 0), 0.2);
    assert_eq!(knob(&[0.2], 3), 0.5, "out-of-range slot is neutral");
}

#[test]
fn texture_preview_is_grayscale_opaque_and_varies() {
    let (w, h) = (24u32, 16u32);
    let mut buf = vec![0u8; (w * h * 4) as usize];
    let mut params = [0.5f32; 6];
    params[2] = 0.0; // hard checker → crisp dark/light cells
    render_texture_preview(
        TextureKind::Checker,
        params,
        [1.0, 1.0],
        [0.0, 0.0],
        0,
        None,
        None,
        &mut buf,
        w,
        h,
    );
    let (mut dark, mut light) = (0, 0);
    for px in buf.as_chunks::<4>().0 {
        assert_eq!(px[3], 255, "preview pixels are opaque");
        assert!(px[0] == px[1] && px[1] == px[2], "preview is grayscale");
        if px[0] < 40 {
            dark += 1;
        } else if px[0] > 215 {
            light += 1;
        }
    }
    assert!(dark > 0 && light > 0, "checker preview shows both cells");
    // A too-small buffer is a no-op (must not panic or write OOB).
    let mut tiny = vec![7u8; 4];
    render_texture_preview(
        TextureKind::Checker,
        params,
        [1.0, 1.0],
        [0.0, 0.0],
        0,
        None,
        None,
        &mut tiny,
        w,
        h,
    );
    assert_eq!(tiny, [7u8; 4], "too-small buffer left untouched");
}

#[test]
fn texture_preview_colorizes_with_the_ramp_and_shows_alpha() {
    use crate::ramp_alpha::RampAlphaMode;
    let (w, h) = (32u32, 24u32);
    let mut params = [0.5f32; 6];
    params[2] = 0.0; // hard checker → s is exactly 0 or 1
    // LUT: opaque RED at s=0 … TRANSPARENT green at s=1 (sRGB-straight RGBA).
    let mut lut = [[0.0f32; 4]; 256];
    for (i, c) in lut.iter_mut().enumerate() {
        let t = i as f32 / 255.0;
        *c = [1.0 - t, t, 0.0, 1.0 - t]; // red→green, alpha 1→0
    }
    let render = |mode| {
        let mut buf = vec![0u8; (w * h * 4) as usize];
        render_texture_preview(
            TextureKind::Checker,
            params,
            [1.0, 1.0],
            [0.0, 0.0],
            0,
            None,
            Some((&lut[..], mode)),
            &mut buf,
            w,
            h,
        );
        buf
    };
    // Sprite-alpha mode: the s=1 (transparent) cells composite over the checker → NOT pure green;
    // the s=0 cells are opaque red.
    let sprite = render(RampAlphaMode::TextureAlpha);
    let mut reddish = 0;
    let mut pure_green = 0;
    for px in sprite.as_chunks::<4>().0 {
        if px[0] > 150 && px[1] < 90 {
            reddish += 1;
        }
        if px[0] < 40 && px[1] > 215 {
            pure_green += 1;
        }
    }
    assert!(reddish > 0, "the opaque ramp end paints red");
    assert_eq!(
        pure_green, 0,
        "the transparent ramp end is composited over the checker, never pure green"
    );
    // Off mode: alpha ignored → the s=1 cells ARE pure-ish green (no checker bleed).
    let off = render(RampAlphaMode::None);
    let off_green = off
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|px| px[0] < 40 && px[1] > 215)
        .count();
    assert!(off_green > 0, "Off mode shows the opaque green end");
}

/// **GRAN-2 regression (doc 12, medido por FFT): o tooth dos papéis é MID-BAND, não blotch.**
/// A métrica da auditoria: `std(médias de blocos 32px) / std(campo)` — o gerador antigo media
/// **91,7%** (a energia era macro-mancha, λ > 32 px → o wash saía "mottled"); um tooth de verdade
/// concentra energia em 2-32 px, então as médias de bloco são quase uniformes.
#[test]
fn paper_tiles_are_mid_band_tooth_not_low_frequency_blotch() {
    for kind in [
        TextureKind::PaperCold,
        TextureKind::PaperRough,
        TextureKind::PaperHot,
    ] {
        let mut params = [0.5f32; 6];
        for (slot, s) in param_specs(kind).iter().enumerate() {
            params[slot] = s.default;
        }
        // 256² patch at 1 texel/px (Size 1): u = i/256 spans exactly one tile.
        const N: usize = 256;
        let mut field = vec![0.0f32; N * N];
        for j in 0..N {
            for i in 0..N {
                field[j * N + i] = sample_kind(
                    kind,
                    [i as f32 / N as f32, j as f32 / N as f32],
                    params,
                    None,
                );
            }
        }
        let mean = field.iter().sum::<f32>() / (N * N) as f32;
        let std =
            (field.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / (N * N) as f32).sqrt();
        assert!(
            std > 0.03,
            "{} needs real tooth relief (std {std})",
            kind.name_key()
        );
        // 32-px block means.
        let mut blocks = Vec::new();
        for bj in 0..(N / 32) {
            for bi in 0..(N / 32) {
                let mut acc = 0.0f32;
                for j in 0..32 {
                    for i in 0..32 {
                        acc += field[(bj * 32 + j) * N + (bi * 32 + i)];
                    }
                }
                blocks.push(acc / 1024.0);
            }
        }
        let bmean = blocks.iter().sum::<f32>() / blocks.len() as f32;
        let bstd = (blocks
            .iter()
            .map(|v| (v - bmean) * (v - bmean))
            .sum::<f32>()
            / blocks.len() as f32)
            .sqrt();
        let ratio = bstd / std;
        assert!(
            ratio < 0.35,
            "{}: block-mean std / field std = {ratio:.3} — low-frequency blotch (the old generator \
             measured 0.917); the tooth must live in the 2-32 px band",
            kind.name_key()
        );
    }
}

/// Correlação de Pearson do campo `f` (lado `n`) consigo mesmo deslocado `lag` px em x (`eixo 0`) ou y.
fn autocorrelacao(f: &[f32], n: usize, lag: usize, eixo: u8) -> f64 {
    let (mut sa, mut sb, mut saa, mut sbb, mut sab, mut c) = (0f64, 0f64, 0f64, 0f64, 0f64, 0f64);
    for y in 0..n - if eixo == 1 { lag } else { 0 } {
        for x in 0..n - if eixo == 0 { lag } else { 0 } {
            let a = f64::from(f[y * n + x]);
            let b = f64::from(if eixo == 0 {
                f[y * n + x + lag]
            } else {
                f[(y + lag) * n + x]
            });
            (sa, sb, saa, sbb, sab, c) = (
                sa + a,
                sb + b,
                saa + a * a,
                sbb + b * b,
                sab + a * b,
                c + 1.0,
            );
        }
    }
    let cov = sab / c - (sa / c) * (sb / c);
    cov / ((saa / c - (sa / c).powi(2)).sqrt() * (sbb / c - (sb / c).powi(2)).sqrt())
}

/// ⭐ **B33 (decisão 7b): o papel não se repete a 256 px.** Num campo de 1024² a Size 1, a autocorrelação nos
/// desfasamentos 256 · 512 (o ladrilho) e 241 (a segunda leitura) fica longe de 1 em x e em y, e a média e o
/// desvio (o fundo do dente) ficam os do preset. Controlo: a leitura única de antes mede ≥ 0,98 a 256 (o vermelho).
#[test]
fn o_papel_nao_se_repete_a_256_px() {
    const N: usize = 1024;
    let campo = |f: &dyn Fn(f32, f32) -> f32| -> Vec<f32> {
        (0..N * N)
            .map(|i| f((i % N) as f32 / 256.0, (i / N) as f32 / 256.0))
            .collect()
    };
    let media_desvio = |f: &[f32]| {
        let m = f.iter().map(|&v| f64::from(v)).sum::<f64>() / f.len() as f64;
        let d =
            (f.iter().map(|&v| (f64::from(v) - m).powi(2)).sum::<f64>() / f.len() as f64).sqrt();
        (m, d)
    };
    for which in 0..3 {
        let antes = campo(&|u, v| paper_tile::sample_uma_leitura(which, u, v));
        let depois = campo(&|u, v| paper_tile::sample(which, u, v));
        for eixo in [0u8, 1] {
            let r0 = autocorrelacao(&antes, N, 256, eixo);
            assert!(
                r0 > 0.98,
                "papel {which} eixo {eixo}: o controlo devia repetir a 256 (r = {r0:.3})"
            );
            for lag in [256usize, 512, 241] {
                let r = autocorrelacao(&depois, N, lag, eixo);
                assert!(
                    r < 0.6,
                    "papel {which} eixo {eixo}: repete a {lag} px (r = {r:.3}, antes {r0:.3})"
                );
            }
        }
        let ((ma, da), (md, dd)) = (media_desvio(&antes), media_desvio(&depois));
        assert!(
            (ma - md).abs() < 0.01,
            "papel {which}: média {ma:.4} → {md:.4}"
        );
        assert!(
            (dd / da - 1.0).abs() < 0.10,
            "papel {which}: desvio {da:.4} → {dd:.4}"
        );
    }
}

/// Bug #59: os Bricks pela área são a média do degrau no píxel (a lei dos Dots), e `fw = 0` é o degrau de antes.
#[test]
fn os_bricks_pela_area_sao_a_media_do_degrau_no_pixel() {
    let params = [0.0, 1.0, 0.5, 0.5, 0.5, 0.5];
    let k = &params[2..];
    let ponto = |u: f32, v: f32| bricks(u, v, k, 0.0);
    let mut na_borda = 0;
    for i in 0..400 {
        let (u, v) = (0.137 + i as f32 * 0.0171, 0.291 + i as f32 * 0.0093);
        for fw in [0.05f32, 0.2] {
            // Uma fila só: o Bond muda de fila para fila e a área lê a do centro do píxel.
            if ifloor(v - fw * 0.5) != ifloor(v + fw * 0.5) {
                continue;
            }
            let n = 64;
            let mut soma = 0.0;
            for a in 0..n {
                for b in 0..n {
                    let d = |j: i32| (j as f32 + 0.5) / n as f32 - 0.5;
                    soma += ponto(u + d(a) * fw, v + d(b) * fw);
                }
            }
            let media = soma / (n * n) as f32;
            let area = bricks(u, v, k, fw);
            assert!(
                (area - media).abs() < 0.04,
                "({u}, {v}) fw {fw}: área {area}, média {media}"
            );
            na_borda += usize::from(area > 0.05 && area < 0.95);
        }
    }
    assert!(
        na_borda > 20,
        "a amostra quase não pisou a borda ({na_borda})"
    );
}

/// Bug #59: a pré-visualização do grão lê os degraus pela área, como a tela: há tons de borda onde o ponto só dava dois.
#[test]
fn a_previsualizacao_le_os_degraus_pela_area() {
    let (w, h) = (120u32, 60u32);
    for kind in [TextureKind::Dots, TextureKind::Bricks] {
        let params = [0.5f32, 0.5, 0.5, 0.0, 0.0, 0.5];
        let mut buf = vec![0u8; (w * h * 4) as usize];
        render_texture_preview(
            kind,
            params,
            [1.0, 1.0],
            // Fora da grelha dos píxeis: a 120 px por 3 células a borda do tijolo cai certa entre dois.
            [0.0071, 0.0033],
            0,
            None,
            None,
            &mut buf,
            w,
            h,
        );
        let meios = buf
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| p[0] > 10 && p[0] < 245)
            .count();
        assert!(meios > 20, "{kind:?}: só {meios} píxeis de borda na faixa");
    }
}
