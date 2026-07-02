//! Counter-based 32-bit hash PRNG — the per-pixel randomness for NNF init and
//! random search. Unlike a stateful stream, a counter hash is a pure function of
//! `(seed, index, salt, draw)`, so a massively-parallel GPU thread reproduces the
//! *exact same* draws the CPU reference makes for its pixel. Crucially it is
//! **32-bit only** (`u32` ops): WGSL has no `u64`, so a splitmix64 stream could
//! not be mirrored on GPU — this can. That is what lets the W2 compute path
//! reconcile with the CPU reference within float ε (ADR-0102).
//!
//! All operations are integer wrapping arithmetic — transcendental-free (HR-5).

/// A single-word integer avalanche hash (Wang/Jenkins-style), all `u32` ops so a
/// WGSL kernel reproduces it bit-for-bit.
#[inline]
pub fn hash32(mut x: u32) -> u32 {
    x ^= x >> 17;
    x = x.wrapping_mul(0xed5a_d4bb);
    x ^= x >> 11;
    x = x.wrapping_mul(0xac4c_1b51);
    x ^= x >> 15;
    x = x.wrapping_mul(0x3184_8bab);
    x ^= x >> 14;
    x
}

/// A counter-based draw: hash `(seed, index, salt, draw)` into an independent
/// `u32`. `draw` is the 0-based draw number within this pixel's stream.
#[inline]
pub fn rand_u32(seed: u32, idx: u32, salt: u32, draw: u32) -> u32 {
    let a = hash32(salt ^ draw.wrapping_mul(0x85eb_ca77));
    let b = hash32(idx.wrapping_mul(0x9e37_79b1) ^ a);
    hash32(seed ^ b)
}

/// Fold a `u64` seed into the `u32` the CPU and GPU both consume.
#[inline]
pub fn seed32(seed: u64) -> u32 {
    (seed ^ (seed >> 32)) as u32
}

/// A counter-based draw mapped into the inclusive range `[lo, hi]`. Returns `lo`
/// when the range is empty (`hi <= lo`). Modulo mapping — matches the WGSL side.
#[inline]
pub fn rand_range(seed: u32, idx: u32, salt: u32, draw: u32, lo: i32, hi: i32) -> i32 {
    if hi <= lo {
        return lo;
    }
    let span = (hi - lo + 1) as u32;
    lo + (rand_u32(seed, idx, salt, draw) % span) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_for_same_inputs() {
        assert_eq!(rand_u32(7, 42, 3, 0), rand_u32(7, 42, 3, 0));
        assert_eq!(rand_range(1, 2, 3, 4, -5, 5), rand_range(1, 2, 3, 4, -5, 5));
    }

    #[test]
    fn range_stays_inclusive_and_covers() {
        let mut seen_lo = false;
        let mut seen_hi = false;
        for draw in 0..5000 {
            let v = rand_range(99, 7, 1, draw, -3, 3);
            assert!((-3..=3).contains(&v));
            seen_lo |= v == -3;
            seen_hi |= v == 3;
        }
        assert!(seen_lo && seen_hi, "range endpoints should both appear");
    }

    #[test]
    fn empty_range_returns_lo() {
        assert_eq!(rand_range(1, 1, 1, 1, 5, 5), 5);
        assert_eq!(rand_range(1, 1, 1, 1, 9, 2), 9);
    }

    #[test]
    fn distinct_pixels_decorrelate() {
        // Two adjacent pixels drawing #0 should (almost surely) differ.
        assert_ne!(rand_u32(4, 100, 0, 0), rand_u32(4, 101, 0, 0));
    }
}
