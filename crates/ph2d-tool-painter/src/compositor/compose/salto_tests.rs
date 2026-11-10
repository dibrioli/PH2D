//! G4 of B19.10/B19.11: the α = 0 skip in `blend_window` is `apply_blend` to the bit, and every term
//! of its guard is needed (a control per term where skipping would change the pixel).

use super::*;
use ph2d_painter_effects::MAX_BLEND_MODES;

const UM_MAIS: f32 = 1.000_000_1;

fn bits(p: [f32; 4]) -> [u32; 4] {
    p.map(f32::to_bits)
}

/// ORACLE — `blend_window` as it was before the skip, verbatim but for the name.
#[allow(clippy::too_many_arguments)]
fn blend_window_antigo(
    acc: &mut [[f32; 4]],
    rx: u32,
    ry: u32,
    rw: u32,
    rh: u32,
    mode: BlendMode,
    opacity: f32,
    sample: impl Fn(u32, u32) -> [f32; 4],
) {
    for ly in 0..rh {
        for lx in 0..rw {
            let mut s = sample(rx + lx, ry + ly);
            s[3] *= opacity;
            let i = (ly * rw + lx) as usize;
            acc[i] = apply_blend(mode, acc[i], s);
        }
    }
}

/// ⭐ Wherever the guard lets a transparent `Normal` source skip, `apply_blend` returns `dst` to the
/// bit — over a grid with −0, NaN, ±∞, out-of-range channels, αb ∈ (0, 1), αb > 1 and α ∈ {+0, −0, <0}.
#[test]
fn o_salto_alfa_zero_e_o_apply_ao_bit() {
    let canais = [
        0.0f32,
        -0.0,
        1e-30,
        0.3,
        0.5,
        0.7,
        1.0,
        UM_MAIS,
        -1e-30,
        f32::NAN,
        f32::INFINITY,
    ];
    let alfas_d = [
        0.0f32,
        -0.0,
        1e-30,
        0.3,
        0.999_999_9,
        1.0,
        UM_MAIS,
        f32::NAN,
    ];
    let alfas_s = [0.0f32, -0.0, -1e-30, -1.0];
    let cores_s = [0.0f32, -0.0, 0.42, 1.0, 3.0, f32::NAN, f32::NEG_INFINITY];
    let mut saltos = 0usize;
    for &c0 in &canais {
        for &c1 in &[0.0f32, 0.61, -0.0, f32::NAN] {
            for &ad in &alfas_d {
                for &cs in &cores_s {
                    for &as_ in &alfas_s {
                        let s = [cs, 0.25, cs, as_];
                        for d in [[c0, c1, 0.9, ad], [c0, 0.0, c1, ad], [0.0, 0.0, 0.0, ad]] {
                            if s[3] <= 0.0 && normal_transparente_e_identidade(d, s) {
                                saltos += 1;
                                assert_eq!(
                                    bits(apply_blend(BlendMode::Normal, d, s)),
                                    bits(d),
                                    "d {d:?} s {s:?}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    assert!(saltos > 500, "the grid must exercise the skip ({saltos})");
    // CONTROLS: each guard term rejects a pixel that `apply_blend` WOULD change.
    let s0 = [0.42f32, 0.25, 0.8, 0.0];
    let muda = |d: [f32; 4], s: [f32; 4], o_que: &str| {
        assert!(!normal_transparente_e_identidade(d, s), "{o_que}: guard");
        assert_ne!(
            bits(apply_blend(BlendMode::Normal, d, s)),
            bits(d),
            "{o_que}"
        );
    };
    muda([-0.0, 0.5, 0.5, 1.0], s0, "−0 channel");
    muda([f32::NAN, 0.5, 0.5, 1.0], s0, "NaN channel");
    muda([UM_MAIS, 0.5, 0.5, 1.0], s0, "channel > 1");
    muda([0.5, 0.5, 0.5, UM_MAIS], s0, "αb > 1");
    muda([0.5, 0.5, 0.5, 0.0], s0, "αb = 0 with colour");
    muda([0.5, 0.5, 0.5, -0.0], s0, "αb = −0");
    muda(
        [0.5, 0.5, 0.5, 1.0],
        [f32::NAN, 0.25, 0.8, 0.0],
        "NaN source",
    );
    muda(
        [0.5, 0.5, 0.5, 1.0],
        [f32::INFINITY, 0.25, 0.8, 0.0],
        "∞ source",
    );
    // αb ∈ (0, 1): the un-premultiply does not round-trip every channel.
    let parcial = (1..=255u8).any(|b| {
        let d = [decode_byte(b), 0.5, 0.5, 0.3];
        apply_blend(BlendMode::Normal, d, s0)[0].to_bits() != d[0].to_bits()
    });
    assert!(
        parcial,
        "αb = 0.3 must have a channel the skip would get wrong"
    );
    assert!(!normal_transparente_e_identidade([0.5, 0.5, 0.5, 0.3], s0));
}

/// The skipping `blend_window` IS the old loop to the bit, in every mode and opacity, over windows
/// mixing opaque, partial, zeroed and corrupt accumulator pixels with transparent and opaque sources.
#[test]
fn a_janela_com_salto_e_o_laco_antigo_ao_bit() {
    let (rx, ry, rw, rh) = (3u32, 2u32, 17u32, 11u32);
    let mut x = 0x9E37_79B9u32;
    let mut rnd = move || {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        x
    };
    let especiais = [0.0f32, -0.0, 1.0, UM_MAIS, f32::NAN, 0.3];
    let acc0: Vec<[f32; 4]> = (0..rw * rh)
        .map(|_| {
            let r = rnd();
            let c = |k: u32| match (r >> k) & 7 {
                0..=4 => decode_byte((r >> (k + 3)) as u8),
                v => especiais[(v as usize + (r as usize >> 20)) % especiais.len()],
            };
            match r % 5 {
                0 => [0.0; 4],
                1 => [c(0), c(4), c(8), c(12)],
                _ => [c(0), c(4), c(8), 1.0],
            }
        })
        .collect();
    let fonte: Vec<[f32; 4]> = (0..(rx + rw) * (ry + rh))
        .map(|_| {
            let r = rnd();
            let a = [0.0f32, 0.0, -0.0, 1.0, 0.5][(r % 5) as usize];
            let cor = if r >> 28 == 0 {
                f32::NAN
            } else {
                decode_byte((r >> 8) as u8)
            };
            [
                cor,
                decode_byte((r >> 16) as u8),
                decode_byte((r >> 24) as u8),
                a,
            ]
        })
        .collect();
    let sample = |gx: u32, gy: u32| fonte[(gy * (rx + rw) + gx) as usize];
    let mut saltou = false;
    for m in 0..MAX_BLEND_MODES {
        let mode = BlendMode::from_u8(m);
        for opacity in [1.0f32, 0.37, 0.0] {
            let mut antigo = acc0.clone();
            blend_window_antigo(&mut antigo, rx, ry, rw, rh, mode, opacity, sample);
            let mut novo = acc0.clone();
            blend_window(&mut novo, rx, ry, rw, rh, mode, opacity, sample);
            let (a, n): (Vec<_>, Vec<_>) = (
                antigo.iter().map(|p| bits(*p)).collect(),
                novo.iter().map(|p| bits(*p)).collect(),
            );
            assert!(a == n, "{mode:?}, opacity {opacity}");
            saltou |= mode == BlendMode::Normal
                && (0..rw * rh).any(|i| {
                    let (lx, ly) = (i % rw, i / rw);
                    let mut s = sample(rx + lx, ry + ly);
                    s[3] *= opacity;
                    s[3] <= 0.0 && normal_transparente_e_identidade(acc0[i as usize], s)
                });
        }
    }
    assert!(saltou, "the window must take the skip somewhere");
}
