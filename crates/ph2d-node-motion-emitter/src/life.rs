//! **QUANTO uma partícula vive** — a variância de vida (doc 89, folha 01: Particular
//! *Life Random %*, Niagara *Lifetime Mode: Random(min,max)*, Cavalry *Override
//! Lifespan*).
//!
//! Separado do `lib.rs` pelo tecto de LOC (HR-18, 700 para `crates/`), na costura que a
//! própria capacidade desenha e pelo molde do [`super::history`]: lá fica *quantas
//! nascem e quando*, aqui *até quando cada uma dura*.

use super::rand01;

/// Hash lane for the particle's LIFE draw — uma pista própria pela mesma razão que o
/// `LANE_SPEED` tem a dele: partilhar uma amarraria a duração de uma partícula à sua
/// direção, e um leque em que as da esquerda vivem mais é um padrão que ninguém pediu.
const LANE_LIFE: u32 = 6;

/// A chave do param **da variância de VIDA**.
///
/// ⚠️ **`life` passa a ser o TETO da janela, e a variância só ENCURTA.** Uma partícula
/// que pudesse viver mais que `life` sairia da janela `[t−life, t]` que a
/// [`super::window`] calcula, e o nó nunca a veria — a janela é a lei da contagem, e ela
/// é aritmética em `life`. Encurtar é a única direção que não a contradiz, e é o que a
/// referência de facto faz: um *Life Random %* reparte para baixo de um máximo.
pub const LIFE_RANDOM: &str = "life_random";

/// **A vida DESTA partícula.**
///
/// ⚠️ **Com `life_random = 0` devolve `life` por RAMO**, e não por uma multiplicação por
/// `1,0`: o default tem de reduzir ao nó que shipava **ao bit**, não a um ULP dele.
///
/// A extração é da IDENTIDADE (`id`), nunca do índice na lista — a janela viva desliza,
/// e uma vida indexada pela posição faria toda partícula trocar de duração no tique em
/// que a mais velha morresse (§0.2 da folha, a doença que já matou o
/// `value.instance_field(Random)` aqui).
pub(super) fn life_of(life: f32, life_random: f32, seed: u32, id: u32) -> f32 {
    if life_random > 0.0 {
        life * (1.0 - life_random.min(1.0) * rand01(seed, id, LANE_LIFE))
    } else {
        life
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O neutro é o param, ao bit.
    #[test]
    fn a_zero_variance_returns_the_param_bit_for_bit() {
        for id in 0..64 {
            assert_eq!(life_of(2.5, 0.0, 7, id).to_bits(), 2.5_f32.to_bits());
        }
    }

    /// ⭐ **Só encurta**, e a faixa é a que o knob declara.
    #[test]
    fn the_draw_only_shortens_and_stays_inside_the_declared_band() {
        let (life, r) = (2.0_f32, 0.7_f32);
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for id in 0..512 {
            let v = life_of(life, r, 3, id);
            lo = lo.min(v);
            hi = hi.max(v);
        }
        assert!(hi <= life, "alguem passou o teto: {hi}");
        assert!(lo >= life * (1.0 - r), "alguem furou o piso: {lo}");
        assert!(hi - lo > life * 0.5, "a faixa colapsou: {lo}..{hi}");
    }

    /// ⚠️ **Um `life_random` acima de `1` não faz a vida NEGATIVA** — o knob é aparado
    /// onde é lido, e não onde é escrito: um param dirigido por fio não passa pelo
    /// slider.
    #[test]
    fn a_driven_param_past_one_never_makes_a_negative_life() {
        for r in [1.0_f32, 3.0, 99.0] {
            for id in 0..128 {
                assert!(life_of(2.0, r, 1, id) >= 0.0, "r={r} id={id}");
            }
        }
    }
}
