//! Small numeric helpers shared across painters / interaction code.
//!
//! Wave 10 / Etapa 5.3 — `safe_clamp` is the NaN-aware, swap-tolerant
//! companion to `f32::clamp`. Use it whenever the bounds come from a
//! dynamic source (store values, computed layout, drag deltas) instead
//! of literal constants.

/// NaN-aware, swap-tolerant clamp.
///
/// `f32::clamp` panics if `min > max` and propagates NaN (`v.is_nan()`
/// → returns NaN). UI code that fed `clamp(prev_min, prev_max)` from
/// store values has crashed editors when a malformed edit inverted
/// the bounds, or when an upstream layout produced NaN.
///
/// This variant:
///   - returns `min` (the recovered lower bound) if `v` is NaN,
///   - swaps the bounds if `min > max`,
///   - falls through to `f32::clamp` otherwise.
#[inline]
pub fn safe_clamp(v: f32, min: f32, max: f32) -> f32 {
    let (lo, hi) = if min <= max { (min, max) } else { (max, min) };
    if v.is_nan() {
        return lo;
    }
    v.clamp(lo, hi)
}

/// [`safe_clamp`] em `f64` — a MESMA lei, para quem faz a conta em `f64` (a caixa de número do
/// cartão do grafo: converter para `f32` antes da face mudaria o arredondamento do que se escreve).
#[inline]
pub fn safe_clamp_f64(v: f64, min: f64, max: f64) -> f64 {
    let (lo, hi) = if min <= max { (min, max) } else { (max, min) };
    if v.is_nan() {
        return lo;
    }
    v.clamp(lo, hi)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nan_falls_back_to_min() {
        assert_eq!(safe_clamp(f32::NAN, 0.0, 1.0), 0.0);
    }

    #[test]
    fn inverted_bounds_swap() {
        // min > max — `f32::clamp` would panic; safe_clamp swaps.
        assert_eq!(safe_clamp(0.5, 1.0, 0.0), 0.5);
        assert_eq!(safe_clamp(-1.0, 1.0, 0.0), 0.0);
        assert_eq!(safe_clamp(2.0, 1.0, 0.0), 1.0);
    }

    #[test]
    fn the_f64_twin_obeys_the_same_law() {
        assert_eq!(safe_clamp_f64(f64::NAN, 0.0, 1.0), 0.0);
        assert_eq!(safe_clamp_f64(-1.0, 1.0, 0.0), 0.0);
        assert_eq!(safe_clamp_f64(2.0, 0.0, 1.0), 1.0);
        assert_eq!(safe_clamp_f64(0.503, 0.0, 1.0), 0.503);
    }

    #[test]
    fn normal_bounds_match_f32_clamp() {
        assert_eq!(safe_clamp(0.5, 0.0, 1.0), 0.5);
        assert_eq!(safe_clamp(-1.0, 0.0, 1.0), 0.0);
        assert_eq!(safe_clamp(2.0, 0.0, 1.0), 1.0);
    }
}
