//! **Linear prediction (LPC)** — the shared core under De-Click and Formant Shift.
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
