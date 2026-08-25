//! **O PERFIL DE DISTÂNCIA de um atrator** — onde a força começa, onde ela é máxima, e
//! onde ela vira do avesso (doc 89, folha 02: as três células do POP Attract).
//!
//! O nó tinha **uma** rampa: `w = curve(1 − d/R)`, máxima no centro e nula em `R`. A
//! referência (Houdini POP Attract) descreve a mesma força com **quatro** números —
//! `Min Distance`, `Peak Force Distance`, `Reversal Distance` e `Max Distance` — e as três
//! células que faltavam eram, todas, *a mesma pergunta*: **qual é a FORMA da resposta ao
//! longo da distância?**
//!
//! ## ⭐ Elas são um perfil, e não três knobs soltos
//!
//! - **`inner`** (*Min Distance*) — dentro dele a força é zero. Hoje é a zona morta.
//! - **`peak`** (*Peak Force Distance*) — onde `w = 1`. Hoje é o centro.
//! - **`reverse`** (*Reversal Distance*) — dentro dele o sinal inverte: o atrator EMPURRA
//!   de perto e PUXA de longe, que é como se autora uma órbita estável sem escrever um
//!   solver.
//! - `radius` (*Max Distance*) — já existia.
//!
//! ⚠️ **Com `peak <= inner` o perfil devolve a rampa de sempre, pela EXPRESSÃO de sempre**
//! (`curve(1 − d/R)`) e não por uma álgebra equivalente: `(R − d)/(R − 0)` é o mesmo número
//! real e **não** os mesmos bits para todo `d`, e o default de um nó que já shipou reduz ao
//! bit ou não reduz.
//!
//! ⚠️ **E o `reverse` inverte o SINAL, não a direção.** Um `repel` que só se aplicasse
//! dentro seria um segundo `repel`, e os dois teriam de decidir quem manda; um sinal que
//! multiplica compõe com o que já existe — `repel` inverte tudo, `reverse` inverte um
//! pedaço, e a composição dos dois é uma multiplicação, não uma tabela de precedência.

/// A chave do param **do raio de dentro** (o *Min Distance* da referência).
pub const INNER: &str = "inner";
/// A chave do param **da distância de PICO**.
pub const PEAK: &str = "peak";
/// A chave do param **da distância de INVERSÃO**.
pub const REVERSE: &str = "reverse";

/// O perfil resolvido de um cook.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Profile {
    /// A borda de dentro, nunca abaixo da zona morta.
    pub(crate) lo: f32,
    pub(crate) peak: f32,
    pub(crate) reverse: f32,
    pub(crate) radius: f32,
    pub(crate) kind: i32,
}

impl Profile {
    /// Constrói o perfil dos params crus.
    ///
    /// ⚠️ **O `lo` nunca desce abaixo da zona morta**, e é isso que faz o default reduzir:
    /// com `inner = 0` a janela volta a ser exactamente `[DEAD_ZONE, radius]`.
    pub(crate) fn of(
        inner: f32,
        peak: f32,
        reverse: f32,
        radius: f32,
        kind: i32,
        dead: f32,
    ) -> Self {
        let radius = radius.max(dead);
        Self {
            lo: if inner.is_finite() {
                inner.max(dead)
            } else {
                dead
            },
            peak: if peak.is_finite() { peak.max(0.0) } else { 0.0 },
            reverse: if reverse.is_finite() {
                reverse.max(0.0)
            } else {
                0.0
            },
            radius,
            kind,
        }
    }

    /// **A rampa de sempre**, sem pico autorado.
    pub(crate) fn is_plain(&self) -> bool {
        self.peak <= self.lo
    }

    /// A posição normalizada `s ∈ [0,1]` que a curva recebe, ou `None` fora da janela.
    ///
    /// ⚠️ **O ramo `is_plain` devolve a expressão LITERAL do nó de sempre.**
    pub(crate) fn shape_at(&self, d: f32) -> Option<f32> {
        if d < self.lo || d > self.radius {
            return None;
        }
        if self.is_plain() {
            return Some((1.0 - d / self.radius).clamp(0.0, 1.0));
        }
        // Duas rampas que se encontram no pico: sobe de `lo` até ele, desce dele até `R`.
        let s = if d <= self.peak {
            (d - self.lo) / (self.peak - self.lo)
        } else {
            // ⚠️ Um pico ENCOSTADO no raio deixa a rampa de descida sem largura; ali o
            // peso é `1` em vez de `0/0`, que é o limite da própria expressão.
            let span = self.radius - self.peak;
            if span > 0.0 {
                (self.radius - d) / span
            } else {
                1.0
            }
        };
        Some(s.clamp(0.0, 1.0))
    }

    /// **A força inverte-se aqui?** — `d` estritamente dentro da distância de inversão.
    pub(crate) fn flips(&self, d: f32) -> bool {
        d < self.reverse
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEAD: f32 = 1e-3;

    fn plain(radius: f32) -> Profile {
        Profile::of(0.0, 0.0, 0.0, radius, 0, DEAD)
    }

    /// ⭐ **O DEFAULT É A RAMPA DE SEMPRE, AO BIT** — a mesma expressão, não uma álgebra
    /// equivalente.
    ///
    /// ⚠️ **A primeira versão deste gate media `R = 4,0` e uma mutação que APAGAVA o ramo
    /// sobreviveu.** Não porque a afirmação seja falsa: porque `4` é uma potência de dois,
    /// e ali `1 − d/R` e `(R−d)/R` arredondam ao MESMO bit em todos os 4001 pontos. Fora
    /// dela divergem — medido: `R = 3,7` diverge em **1877 de 4001**, `R = 5` em 1679,
    /// `R = 6,25` em 1968. *Uma fixture só prova o que contém, e um raio potência-de-dois é
    /// exactamente o caso em que a diferença que o ramo compra não existe.*
    #[test]
    fn the_default_profile_is_the_old_ramp_bit_for_bit() {
        for radius in [4.0_f32, 3.7, 5.0, 6.25] {
            let p = plain(radius);
            assert!(p.is_plain());
            let steps = 4000;
            for k in 0..=steps {
                let d = k as f32 * radius / steps as f32;
                let got = p.shape_at(d);
                let want = if d < DEAD || d > radius {
                    None
                } else {
                    Some((1.0 - d / radius).clamp(0.0, 1.0))
                };
                match (got, want) {
                    (Some(a), Some(b)) => {
                        assert_eq!(a.to_bits(), b.to_bits(), "R={radius} d={d}");
                    }
                    (None, None) => {}
                    _ => panic!("R={radius} d={d}: {got:?} contra {want:?}"),
                }
            }
            // E nada inverte.
            for k in 0..=400 {
                assert!(!p.flips(k as f32 * 0.01));
            }
        }
    }

    /// ⭐⭐ **COM PICO, a força é máxima NO PICO e cai para os dois lados.**
    #[test]
    fn a_peak_makes_the_force_strongest_away_from_the_centre() {
        let p = Profile::of(0.5, 2.0, 0.0, 5.0, 0, DEAD);
        assert!(!p.is_plain());
        assert_eq!(p.shape_at(2.0), Some(1.0), "o pico vale 1");
        assert_eq!(p.shape_at(0.5), Some(0.0), "a borda de dentro vale 0");
        assert_eq!(p.shape_at(5.0), Some(0.0), "a borda de fora vale 0");
        assert_eq!(p.shape_at(0.4), None, "dentro da borda nao ha' forca");
        assert_eq!(p.shape_at(5.1), None, "fora do raio idem");
        // Monotónica de cada lado.
        let up: Vec<f32> = (0..20)
            .map(|k| p.shape_at(0.5 + k as f32 * 0.075).unwrap())
            .collect();
        let down: Vec<f32> = (0..20)
            .map(|k| p.shape_at(2.0 + k as f32 * 0.15).unwrap())
            .collect();
        for w in up.windows(2) {
            assert!(w[1] >= w[0] - 1e-6, "a subida caiu: {w:?}");
        }
        for w in down.windows(2) {
            assert!(w[1] <= w[0] + 1e-6, "a descida subiu: {w:?}");
        }
    }

    /// ⭐ **A INVERSÃO é um pedaço de dentro**, e ela compõe com o `repel` em vez de
    /// competir com ele.
    #[test]
    fn the_reversal_flips_only_inside_its_own_distance() {
        let p = Profile::of(0.0, 0.0, 1.5, 5.0, 0, DEAD);
        assert!(p.flips(0.2) && p.flips(1.4));
        assert!(!p.flips(1.5), "a borda NAO inverte -- ela e' o limite");
        assert!(!p.flips(3.0));
    }

    /// ⚠️ **Um pico ENCOSTADO no raio não divide por zero** — o peso ali é `1`, que é o
    /// limite da própria expressão, e não um `NaN` que envenena o `accel` inteiro.
    #[test]
    fn a_peak_at_the_rim_never_divides_by_zero() {
        let p = Profile::of(0.1, 5.0, 0.0, 5.0, 0, DEAD);
        for k in 0..=100 {
            let d = k as f32 * 0.05;
            if let Some(s) = p.shape_at(d) {
                assert!(s.is_finite(), "d={d} deu {s}");
                assert!((0.0..=1.0).contains(&s), "d={d} deu {s}");
            }
        }
    }

    /// ⚠️ Params doentes (um fio pode entregar `NaN`) caem no perfil de sempre.
    #[test]
    fn a_driven_param_can_be_anything() {
        for bad in [f32::NAN, f32::INFINITY, -3.0] {
            let p = Profile::of(bad, bad, bad, 4.0, 0, DEAD);
            for k in 0..=100 {
                let d = k as f32 * 0.05;
                if let Some(s) = p.shape_at(d) {
                    assert!(s.is_finite(), "bad={bad} d={d}");
                }
            }
        }
    }
}
