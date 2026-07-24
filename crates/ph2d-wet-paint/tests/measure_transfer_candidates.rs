//! Candidate sRGB transfers for the EXPERIMENTAL paths: speed AND accuracy
//! against `libm::pow`, which is the reference and the current cost.
//!
//!   cargo test -p ph2d-wet-paint --release --test measure_transfer_candidates \
//!     -- --ignored --nocapture

use std::time::Instant;

// ---------------------------------------------------------------- reference

fn ref_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        libm::pow((c + 0.055) / 1.055, 2.4)
    }
}
fn ref_to_srgb(c: f64) -> f64 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * libm::pow(c, 1.0 / 2.4) - 0.055
    }
}

// ------------------------------------------------- candidate: uniform + lerp

struct Lut {
    to_lin: Vec<f64>,
    to_srgb: Vec<f64>,
    n: usize,
}

impl Lut {
    fn new(n: usize) -> Self {
        let mut to_lin = Vec::with_capacity(n + 1);
        let mut to_srgb = Vec::with_capacity(n + 1);
        for k in 0..=n {
            let t = k as f64 / n as f64;
            to_lin.push(ref_to_linear(t));
            to_srgb.push(ref_to_srgb(t));
        }
        Lut { to_lin, to_srgb, n }
    }
    #[inline]
    fn lin(&self, c: f64) -> f64 {
        let x = if c < 0.0 {
            0.0
        } else if c > 1.0 {
            1.0
        } else {
            c
        } * self.n as f64;
        let i = x as usize;
        let i = if i >= self.n { self.n - 1 } else { i };
        let f = x - i as f64;
        self.to_lin[i] + (self.to_lin[i + 1] - self.to_lin[i]) * f
    }
    #[inline]
    fn srgb(&self, c: f64) -> f64 {
        let x = if c < 0.0 {
            0.0
        } else if c > 1.0 {
            1.0
        } else {
            c
        } * self.n as f64;
        let i = x as usize;
        let i = if i >= self.n { self.n - 1 } else { i };
        let f = x - i as f64;
        self.to_srgb[i] + (self.to_srgb[i + 1] - self.to_srgb[i]) * f
    }
}

// ------------------------------- candidate: SEGMENTED inverse (fine near 0)
// The inverse's curvature is concentrated at the dark end (f'' ~ r^-1.583),
// so split the domain: a fine table on [0, SPLIT] and a coarse one above.

struct SegLut {
    fine: Vec<f64>,
    coarse: Vec<f64>,
    n: usize,
}
const SPLIT: f64 = 1.0 / 32.0;

impl SegLut {
    fn new(n: usize) -> Self {
        let mut fine = Vec::with_capacity(n + 1);
        let mut coarse = Vec::with_capacity(n + 1);
        for k in 0..=n {
            let t = k as f64 / n as f64;
            fine.push(ref_to_srgb(t * SPLIT));
            coarse.push(ref_to_srgb(SPLIT + t * (1.0 - SPLIT)));
        }
        SegLut { fine, coarse, n }
    }
    #[inline]
    fn srgb(&self, c: f64) -> f64 {
        let c = if c < 0.0 {
            0.0
        } else if c > 1.0 {
            1.0
        } else {
            c
        };
        let (tab, x) = if c < SPLIT {
            (&self.fine, c / SPLIT * self.n as f64)
        } else {
            (&self.coarse, (c - SPLIT) / (1.0 - SPLIT) * self.n as f64)
        };
        let i = x as usize;
        let i = if i >= self.n { self.n - 1 } else { i };
        let f = x - i as f64;
        tab[i] + (tab[i + 1] - tab[i]) * f
    }
}

fn max_err(f: &dyn Fn(f64) -> f64, r: &dyn Fn(f64) -> f64, samples: usize) -> (f64, f64) {
    let mut worst = 0.0f64;
    let mut at = 0.0f64;
    for k in 0..=samples {
        let x = k as f64 / samples as f64;
        let e = (f(x) - r(x)).abs();
        if e > worst {
            worst = e;
            at = x;
        }
    }
    (worst, at)
}

/// A RANDOM probe over the whole domain — a cycling 1024-value probe keeps the
/// working set in L1 no matter how big the table is, which flatters exactly
/// the candidate that would thrash in the product (colours in a render vary).
fn random_probe(n: usize) -> Vec<f64> {
    let mut s = 0x243f_6a88_85a3_08d3u64;
    (0..n)
        .map(|_| {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            (s >> 11) as f64 / (1u64 << 53) as f64
        })
        .collect()
}

#[test]
#[ignore = "wall-clock: run with --release -- --ignored --nocapture"]
fn measure_transfer_candidates() {
    const N: usize = 3_000_000;
    let probe = random_probe(1 << 16);
    let mask = probe.len() - 1;

    macro_rules! bench {
        ($name:expr, $f:expr) => {{
            let mut acc = 0.0f64;
            let t = Instant::now();
            for k in 0..N {
                acc += $f(probe[k & mask]);
            }
            let ms = t.elapsed().as_secs_f64() * 1000.0;
            println!(
                "    {:<34} {:6.1} ns/call   [sink {:.4}]",
                $name,
                ms * 1e6 / N as f64,
                acc / N as f64
            );
        }};
    }

    println!("\n  SPEED (random probe over the full domain; tables in bytes)");
    bench!("libm  srgb_to_linear", ref_to_linear);
    bench!("libm  linear_to_srgb", ref_to_srgb);
    for n in [512usize, 1024, 4096, 16384] {
        let lut = Lut::new(n);
        let kb = (n + 1) * 8 / 1024;
        bench!(format!("lut{n} ({kb} KB) srgb_to_linear"), |x| lut.lin(x));
        bench!(format!("lut{n} ({kb} KB) linear_to_srgb"), |x| lut.srgb(x));
    }
    for n in [512usize, 1024, 2048, 4096] {
        let seg = SegLut::new(n);
        let kb = (n + 1) * 16 / 1024;
        bench!(format!("seg{n} ({kb} KB) linear_to_srgb"), |x| seg.srgb(x));
    }

    println!("\n  ACCURACY (max |candidate - libm| over 4M samples in [0,1])");
    for n in [512usize, 1024, 4096, 16384] {
        let lut = Lut::new(n);
        let (e1, a1) = max_err(&|x| lut.lin(x), &ref_to_linear, 4_000_000);
        let (e2, a2) = max_err(&|x| lut.srgb(x), &ref_to_srgb, 4_000_000);
        println!("    lut{n:<6} to_linear {e1:.3e} @ {a1:.5}   to_srgb {e2:.3e} @ {a2:.5}");
    }
    for n in [512usize, 1024, 2048, 4096] {
        let seg = SegLut::new(n);
        let (e, a) = max_err(&|x| seg.srgb(x), &ref_to_srgb, 4_000_000);
        println!("    seg{n:<6}  to_srgb   {e:.3e} @ {a:.5}");
    }

    println!("\n  SCALE: an error of 1/255 in sRGB = 3.92e-3 (one byte level).");
}

/// The drift the SPEC warns about: a wet cell "re-mixes thousands of times".
/// Iterate the K–M mix with the reference transfer and with a candidate, and
/// report how far apart the two colours end up — the number that decides
/// whether a table is safe on the SIM side (the render side only has to beat
/// one byte level).
#[test]
#[ignore = "wall-clock: run with --release -- --ignored --nocapture"]
fn measure_iterated_mix_drift() {
    const R_FLOOR: f64 = 1.0 / 255.0;
    fn ks_of(r: f64) -> f64 {
        let rr = if r < R_FLOOR { R_FLOOR } else { r };
        ((1.0 - rr) * (1.0 - rr)) / (2.0 * rr)
    }
    fn r_of(ks: f64) -> f64 {
        1.0 + ks - (ks * ks + 2.0 * ks).sqrt()
    }
    // One K–M channel mix, parameterised by the transfer pair under test.
    fn mix(d: f64, s: f64, w: f64, to_lin: &dyn Fn(f64) -> f64, to_s: &dyn Fn(f64) -> f64) -> f64 {
        let ks = (1.0 - w) * ks_of(to_lin(d / 255.0)) + w * ks_of(to_lin(s / 255.0));
        to_s(r_of(ks)) * 255.0
    }
    println!("\n  ITERATED MIX DRIFT (the SPEC's 'thousands of re-mixes')");
    for n in [512usize, 1024, 4096] {
        let lut = Lut::new(n);
        let seg = SegLut::new(n);
        // Alternate two pigments at a light weight, 5000 times — a cell in a
        // wash being re-mixed by every passing front.
        let (mut a_ref, mut a_lut) = (200.0f64, 200.0f64);
        for k in 0..5000 {
            let src = if k % 2 == 0 { 12.0 } else { 190.0 };
            a_ref = mix(a_ref, src, 0.03, &ref_to_linear, &ref_to_srgb);
            a_lut = mix(a_lut, src, 0.03, &|x| lut.lin(x), &|x| seg.srgb(x));
        }
        println!(
            "    lut{n}+seg{n}  after 5000 mixes: ref {a_ref:.6}  cand {a_lut:.6}  \
             drift {:.3e} of 255 ({:.4} byte levels)",
            (a_ref - a_lut).abs(),
            (a_ref - a_lut).abs()
        );
    }
}
