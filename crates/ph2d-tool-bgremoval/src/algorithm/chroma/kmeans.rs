//! **O agrupamento k=2 das amostras de canto** (o passo 2 do pipeline do `chroma`), irmão do
//! `chroma/mod.rs` por tecto de LOC.
//!
//! Corte mecânico: a função saiu inteira, verbatim; abriu-se só a visibilidade para o módulo pai.

use super::{KMEANS_MAX_ITERS, oklab_dist_sq};

/// Serial k=2 Lloyd k-means in Oklab. Deterministic init: centroid
/// A = sample with lowest L, centroid B = sample with highest L
/// (tie-broken by index → deterministic). Returns the centroid of
/// the cluster with the larger membership.
pub(super) fn kmeans_k2(samples: &[[f32; 3]], assignments: &mut [u8]) -> [f32; 3] {
    debug_assert_eq!(samples.len(), assignments.len());
    if samples.is_empty() {
        return [0.0; 3];
    }
    if samples.len() == 1 {
        return samples[0];
    }

    // Init: pick min-L and max-L samples as initial centroids.
    let (mut ci_a, mut ci_b) = (0usize, 0usize);
    let (mut min_l, mut max_l) = (samples[0][0], samples[0][0]);
    for (i, s) in samples.iter().enumerate() {
        if s[0] < min_l {
            min_l = s[0];
            ci_a = i;
        }
        if s[0] > max_l {
            max_l = s[0];
            ci_b = i;
        }
    }
    if ci_a == ci_b {
        // Monochrome corners; pick any second seed.
        ci_b = (ci_a + 1) % samples.len();
    }
    let mut centroid_a = samples[ci_a];
    let mut centroid_b = samples[ci_b];

    let mut last_changes = u32::MAX;
    for _ in 0..KMEANS_MAX_ITERS {
        // Assign.
        let mut changes = 0u32;
        for (i, s) in samples.iter().enumerate() {
            let da = oklab_dist_sq(*s, centroid_a);
            let db = oklab_dist_sq(*s, centroid_b);
            let new_assign = if da <= db { 0u8 } else { 1u8 };
            if assignments[i] != new_assign {
                changes += 1;
                assignments[i] = new_assign;
            }
        }
        // Early-exit once no sample changes cluster. The first
        // pass starts from `assignments` initialised to `0` (so
        // it always records "changes"), guaranteeing centroid_b
        // gets at least one update — no risk of premature exit.
        if changes == 0 && last_changes != u32::MAX {
            break;
        }
        last_changes = changes;

        // Update centroids — serial sum, deterministic order.
        let (mut sa, mut sb, mut na, mut nb) = ([0.0f32; 3], [0.0f32; 3], 0u32, 0u32);
        for (i, s) in samples.iter().enumerate() {
            match assignments[i] {
                0 => {
                    sa[0] += s[0];
                    sa[1] += s[1];
                    sa[2] += s[2];
                    na += 1;
                }
                _ => {
                    sb[0] += s[0];
                    sb[1] += s[1];
                    sb[2] += s[2];
                    nb += 1;
                }
            }
        }
        if na > 0 {
            let inv = 1.0 / (na as f32);
            centroid_a = [sa[0] * inv, sa[1] * inv, sa[2] * inv];
        }
        if nb > 0 {
            let inv = 1.0 / (nb as f32);
            centroid_b = [sb[0] * inv, sb[1] * inv, sb[2] * inv];
        }
    }

    // Final membership tally. `>=` tie-breaks toward `centroid_a`
    // (the min-L seed) — deterministic regardless of sample order.
    let (mut na, mut nb) = (0u32, 0u32);
    for a in assignments.iter() {
        match a {
            0 => na += 1,
            _ => nb += 1,
        }
    }
    if na >= nb { centroid_a } else { centroid_b }
}
