//! O sorteio da probabilidade: um hash inteiro bem misturado de
//! `(linha, ordinal do pulso)` → `f32 ∈ [0, 1)`.
//!
//! **Sem estado é o ponto inteiro** (o mesmo argumento do `motion.emitter`,
//! Jarzynski & Olano 2020): o sorteio de uma instância é função pura da
//! identidade dela e de QUAL pulso é, nunca de uma sequência de saques. Então o
//! scrub para trás reproduz a mesma cena bit a bit — o `CheckpointRing` devolve a
//! fase e a pista, e os sorteios re-saem iguais. Transcendental-free (HR-5).
//!
//! ⚠️ **Espelhado de propósito**, não importado: uma crate-nó não depende de
//! outra crate-nó (ADR-0075), e o vocabulário partilhado é a LEI (um avalanche
//! splitmix sobre uma rede de 32 bits), não um símbolo. É o mesmo espelho que o
//! `motion.wiggle`, o `force.curl` e o `force.wind` já carregam.

/// splitmix64-style avalanche numa rede de 32 bits → `[0, 1)`.
fn hash2(a: u32, b: u32) -> f32 {
    let mut h = a
        .wrapping_mul(0x9e37_79b9)
        .wrapping_add(b.wrapping_mul(0x85eb_ca6b));
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb_352d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846c_a68b);
    h ^= h >> 16;
    // 24 bits na mantissa → exatamente representável, uniforme, e nunca 1.0.
    (h >> 8) as f32 / (1u32 << 24) as f32
}

/// O saque da linha `row` no seu `ordinal`-ésimo pulso, em `[0, 1)`.
pub(crate) fn rand01(row: u32, ordinal: u32) -> f32 {
    hash2(row, ordinal)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **O mesmo par re-sorteia o mesmo número, sempre** — é o que torna o scrub
    /// exato.
    #[test]
    fn a_draw_is_a_pure_function_of_its_identity() {
        for row in 0..300u32 {
            let v = rand01(row, 3);
            assert!((0.0..1.0).contains(&v), "saque {v} fora da faixa em {row}");
            assert_eq!(v, rand01(row, 3));
        }
    }

    /// **Linhas e ordinais decorrelacionam** — sem isto, ou todas as instâncias
    /// acendem juntas, ou uma que recusou recusa para sempre.
    #[test]
    fn rows_and_ordinals_decorrelate() {
        assert_ne!(rand01(1, 0), rand01(2, 0), "linhas diferem");
        assert_ne!(rand01(1, 0), rand01(1, 1), "ordinais diferem");
    }

    /// **A distribuição enche o intervalo** — um saque enviesado faria a
    /// `probability` significar outro número que não ela mesma.
    #[test]
    fn the_draw_spreads_over_the_unit_interval() {
        let mut buckets = [0u32; 4];
        for row in 0..1000u32 {
            let q = (rand01(row, 7) * 4.0) as usize;
            buckets[q.min(3)] += 1;
        }
        assert!(
            buckets.iter().all(|&c| c > 150),
            "quartis sub-preenchidos: {buckets:?}"
        );
    }
}
