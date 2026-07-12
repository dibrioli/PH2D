//! **Linear prediction (LPC)** — the shared core under De-Click, De-Clip and Formant Shift.
//!
//! Fit an all-pole model to a block of audio: `x[n] ≈ Σ a[k]·x[n−k]`. What the model
//! captures is the **spectral envelope** (the resonances — a voice's formants, an
//! instrument's body); what it fails to capture is the **residual** `e[n]` (the
//! excitation — the glottal pulses that carry the pitch, and anything the signal had
//! no business doing, like a click). Splitting a signal into those two halves is the
//! source-filter model, and it is what both effects here are built on:
//!
//! - **De-Click** hunts outliers in the residual, then re-derives the samples under
//!   them from the model (the click is what the model could not predict).
//! - **Formant Shift** warps the filter and leaves the residual alone (the formants
//!   move, the pitch does not).
//!
//! Method: autocorrelation + **Levinson–Durbin** recursion (Rabiner & Schafer,
//! *Digital Processing of Speech Signals*, §8.3). The autocorrelation method is the
//! one that guarantees a stable (minimum-phase) filter — the covariance method does
//! not, and an unstable `1/A(z)` would blow up the resynthesis.
//!
//! `f64` throughout: at order 32 the normal equations are ill-conditioned enough that
//! `f32` loses the small reflection coefficients, and a lost coefficient is a formant
//! that moves when it should not. Control thread only.

/// White-noise correction applied to `r[0]` before the recursion — the standard
/// speech-coder ridge. It bounds the condition number of an otherwise singular
/// Toeplitz system (a perfectly periodic block is rank-deficient), at the cost of a
/// spectral floor far below anything audible.
const RIDGE: f64 = 1e-9;

/// Autocorrelation `r[lag]` for `lag` in `0..=order`.
///
/// The plain (unwindowed-lag) estimator: `r[lag] = Σ_n x[n]·x[n+lag]`, summed over
/// the samples that exist. This is what makes the resulting Toeplitz matrix positive
/// semi-definite — the property Levinson leans on.
use std::ops::Range;

pub(super) fn autocorrelation(x: &[f64], order: usize) -> Vec<f64> {
    (0..=order)
        .map(|lag| {
            if lag >= x.len() {
                return 0.0;
            }
            x.iter().zip(&x[lag..]).map(|(a, b)| a * b).sum()
        })
        .collect()
}

/// Solve the Yule–Walker system for the prediction coefficients `a[0..p]`, where
/// `a[k]` weighs lag `k+1`: `x[n] ≈ Σ_{k=0..p} a[k]·x[n−k−1]`.
///
/// Returns the coefficients and the **residual energy** left over. `None` when the
/// block carries no energy at all (silence has no envelope to model) or the recursion
/// runs into a non-positive error — the ridge makes that vanishingly unlikely, but a
/// denormal block should bypass, not produce garbage coefficients.
pub(super) fn levinson(r: &[f64]) -> Option<(Vec<f64>, f64)> {
    let p = r.len().checked_sub(1)?;
    let mut err = r[0] * (1.0 + RIDGE);
    if p == 0 || !err.is_finite() || err <= 0.0 {
        return None;
    }
    let mut a = vec![0.0f64; p];
    for i in 1..=p {
        // Reflection coefficient: how much of lag `i` the model still cannot explain.
        let acc: f64 = (1..i).map(|j| a[j - 1] * r[i - j]).sum();
        let k = (r[i] - acc) / err;
        // Update the coefficients in place, from the outside in — `a[j]` and
        // `a[i−j−1]` are the pair the recursion mixes, so walk half and swap.
        let prev = a.clone();
        a[i - 1] = k;
        for j in 1..i {
            a[j - 1] = prev[j - 1] - k * prev[i - j - 1];
        }
        err *= 1.0 - k * k;
        if !err.is_finite() || err <= 0.0 {
            return None;
        }
    }
    Some((a, err))
}

/// The prediction residual `e[n] = x[n] − Σ a[k]·x[n−k−1]` over the whole of `x`,
/// treating everything before `x[0]` as silence.
///
/// This is the excitation: flat where the model fits (a vowel, a sustained note),
/// spiky where it does not (a glottal pulse — or a click).
pub(super) fn residual(x: &[f64], a: &[f64]) -> Vec<f64> {
    x.iter()
        .enumerate()
        .map(|(n, &xn)| {
            let pred: f64 = a
                .iter()
                .enumerate()
                .filter_map(|(k, &ak)| n.checked_sub(k + 1).map(|i| ak * x[i]))
                .sum();
            xn - pred
        })
        .collect()
}

/// Truncated impulse response of the all-pole synthesis filter `1/A(z)` — the
/// spectral envelope, as a signal.
///
/// `h[0] = 1`, `h[n] = Σ a[k]·h[n−k−1]`: feed the filter a unit impulse and read
/// what comes out. Both effects need the envelope in the time domain (one warps it,
/// the other never leaves it), which is what keeps this whole module FFT-free.
pub(super) fn impulse_response(a: &[f64], len: usize) -> Vec<f64> {
    let mut h = vec![0.0f64; len];
    if len == 0 {
        return h;
    }
    h[0] = 1.0;
    for n in 1..len {
        h[n] = a
            .iter()
            .enumerate()
            .filter_map(|(k, &ak)| n.checked_sub(k + 1).map(|i| ak * h[i]))
            .sum();
    }
    h
}

/// A pivot smaller than this means the linear system is singular; bail rather than divide
/// by dust.
const PIVOT_EPS: f64 = 1e-12;

/// Replace `gap` with the samples that minimise the AR residual energy across it
/// (Janssen et al. 1986). The known audio on both sides pins the solution.
///
/// Writing the error filter as `b = [1, −a₀, −a₁, …]`, the residual energy is
/// quadratic in the unknowns, so setting its derivative to zero gives one linear
/// equation per missing sample:
///
/// ```text
///   Σ_{m' ∈ gap} R(m − m')·x[m']  =  − Σ_{j ∉ gap} R(m − j)·x[j]      ∀ m ∈ gap
/// ```
///
/// where `R` is the autocorrelation of `b`.
///
/// **That matrix is Toeplitz.** The gap is contiguous, so `R(mᵢ − mⱼ)` depends only on
/// `i − j`: every diagonal is constant. Gaussian elimination does not know that and pays
/// **O(m³)** for it — 2.2 M flops for a single 192-sample plateau, which is what made
/// dragging De-Clip's Threshold slider freeze the editor at 2 s per frame (audit
/// 2026-07-12). The **Levinson recursion** exploits the structure and solves the same system
/// in **O(m²)** — ~200× fewer operations at m = 192, to the same answer.
///
/// It is the same answer, and `levinson_matches_gaussian_elimination` says so by keeping the
/// old dense solver around as a test-only oracle and comparing against it.
pub(super) fn lsar_interpolate(x: &mut [f64], a: &[f64], gap: Range<usize>) {
    let p = a.len();
    let m = gap.len();
    if m == 0 {
        return;
    }
    // The error filter, then its own autocorrelation R(0..=p).
    let mut b = vec![0.0f64; p + 1];
    b[0] = 1.0;
    for (k, &ak) in a.iter().enumerate() {
        b[k + 1] = -ak;
    }
    let rr: Vec<f64> = (0..=p)
        .map(|d| (0..=(p - d)).map(|i| b[i] * b[i + d]).sum())
        .collect();
    let r = |d: isize| -> f64 {
        let d = d.unsigned_abs();
        if d <= p { rr[d] } else { 0.0 }
    };

    let mut mat = vec![0.0f64; m * m];
    let mut rhs = vec![0.0f64; m];
    for (i, mi) in gap.clone().enumerate() {
        for (j, mj) in gap.clone().enumerate() {
            mat[i * m + j] = r(mi as isize - mj as isize);
        }
        // Everything within `p` of the gap that is NOT missing pins this equation.
        let lo = mi.saturating_sub(p);
        let hi = (mi + p + 1).min(x.len());
        let known: f64 = (lo..hi)
            .filter(|j| !gap.contains(j))
            .map(|j| r(mi as isize - j as isize) * x[j])
            .sum();
        rhs[i] = -known;
    }
    if let Some(sol) = solve_toeplitz(&rr, &rhs) {
        for (i, mi) in gap.enumerate() {
            x[mi] = sol[i];
        }
    }
}

/// Solve a symmetric **Toeplitz** system `T·x = rhs`, where `T[i][j] = rr[|i−j|]`, by the
/// Levinson recursion — O(m²) instead of the O(m³) a general solver pays for not knowing the
/// structure.
///
/// The recursion grows the solution one row at a time, carrying the *forward vector* `f`
/// (the solution of `T·f = e₁`). For a symmetric Toeplitz matrix the backward vector is
/// simply `f` reversed, which is what makes the step cheap.
///
/// `None` when the recursion breaks down (a reflection coefficient reaching ±1, i.e. a
/// singular leading minor) or when anything stops being finite — the caller then leaves the
/// audio alone rather than writing dust into it.
fn solve_toeplitz(rr: &[f64], rhs: &[f64]) -> Option<Vec<f64>> {
    let m = rhs.len();
    if m == 0 {
        return Some(Vec::new());
    }
    // `rr` only spans the AR order; beyond it the autocorrelation is zero.
    let r = |d: usize| -> f64 { rr.get(d).copied().unwrap_or(0.0) };
    let r0 = r(0);
    if r0.abs() < PIVOT_EPS {
        return None;
    }

    let mut f = vec![1.0 / r0];
    let mut x = vec![rhs[0] / r0];
    for n in 1..m {
        // How badly the current forward vector predicts the new row.
        let eps_f: f64 = (0..n).map(|i| r(n - i) * f[i]).sum();
        let denom = 1.0 - eps_f * eps_f;
        if denom.abs() < PIVOT_EPS {
            return None;
        }
        // Extend f, using its own reverse as the backward vector (the symmetry).
        let mut fx = vec![0.0f64; n + 1];
        for i in 0..n {
            fx[i] += f[i] / denom;
            fx[i + 1] -= eps_f * f[n - 1 - i] / denom;
        }
        // Extend the solution along the new backward vector.
        let eps_x: f64 = (0..n).map(|i| r(n - i) * x[i]).sum();
        let mu = rhs[n] - eps_x;
        let mut xx = vec![0.0f64; n + 1];
        xx[..n].copy_from_slice(&x);
        for i in 0..=n {
            xx[i] += mu * fx[n - i];
        }
        f = fx;
        x = xx;
    }
    x.iter().all(|v| v.is_finite()).then_some(x)
}

/// Gaussian elimination with partial pivoting — the **reference oracle**.
///
/// This is what `lsar_interpolate` used to run in production, at O(m³). It is kept because it
/// is the independent answer: [`solve_toeplitz`] is fast because it assumes a structure, and
/// the only honest way to trust that assumption is to compare against a solver that assumes
/// nothing.
#[cfg(test)]
fn solve(mat: &mut [f64], rhs: &mut [f64], n: usize) -> Option<Vec<f64>> {
    for col in 0..n {
        let mut piv = col;
        for row in col + 1..n {
            if mat[row * n + col].abs() > mat[piv * n + col].abs() {
                piv = row;
            }
        }
        if mat[piv * n + col].abs() < PIVOT_EPS {
            return None;
        }
        if piv != col {
            for c in 0..n {
                mat.swap(col * n + c, piv * n + c);
            }
            rhs.swap(col, piv);
        }
        let diag = mat[col * n + col];
        for row in col + 1..n {
            let f = mat[row * n + col] / diag;
            if f == 0.0 {
                continue;
            }
            for c in col..n {
                mat[row * n + c] -= f * mat[col * n + c];
            }
            rhs[row] -= f * rhs[col];
        }
    }
    let mut sol = vec![0.0f64; n];
    for i in (0..n).rev() {
        let acc: f64 = rhs[i] - (i + 1..n).map(|c| mat[i * n + c] * sol[c]).sum::<f64>();
        sol[i] = acc / mat[i * n + i];
    }
    sol.iter().all(|v| v.is_finite()).then_some(sol)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The identity the whole module rests on: inverse-filtering a signal with its
    /// own AR model and then re-synthesising through `1/A(z)` returns the signal.
    /// If this drifts, De-Click repairs toward the wrong values and Formant Shift
    /// colours the audio even at a shift of zero.
    #[test]
    fn residual_through_the_synthesis_filter_reconstructs_the_signal() {
        // An AR(2) process, generated by the very recursion the model must recover.
        let a_true = [1.6f64, -0.8];
        let mut x = vec![0.0f64; 512];
        for n in 0..x.len() {
            let drive = if n % 64 == 0 { 1.0 } else { 0.0 }; // a pulse train
            let p1 = if n >= 1 { a_true[0] * x[n - 1] } else { 0.0 };
            let p2 = if n >= 2 { a_true[1] * x[n - 2] } else { 0.0 };
            x[n] = drive + p1 + p2;
        }
        let (a, _) = levinson(&autocorrelation(&x, 8)).expect("a live block models");
        let e = residual(&x, &a);
        // Re-synthesise: y[n] = e[n] + Σ a[k]·y[n−k−1].
        let mut y = vec![0.0f64; x.len()];
        for n in 0..x.len() {
            let pred: f64 = a
                .iter()
                .enumerate()
                .filter_map(|(k, &ak)| n.checked_sub(k + 1).map(|i| ak * y[i]))
                .sum();
            y[n] = e[n] + pred;
        }
        for (n, (&xi, &yi)) in x.iter().zip(&y).enumerate() {
            assert!((xi - yi).abs() < 1e-6, "frame {n}: {xi} vs {yi}");
        }
    }

    /// Levinson must actually recover the process that generated the block — this is
    /// what makes the residual an excitation rather than a scrambled copy.
    #[test]
    fn levinson_recovers_the_generating_coefficients() {
        let a_true = [1.6f64, -0.8];
        let mut x = vec![0.0f64; 4_096];
        for n in 0..x.len() {
            let drive = if n % 128 == 0 { 1.0 } else { 0.0 };
            let p1 = if n >= 1 { a_true[0] * x[n - 1] } else { 0.0 };
            let p2 = if n >= 2 { a_true[1] * x[n - 2] } else { 0.0 };
            x[n] = drive + p1 + p2;
        }
        let (a, err) = levinson(&autocorrelation(&x, 2)).expect("models");
        assert!((a[0] - a_true[0]).abs() < 0.05, "a1 was {}", a[0]);
        assert!((a[1] - a_true[1]).abs() < 0.05, "a2 was {}", a[1]);
        assert!(err > 0.0, "the residual energy must stay positive");
    }

    /// Silence has no envelope. The caller must get `None`, not a filter built out of
    /// zeros (which would divide by nothing downstream).
    #[test]
    fn a_silent_block_has_no_model() {
        assert!(levinson(&autocorrelation(&[0.0; 256], 16)).is_none());
    }

    /// The synthesis filter of a stable model decays. An impulse response that grows
    /// would mean Levinson handed back an unstable filter — the failure the
    /// autocorrelation method exists to prevent.
    #[test]
    fn the_synthesis_impulse_response_decays() {
        let a_true = [1.6f64, -0.8];
        let mut x = vec![0.0f64; 1_024];
        for n in 0..x.len() {
            let drive = if n % 64 == 0 { 1.0 } else { 0.0 };
            let p1 = if n >= 1 { a_true[0] * x[n - 1] } else { 0.0 };
            let p2 = if n >= 2 { a_true[1] * x[n - 2] } else { 0.0 };
            x[n] = drive + p1 + p2;
        }
        let (a, _) = levinson(&autocorrelation(&x, 12)).expect("models");
        let h = impulse_response(&a, 512);
        assert_eq!(h[0], 1.0, "the response opens on the impulse itself");
        let head: f64 = h[..64].iter().map(|v| v * v).sum();
        let tail: f64 = h[448..].iter().map(|v| v * v).sum();
        assert!(
            tail < head * 0.01,
            "head {head}, tail {tail}: it must decay"
        );
    }
}

#[cfg(test)]
mod solver_tests {
    use super::*;

    /// **The fast solver gives the SAME answer as the one that assumes nothing.**
    ///
    /// [`solve_toeplitz`] is O(m²) instead of O(m³) *because* it assumes the matrix is
    /// symmetric Toeplitz. That assumption is true — the gap is contiguous, so `R(mᵢ − mⱼ)`
    /// depends only on `i − j` — but "true" is a claim, and a claim about numerics is worth
    /// exactly what it is measured against. So: build the same system both ways and compare.
    ///
    /// Red if the recursion's indices are off by one, if the backward vector is not the
    /// reverse of the forward one, or if the symmetry assumption is ever violated upstream.
    #[test]
    fn levinson_matches_gaussian_elimination() {
        // A positive-definite Toeplitz matrix: the autocorrelation of a short filter — the
        // same shape `lsar_interpolate` actually builds.
        let filt = [1.0f64, -0.7, 0.35, -0.12, 0.04];
        let p = filt.len() - 1;
        let rr: Vec<f64> = (0..=p)
            .map(|d| (0..=(p - d)).map(|i| filt[i] * filt[i + d]).sum())
            .collect();

        for m in [1usize, 2, 3, 7, 16, 64, 192] {
            let rhs: Vec<f64> = (0..m)
                .map(|i| ((i as f64) * 0.37).sin() * 0.9 + 0.1)
                .collect();

            let fast = solve_toeplitz(&rr, &rhs).expect("levinson solved");

            let mut mat = vec![0.0f64; m * m];
            for i in 0..m {
                for j in 0..m {
                    mat[i * m + j] = rr.get(i.abs_diff(j)).copied().unwrap_or(0.0);
                }
            }
            let mut b = rhs.clone();
            let dense = solve(&mut mat, &mut b, m).expect("gauss solved");

            let worst = fast
                .iter()
                .zip(&dense)
                .map(|(a, b)| (a - b).abs())
                .fold(0.0f64, f64::max);
            assert!(
                worst < 1e-9,
                "m = {m}: the Levinson recursion disagrees with Gaussian elimination by {worst:e}"
            );
        }
    }

    /// …and it actually solves the system: `T·x` has to come back as `rhs`. (Agreeing with
    /// another solver would be cold comfort if both were wrong the same way — this checks the
    /// answer against the *question*.)
    #[test]
    fn the_solution_satisfies_the_system() {
        let rr = [2.0f64, -0.8, 0.25, -0.05];
        let m = 32usize;
        let rhs: Vec<f64> = (0..m).map(|i| ((i * 7 % 13) as f64) / 13.0 - 0.5).collect();
        let x = solve_toeplitz(&rr, &rhs).expect("solved");
        for (i, want) in rhs.iter().enumerate() {
            let row: f64 = (0..m)
                .map(|j| rr.get(i.abs_diff(j)).copied().unwrap_or(0.0) * x[j])
                .sum();
            assert!(
                (row - want).abs() < 1e-9,
                "row {i}: T·x = {row}, but rhs = {want}"
            );
        }
    }
}
