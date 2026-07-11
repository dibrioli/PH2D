//! Stateless per-instance randomness: a well-mixed integer hash of
//! `(seed, index, lane)` → `f32 ∈ [0, 1)`. A leaf-local mirror of
//! `motion.emitter`'s `hash.rs` (copied per drop-crate — the shared vocabulary is
//! the *behaviour*, not a shared symbol).
//!
//! **Stateless is the whole point** (Jarzynski & Olano 2020): an instance's draw
//! is a pure function of its identity, never of a stream of draws — so the field
//! is `Effect::Pure`, scrubbing reproduces it bit-for-bit, and a GPU lowering
//! computes the same value per lane. Transcendental-free (HR-5).

/// splitmix-style avalanche on a 32-bit lattice → `[0, 1)`.
fn hash3(a: u32, b: u32, lane: u32) -> f32 {
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

/// Instance `index`'s draw for `seed`, in `[0, 1)`.
pub(crate) fn rand01(seed: u32, index: u32) -> f32 {
    hash3(seed, index, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draws_are_in_range_and_reproducible() {
        for i in 0..500u32 {
            let v = rand01(7, i);
            assert!((0.0..1.0).contains(&v), "draw {v} out of range at {i}");
            assert_eq!(v, rand01(7, i), "the same identity always redraws");
        }
    }

    #[test]
    fn seeds_and_indices_decorrelate() {
        assert_ne!(rand01(0, 1), rand01(1, 1), "seeds differ");
        assert_ne!(rand01(0, 1), rand01(0, 2), "indices differ");
    }

    #[test]
    fn the_draw_spreads_over_the_unit_interval() {
        let mut buckets = [0u32; 4];
        for i in 0..1000u32 {
            let q = (rand01(3, i) * 4.0) as usize;
            buckets[q.min(3)] += 1;
        }
        assert!(
            buckets.iter().all(|&c| c > 150),
            "quartiles under-filled: {buckets:?}"
        );
    }
}
