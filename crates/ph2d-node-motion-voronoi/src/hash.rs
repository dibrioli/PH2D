//! Stateless hash of `(seed, index, lane)` → `f32 ∈ [0, 1)`. A leaf-local mirror
//! of `motion.emitter`/`value.instance_field`'s `hash.rs` (copied per drop-crate —
//! the shared vocabulary is the *behaviour*, not a shared symbol).
//!
//! Stateless (Jarzynski & Olano 2020): a draw is a pure function of its identity,
//! so the scatter is `Effect::Pure`, reproduces bit-for-bit, and never needs a
//! stream of draws. Transcendental-free (HR-5).

/// splitmix-style avalanche on a 32-bit lattice → `[0, 1)`.
pub(crate) fn hash3(a: u32, b: u32, lane: u32) -> f32 {
    let mut h = a
        .wrapping_mul(0x9e37_79b9)
        .wrapping_add(b.wrapping_mul(0x85eb_ca6b))
        .wrapping_add(lane.wrapping_mul(0xc2b2_ae35));
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb_352d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846c_a68b);
    h ^= h >> 16;
    // 24 bits into the mantissa → exactly representable, uniform, and never 1.0.
    (h >> 8) as f32 / (1u32 << 24) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draws_are_in_range_and_reproducible() {
        for i in 0..500u32 {
            let v = hash3(7, i, 0);
            assert!((0.0..1.0).contains(&v), "draw {v} out of range at {i}");
            assert_eq!(v, hash3(7, i, 0), "the same identity always redraws");
        }
    }

    #[test]
    fn lanes_and_indices_decorrelate() {
        assert_ne!(hash3(0, 1, 0), hash3(0, 1, 1), "lanes differ (x vs y)");
        assert_ne!(hash3(0, 1, 0), hash3(0, 2, 0), "indices differ");
    }
}
