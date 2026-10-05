//! Stateless per-instance randomness: a well-mixed integer hash of
//! `(seed, index)` → `f32 ∈ [0, 1)`: this crate's wrapper (`rand01(seed, index)`, lane `0`)
//! over the shared law `ph2d_motion_kit::hash::hash3` (bug #11 — one door, not a per-crate
//! copy).
//!
//! ⚠️ **O GOLDEN fica aqui** porque é o oráculo do `sc_rand01` do WGSL deste nó (que É uma
//! segunda escrita da lei, em `gpu.rs`): o gate `the_hash_agrees_with_the_other_copies`
//! prende o VALOR de três pares `(seed, index)` conhecidos através do embrulho deste nó. O
//! mesmo golden, sobre a lei crua, vive em `ph2d_motion_kit::hash` (`hash3_golden_bits`).
//!
//! **Stateless is the whole point** (Jarzynski & Olano 2020): an instance's draw
//! is a pure function of its identity, never of a stream of draws — so the field
//! is `Effect::Pure`, scrubbing reproduces it bit-for-bit, and a GPU lowering
//! computes the same value per lane. Transcendental-free (HR-5).

use ph2d_motion_kit::hash::hash3;

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
