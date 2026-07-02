//! `SplitMix64` — the deterministic, transcendental-free PRNG that seeds NNF
//! initialisation and the PatchMatch random-search jitter. Determinism is a
//! hard requirement (HR-5): the CPU reference must produce byte-identical
//! output for a given seed on every platform, and the GPU path draws the same
//! integer sequence so the two reconcile within float ε. SplitMix64 is pure
//! integer wrapping arithmetic — no `sin`/`exp`/`pow` anywhere.

/// A minimal SplitMix64 generator (Steele/Vigna). One `u64` of state; each draw
/// advances by the golden-ratio odd constant and finalises with the standard
/// avalanche mix. Reproducible and portable.
#[derive(Clone, Copy, Debug)]
pub struct SplitMix64(u64);

impl SplitMix64 {
    /// Seed the generator. Any seed is valid (including 0 — SplitMix64 has no
    /// bad-seed pathology, unlike xorshift).
    #[inline]
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// Next raw 64-bit value.
    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform integer in `[0, bound)`. Returns 0 when `bound == 0` (empty
    /// range) so callers never divide by zero. Uses the simple modulo — the
    /// tiny bias is irrelevant for seeding/jitter and keeps CPU↔GPU identical.
    #[inline]
    pub fn next_u32(&mut self, bound: u32) -> u32 {
        if bound == 0 {
            return 0;
        }
        (self.next_u64() % u64::from(bound)) as u32
    }

    /// Uniform integer in the inclusive range `[lo, hi]`. Returns `lo` when the
    /// range is empty or degenerate (`hi <= lo`).
    #[inline]
    pub fn range_i32(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        let span = (hi - lo + 1) as u32;
        lo + self.next_u32(span) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_is_reproducible() {
        let mut a = SplitMix64::new(12345);
        let mut b = SplitMix64::new(12345);
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn bound_zero_yields_zero_and_never_panics() {
        let mut r = SplitMix64::new(7);
        assert_eq!(r.next_u32(0), 0);
        assert_eq!(r.range_i32(5, 5), 5);
        assert_eq!(r.range_i32(9, 3), 9);
    }

    #[test]
    fn next_u32_stays_in_range() {
        let mut r = SplitMix64::new(99);
        for _ in 0..10_000 {
            assert!(r.next_u32(37) < 37);
        }
    }

    #[test]
    fn range_i32_stays_inclusive() {
        let mut r = SplitMix64::new(2024);
        for _ in 0..10_000 {
            let v = r.range_i32(-4, 4);
            assert!((-4..=4).contains(&v));
        }
    }
}
