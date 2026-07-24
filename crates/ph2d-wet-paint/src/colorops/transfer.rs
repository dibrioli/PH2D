//! The sRGB transfer as a TABLE (child of [`super`]).
//!
//! The transfer is the only transcendental in the model, and it is the entire
//! cost of both EXPERIMENTAL knobs: pigment mixing (K–M) spends 9 `pow` per
//! colour mix and 15 per advected cell, glaze layering 9 per pixel per layer.
//! Measured on the flood scene, that is a 20-33x tax on two solver passes and
//! a 21x tax on the composite (`tests/measure_experimental.rs`).
//!
//! `libm::pow` is ~24 ns because it is a portable, correctly-rounded, general
//! `x^y`. We never need a general one: the exponents are the two sRGB
//! constants, and the domain is [0,1]. A node table plus linear interpolation
//! answers the same question in ~2.2 ns.
//!
//! **Determinism is preserved, and by a stronger argument than before.** The
//! nodes are computed ONCE by `libm::pow` (so every node is the reference
//! value, bit for bit, on every OS); between nodes the only operations are
//! `+ - * /`, comparison and a float->int truncation, all of which IEEE-754
//! specifies exactly and Rust never contracts into an FMA. So the table's
//! output is bit-identical across OSes, which is the property the port law
//! (`lib.rs`) asks of a transcendental.
//!
//! **Accuracy is measured, not asserted** (`tests/transfer_accuracy.rs`):
//! max |table - libm| is 1.5e-8 forward and 7.1e-8 inverse, against one byte
//! level = 3.92e-3 — five decimal orders of headroom. The SPEC's stated worry
//! is the SIM side, where "a wet cell can re-mix thousands of times", and it
//! is the right worry: `libm` is an exact FIXED POINT under re-mixing (a
//! still wash at 60.00000 is still 60.00000 after 5000 passes), so any
//! approximation there does not just err, it WALKS — until it reaches a fixed
//! point of its own interpolation error, a couple of table cells away.
//! Measured, that walk is 0.016 of a byte level.
//!
//! Both piecewise branches keep the reference's exact structure: the LINEAR
//! segment is computed, not tabulated (it is one multiply and it is exact),
//! and an out-of-gamut argument (`c > 1`, which the engine's clamped call
//! sites cannot produce but `km_weighted_mean_color`'s unclamped ones could)
//! falls back to `libm` so behaviour outside the table's domain is unchanged.

use std::sync::LazyLock;

/// Nodes per table (3 tables, 384 KB, built lazily — so a session that never
/// ticks an EXPERIMENTAL knob never allocates them).
///
/// Speed is flat in this parameter and the flatness is MEASURED on the real
/// solver, not on a micro-benchmark: 5 repeats of the flood tick at 48 / 192 /
/// 384 KB gave medians 21.03 / 21.20 / 20.94 ms, i.e. one noise band. (A
/// lookup micro-benchmark cannot answer this question at all — in isolation
/// the table owns the whole cache, so it reports "free" for any size. The
/// solver streams a large grid alongside, which is the only place the cost
/// could appear.) Flat speed means the size is chosen for ACCURACY alone, and
/// the accuracy it buys is the still-wash drift in `tests/transfer_accuracy.rs`:
/// 0.469 byte levels at 1024 nodes, 0.140 at 2048, 0.055 at 8192, 0.016 here.
const N: usize = 16384;

/// The inverse transfer's curvature is concentrated at the dark end
/// (`f'' ~ r^-1.583`), so a single uniform table wastes its nodes: at 4096
/// nodes a uniform table measured 1.6e-5 against 4.3e-7 for the same 4096
/// split at 1/32 — 38x better for the same lookup cost.
const SPLIT: f64 = 1.0 / 32.0;

/// Reference forward transfer (SPEC §14) — the table's own source of truth.
#[inline]
pub fn srgb_to_linear_exact(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        libm::pow((c + 0.055) / 1.055, 2.4)
    }
}

/// Reference inverse transfer (SPEC §14).
#[inline]
pub fn linear_to_srgb_exact(c: f64) -> f64 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * libm::pow(c, 1.0 / 2.4) - 0.055
    }
}

struct Tables {
    to_linear: Vec<f64>,
    to_srgb_fine: Vec<f64>,
    to_srgb_coarse: Vec<f64>,
}

static TABLES: LazyLock<Tables> = LazyLock::new(|| {
    let mut to_linear = Vec::with_capacity(N + 1);
    let mut to_srgb_fine = Vec::with_capacity(N + 1);
    let mut to_srgb_coarse = Vec::with_capacity(N + 1);
    for k in 0..=N {
        let t = k as f64 / N as f64;
        to_linear.push(srgb_to_linear_exact(t));
        to_srgb_fine.push(linear_to_srgb_exact(t * SPLIT));
        to_srgb_coarse.push(linear_to_srgb_exact(SPLIT + t * (1.0 - SPLIT)));
    }
    Tables {
        to_linear,
        to_srgb_fine,
        to_srgb_coarse,
    }
});

/// Linear interpolation on a node table over a normalised position `x` in
/// [0, N]. `x` is already known finite and in range by the callers' branches.
#[inline]
fn lerp_at(table: &[f64], x: f64) -> f64 {
    let i = x as usize;
    // The callers clamp, but keep the index provably in range so the bound
    // check folds away and an out-of-range float can never index past the end.
    let i = if i >= N { N - 1 } else { i };
    let f = x - i as f64;
    table[i] + (table[i + 1] - table[i]) * f
}

/// sRGB -> linear light, `c` in [0,1]. Table-backed; see the module docs for
/// the determinism and accuracy arguments.
#[inline]
pub fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        // The reference's LINEAR segment, computed exactly — this also carries
        // negative arguments through with the reference's own behaviour.
        c / 12.92
    } else if c >= 1.0 {
        srgb_to_linear_exact(c)
    } else {
        lerp_at(&TABLES.to_linear, c * N as f64)
    }
}

/// Linear light -> sRGB, `c` in [0,1].
#[inline]
pub fn linear_to_srgb(c: f64) -> f64 {
    if c <= 0.0031308 {
        c * 12.92
    } else if c >= 1.0 {
        linear_to_srgb_exact(c)
    } else if c < SPLIT {
        lerp_at(&TABLES.to_srgb_fine, c * (N as f64 / SPLIT))
    } else {
        lerp_at(&TABLES.to_srgb_coarse, (c - SPLIT) * (N as f64 / (1.0 - SPLIT)))
    }
}

// ---------------------------------------------------------------------------
// The K/S door. Engine colours are 0..255 floats and every K–M site needs
// them as K/S ratios, so the chain `c/255` -> transfer -> `(1-R)^2/(2R)` runs
// 6-12 times per cell. The `/255` folds into the table's index for free.
//
// ⚠️ THE RATIO ITSELF WAS TABULATED TOO, AND THE MEASUREMENT REJECTED IT — do
// not re-fuse it. `c -> K/S` is one 1-D function, so it looks like one lookup
// waiting to happen, and fusing it did measure 2.2 ms faster on the flood.
// But K/S has a QUADRATIC ZERO at white and `dR/dKS -> -inf` there, so the
// composition is ill-conditioned exactly where paint is lightest: a table's
// tiny absolute error near the zero comes back amplified as colour. Measured
// consequence (`tests/transfer_accuracy.rs`), and it is not subtle — a STILL
// WASH stops being still. `libm` is an exact fixed point under re-mixing
// (60.00000 stays 60.00000 over 5000 passes); the fused table walked to
// 59.73 at c=60 and 253.51 at c=254, i.e. half a byte of colour drift on
// paint nobody touched.
//
// Going through reflectance instead is well conditioned in the same place
// (`dKS/dR -> 0` as R -> 1), so the ratio is COMPUTED, at the price of one
// division. That division is the cheapest correctness in this module.
// ---------------------------------------------------------------------------

/// Reflectance floor: a pure black would send K/S to infinity. Mirrors
/// [`super::R_FLOOR`] — this module owns the tabulated composition, so the
/// constant is asserted equal in the accuracy test rather than imported into
/// a cycle.
const R_FLOOR: f64 = 1.0 / 255.0;

/// The exact chain the table stands in for: a 0..255 sRGB channel -> K/S.
#[inline]
pub fn ks_of_srgb255_exact(c: f64) -> f64 {
    let r = srgb_to_linear_exact(c / 255.0);
    let rr = if r < R_FLOOR { R_FLOOR } else { r };
    ((1.0 - rr) * (1.0 - rr)) / (2.0 * rr)
}

/// A 0..255 sRGB channel -> its Kubelka–Munk K/S ratio: ONE table lookup for
/// the transfer, then the ratio computed. See the block comment above for why
/// the ratio is not tabulated.
///
/// Out of range takes the exact chain — below 0 and above 255 are outside the
/// table's domain, and the unclamped call sites (`km_weighted_mean_color`)
/// can in principle hand over either.
#[inline]
pub fn ks_of_srgb255(c: f64) -> f64 {
    if !(c > 0.0 && c < 255.0) {
        return ks_of_srgb255_exact(c);
    }
    let r = srgb255_to_linear(c);
    let rr = if r < R_FLOOR { R_FLOOR } else { r };
    ((1.0 - rr) * (1.0 - rr)) / (2.0 * rr)
}

/// Linear reflectance -> a 0..255 sRGB channel. The `* 255` folds into the
/// same lookup the inverse transfer already does.
#[inline]
pub fn srgb255_of_linear(r: f64) -> f64 {
    linear_to_srgb(r) * 255.0
}

/// A 0..255 sRGB channel -> linear light, CLAMPED to the gamut first — the
/// renderer's `clamp_byte(v) / 255.0` in one step, without the division.
///
/// Note the deliberate difference from [`ks_of_srgb255`], which does NOT
/// clamp: the glaze call sites clamp today (a tooth offset can push a channel
/// past 255) and the K–M call sites do not, so each door keeps its own call
/// sites' behaviour instead of quietly imposing one policy on both.
#[inline]
pub fn srgb255_to_linear(c: f64) -> f64 {
    let c = if c < 0.0 {
        0.0
    } else if c > 255.0 {
        255.0
    } else {
        c
    };
    if c <= 0.04045 * 255.0 {
        // The reference's LINEAR segment, with the reference's own two-step
        // arithmetic — this branch stays bit-exact rather than folding the
        // rescale into a single constant (which would round differently).
        (c / 255.0) / 12.92
    } else {
        // Above the kink only the table's INDEX is rescaled. Folding `/255`
        // into the index moves the interpolation position by an ulp, which is
        // orders below the 7.4e-8 the interpolation itself costs.
        lerp_at(&TABLES.to_linear, c * (N as f64 / 255.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The piecewise branches are the reference's, so the linear segments and
    /// the endpoints must agree to the BIT, not to a tolerance.
    #[test]
    fn the_linear_segments_and_endpoints_are_bit_exact() {
        for c in [0.0, 1e-9, 0.001, 0.02, 0.04045, 1.0, 1.5] {
            assert_eq!(
                srgb_to_linear(c).to_bits(),
                srgb_to_linear_exact(c).to_bits(),
                "forward diverged at {c}"
            );
        }
        for c in [0.0, 1e-9, 0.001, 0.0031308, 1.0, 1.5] {
            assert_eq!(
                linear_to_srgb(c).to_bits(),
                linear_to_srgb_exact(c).to_bits(),
                "inverse diverged at {c}"
            );
        }
    }

    /// A negative argument must behave exactly as the reference does (the
    /// unclamped K–M call sites can hand one over).
    #[test]
    fn negatives_take_the_reference_branch() {
        for c in [-0.5, -1e-6] {
            assert_eq!(srgb_to_linear(c).to_bits(), srgb_to_linear_exact(c).to_bits());
            assert_eq!(linear_to_srgb(c).to_bits(), linear_to_srgb_exact(c).to_bits());
        }
    }

    /// Monotonic: a table that dips would show as banding in a wash.
    #[test]
    fn both_directions_are_monotonic() {
        let mut prev = -1.0;
        for k in 0..20_000 {
            let v = srgb_to_linear(k as f64 / 20_000.0);
            assert!(v >= prev, "forward dipped at {k}");
            prev = v;
        }
        let mut prev = -1.0;
        for k in 0..20_000 {
            let v = linear_to_srgb(k as f64 / 20_000.0);
            assert!(v >= prev, "inverse dipped at {k}");
            prev = v;
        }
    }
}
