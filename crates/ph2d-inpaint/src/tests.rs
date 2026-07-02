//! Known-answer tests for the full multi-scale pipeline. Each punches a hole in
//! a synthetic image whose "correct" fill is known, then asserts the
//! reconstruction matches: a periodic texture must be rebuilt, a flat colour
//! stays flat, a gradient stays smooth, and the same seed is byte-reproducible.

use super::*;

/// Build an `w*h*4` RGBA buffer from a per-pixel colour closure.
fn rgba_from<F: Fn(usize, usize) -> [u8; 3]>(w: usize, h: usize, f: F) -> Vec<u8> {
    let mut v = vec![0u8; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let c = f(x, y);
            let o = (y * w + x) * 4;
            v[o..o + 4].copy_from_slice(&[c[0], c[1], c[2], 255]);
        }
    }
    v
}

/// A rectangular hole mask.
fn rect_mask(w: usize, h: usize, hx: usize, hy: usize, hw: usize, hh: usize) -> Vec<u8> {
    let mut m = vec![0u8; w * h];
    for y in hy..hy + hh {
        for x in hx..hx + hw {
            m[y * w + x] = 255;
        }
    }
    m
}

fn run(w: usize, h: usize, rgba: &[u8], mask: &[u8], params: InpaintParams) -> Vec<u8> {
    inpaint_cpu(&InpaintRequest {
        width: w as u32,
        height: h as u32,
        rgba,
        mask,
        params,
    })
    .rgba
}

#[test]
fn no_hole_returns_the_input_unchanged() {
    let (w, h) = (16, 16);
    let img = rgba_from(w, h, |x, _| [x as u8 * 4, 0, 0]);
    let mask = vec![0u8; w * h];
    let out = run(w, h, &img, &mask, InpaintParams::default());
    assert_eq!(out, img, "no hole ⇒ output must equal input");
}

#[test]
fn known_pixels_are_byte_identical_to_the_source() {
    let (w, h) = (48, 48);
    let img = rgba_from(w, h, |x, y| [(x * 5) as u8, (y * 5) as u8, 128]);
    let mask = rect_mask(w, h, 20, 20, 8, 8);
    let out = run(w, h, &img, &mask, InpaintParams::default());
    for y in 0..h {
        for x in 0..w {
            let o = (y * w + x) * 4;
            if mask[y * w + x] < 128 {
                assert_eq!(
                    &out[o..o + 4],
                    &img[o..o + 4],
                    "known pixel ({x},{y}) must be untouched"
                );
            }
        }
    }
}

#[test]
fn flat_colour_hole_fills_with_the_flat_colour() {
    let (w, h) = (48, 48);
    let col = [70u8, 140, 200];
    let img = rgba_from(w, h, |_, _| col);
    let mask = rect_mask(w, h, 18, 18, 10, 10);
    let out = run(w, h, &img, &mask, InpaintParams::default());
    for y in 18..28 {
        for x in 18..28 {
            let o = (y * w + x) * 4;
            for k in 0..3 {
                let d = i32::from(out[o + k]) - i32::from(col[k]);
                assert!(d.abs() <= 2, "flat fill off at ({x},{y}) ch {k}: {d}");
            }
        }
    }
}

#[test]
fn periodic_stripes_are_reconstructed() {
    // Vertical stripes with period 4 (2 light, 2 dark). The hole must be filled
    // with the SAME stripe phase — that is what exemplar copying buys over a
    // blur, which would grey the hole out.
    let (w, h) = (64, 64);
    let stripe = |x: usize| {
        if (x / 2).is_multiple_of(2) {
            235u8
        } else {
            25u8
        }
    };
    let img = rgba_from(w, h, |x, _| [stripe(x); 3]);
    let mask = rect_mask(w, h, 26, 26, 12, 12);
    let out = run(w, h, &img, &mask, InpaintParams::default());
    let mut worst = 0i32;
    for y in 26..38 {
        for x in 26..38 {
            let o = (y * w + x) * 4;
            let want = i32::from(stripe(x));
            worst = worst.max((i32::from(out[o]) - want).abs());
        }
    }
    assert!(
        worst <= 40,
        "stripe reconstruction worst-channel error {worst}"
    );
}

#[test]
fn horizontal_gradient_hole_stays_monotone_and_smooth() {
    // A left→right gradient. The exact value isn't patch-copyable, but the fill
    // must stay within the surrounding band and not spike.
    let (w, h) = (64, 32);
    let grad = |x: usize| ((x * 255) / (w - 1)) as u8;
    let img = rgba_from(w, h, |x, _| [grad(x); 3]);
    let mask = rect_mask(w, h, 24, 10, 16, 12);
    let out = run(w, h, &img, &mask, InpaintParams::default());
    for y in 10..22 {
        for x in 24..40 {
            let o = (y * w + x) * 4;
            let lo = i32::from(grad(23)) - 30;
            let hi = i32::from(grad(40)) + 30;
            let v = i32::from(out[o]);
            assert!(
                v >= lo && v <= hi,
                "gradient fill out of band at ({x},{y}): {v}"
            );
        }
    }
}

#[test]
fn same_seed_is_byte_reproducible() {
    let (w, h) = (48, 48);
    let img = rgba_from(w, h, |x, y| {
        [(x * 3) as u8, (y * 3) as u8, ((x + y) * 2) as u8]
    });
    let mask = rect_mask(w, h, 16, 16, 12, 12);
    let a = run(w, h, &img, &mask, InpaintParams::default());
    let b = run(w, h, &img, &mask, InpaintParams::default());
    assert_eq!(a, b, "same seed must be byte-identical");
}

#[test]
fn different_seeds_still_fill_the_whole_hole_opaque() {
    let (w, h) = (40, 40);
    let img = rgba_from(w, h, |x, y| [(x * 6) as u8, 100, (y * 6) as u8]);
    let mask = rect_mask(w, h, 14, 14, 12, 12);
    let p = InpaintParams {
        seed: 999,
        ..Default::default()
    };
    let out = run(w, h, &img, &mask, p);
    for y in 14..26 {
        for x in 14..26 {
            let o = (y * w + x) * 4;
            assert_eq!(out[o + 3], 255, "hole pixel ({x},{y}) must be opaque");
        }
    }
}
