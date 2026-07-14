//! Stateless hash of `(seed, draw, lane)` → `f32 ∈ [0, 1)`. A leaf-local mirror of
//! `motion.scatter`'s `hash.rs` (copied per drop-crate — the shared vocabulary is the
//! *behaviour*, not a shared symbol).
//!
//! Stateless (Jarzynski & Olano 2020): a draw is a pure function of its identity, so
//! the layout reproduces bit-for-bit from the seed alone and the node is
//! `Effect::Pure`. Transcendental-free (HR-5).

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

/// **The draw sequence.** Bridson's algorithm is sequential — how many darts it throws
/// depends on where the last one landed — so unlike `motion.scatter` (whose every
/// candidate is a pure function of its index) this node cannot hash the draw straight
/// from an element id: it needs a *stream* of them.
///
/// A counter fed into the same stateless hash is that stream, and it keeps the
/// property that matters: **the whole layout is a pure function of the seed.** The
/// counter is deterministic because the algorithm is, so re-cooking the same params
/// re-throws the same darts in the same order — scrub-stable, bit-exact, `Pure`.
pub(crate) struct Draws {
    pub(crate) seed: u32,
    pub(crate) n: u32,
}

impl Draws {
    /// The next draw in `[0, 1)`.
    pub(crate) fn next(&mut self) -> f32 {
        let v = hash3(self.seed, self.n, 0);
        self.n = self.n.wrapping_add(1);
        v
    }
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

    /// The counter is the stream, and two runs of it agree — the property the whole
    /// node's determinism rests on.
    #[test]
    fn the_counter_replays_the_same_sequence() {
        let mut a = Draws { seed: 3, n: 0 };
        let mut b = Draws { seed: 3, n: 0 };
        let first: Vec<f32> = (0..64).map(|_| a.next()).collect();
        let again: Vec<f32> = (0..64).map(|_| b.next()).collect();
        assert_eq!(first, again);
        // Consecutive draws are decorrelated (an avalanche hash, not a ramp).
        assert!(first.windows(2).any(|w| (w[0] - w[1]).abs() > 0.2));
    }
}
