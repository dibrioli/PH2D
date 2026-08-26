//! Stateless per-instance randomness: a well-mixed integer hash of
//! `(seed, index, lane)` → `f32 ∈ [0, 1)`. A leaf-local mirror of
//! `motion.emitter`'s `hash.rs` (copied per drop-crate — the shared vocabulary is
//! the *behaviour*, not a shared symbol).
//!
//! ⚠️ **A cópia é a convenção declarada desta biblioteca, e o que a torna segura é o
//! GOLDEN:** o gate `the_hash_agrees_with_the_other_copies` prende o VALOR de três pares
//! `(seed, index)` conhecidos. Uma cópia que derive — um literal trocado, um `>>` a menos —
//! deixa de bater com o número, em vez de deixar de bater com uma cena. *Duplicar uma lei é
//! aceitável quando a divergência é observável de graça.*
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

    /// ⭐ **O GOLDEN das cópias.** Estes três números vêm do `hash.rs` do
    /// `value.instance_field`, que é o mesmo do `motion.emitter` — e são também o oráculo do
    /// `sc_rand01` no WGSL deste nó. Se uma das três cópias derivar, este gate reprova sem
    /// precisar de uma cena, de uma GPU ou de um olho.
    #[test]
    fn the_hash_agrees_with_the_other_copies() {
        for (seed, index) in [(0u32, 0u32), (7, 3), (11, 1000)] {
            let v = rand01(seed, index);
            assert!(
                (0.0..1.0).contains(&v),
                "({seed}, {index}) saiu de alcance: {v}"
            );
        }
        // Os bits exactos — comparados em `to_bits` para o gate falhar num ULP.
        //
        // ⚠️ **Estes três números foram DERIVADOS, não escritos.** A 1.ª versão deste gate
        // trazia-os de cabeça e reprovou no primeiro: `(0, 0)` dá **zero**, porque a
        // avalanche parte de `0·k + 0·k + 0·k` e nenhuma das operações seguintes tira um
        // `0` de lá. *Um golden inventado testa a memória de quem o escreveu.*
        //
        // ⚠️ E o `(0, 0) = 0` é ele próprio uma nota: com `seed = 0` o elemento de índice
        // `0` tira sempre o extremo do intervalo. Não é defeito (a lei é `[0, 1)` e `0`
        // está nele), mas é a razão de um smoke com semente `0` mostrar a 1.ª partícula
        // sempre no extremo — quem estranhar isso está a ver a aritmética, não um bug.
        assert_eq!(rand01(0, 0).to_bits(), 0x0000_0000, "(0, 0)");
        assert_eq!(rand01(7, 3).to_bits(), 0x3edd_93de, "(7, 3)");
        assert_eq!(rand01(11, 1000).to_bits(), 0x3dba_59d0, "(11, 1000)");
    }
}
