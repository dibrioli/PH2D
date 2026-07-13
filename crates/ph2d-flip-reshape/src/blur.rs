//! O **kernel binomial 1D** — o alisador do Flip, com UM dono só.
//!
//! Duas coisas o usam, e por isso ele mora aqui (e não no shell): o **active
//! smoothing** do traço em curso (o "assentar" da cauda enquanto a ponta segue o
//! cursor) e o pincel **Smooth** do Reshape. A diferença entre eles não é o kernel —
//! é a *influência*: o active smoothing alisa tudo por igual, o pincel alisa **por
//! ponto**, na medida em que o pincel o alcança.
//!
//! `[1,2,1]/4` repetido é a aproximação gaussiana **polinomial** (HR-5: zero
//! transcendental — e, ao contrário da forma `exp`, é bit-determinística entre
//! plataformas).

use ph2d_core::Vec2;

/// Um valor que o kernel sabe alisar (posição, largura, opacidade).
pub trait Blurrable: Copy {
    /// `(a + 2b + c) / 4` — o passo do kernel.
    fn kernel(a: Self, b: Self, c: Self) -> Self;
    /// Mistura linear (`t = 0` → self; `t = 1` → other).
    fn mix(self, other: Self, t: f32) -> Self;
}

impl Blurrable for f32 {
    fn kernel(a: Self, b: Self, c: Self) -> Self {
        (a + 2.0 * b + c) * 0.25
    }
    fn mix(self, other: Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

impl Blurrable for Vec2 {
    fn kernel(a: Self, b: Self, c: Self) -> Self {
        (a + b * 2.0 + c) * 0.25
    }
    fn mix(self, other: Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

/// Como o kernel trata as PONTAS de um traço aberto.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Ends {
    /// As pontas não se movem (o 1º e o último ponto ficam onde estão).
    ///
    /// É o que o **active smoothing** quer (a ponta segue o cursor sem lag) e o que
    /// o pincel **Smooth** quer: sem isto, alisar a ponta de um traço a **encolhe** —
    /// cada passe puxa o extremo para dentro, e um traço trabalhado encurta em
    /// silêncio (o aceite da T5.2: *"alisar traço trêmulo sem encolher pontas"*).
    Anchored,
    /// As pontas também alisam (o vizinho ausente é o espelho do presente).
    Smoothed,
}

/// Aplica o kernel `[1,2,1]/4` `iterations` vezes e mistura o resultado no
/// original pela `influence` **de cada ponto**.
///
/// - `influence[i] = 0` → o ponto `i` fica intacto;
/// - `influence[i] = 1` → o ponto `i` vira o valor alisado.
///
/// O alisamento é calculado sobre **TODOS** os pontos (não só os influenciados):
/// o kernel de um ponto lê os vizinhos, e os vizinhos podem estar fora do alcance
/// do pincel. Mascarar a *entrada* faria a borda do pincel deformar o traço em vez
/// de alisá-lo — a máscara pertence à SAÍDA (a mistura), nunca ao kernel.
///
/// `cyclic` propaga através do fecho (um traço fechado não tem pontas).
pub fn binomial<T: Blurrable>(
    values: &[T],
    iterations: u32,
    influence: &dyn Fn(usize) -> f32,
    ends: Ends,
    cyclic: bool,
) -> Vec<T> {
    let n = values.len();
    let mut out = values.to_vec();
    if n < 3 || iterations == 0 {
        return out;
    }
    let mut cur = values.to_vec();
    let mut next = cur.clone();
    for _ in 0..iterations {
        for i in 0..n {
            // O vizinho que falta numa PONTA aberta: em `Anchored` o ponto nem se
            // move; em `Smoothed` o vizinho ausente é o espelho do presente (o que
            // preserva a média — repetir o próprio ponto puxaria o extremo para
            // dentro, que é o encolhimento que se quer evitar).
            let (a, c) = match (cyclic, i) {
                (true, 0) => (cur[n - 1], cur[1]),
                (true, i) if i == n - 1 => (cur[n - 2], cur[0]),
                (false, 0) => {
                    if ends == Ends::Anchored {
                        next[0] = cur[0];
                        continue;
                    }
                    (cur[1], cur[1])
                }
                (false, i) if i == n - 1 => {
                    if ends == Ends::Anchored {
                        next[i] = cur[i];
                        continue;
                    }
                    (cur[i - 1], cur[i - 1])
                }
                _ => (cur[i - 1], cur[i + 1]),
            };
            next[i] = T::kernel(a, cur[i], c);
        }
        std::mem::swap(&mut cur, &mut next);
    }
    for (i, o) in out.iter_mut().enumerate() {
        let t = influence(i).clamp(0.0, 1.0);
        if t > 0.0 {
            *o = o.mix(cur[i], t);
        }
    }
    out
}

/// O caso uniforme (a mesma influência em todo ponto) — o do active smoothing.
#[must_use]
pub fn binomial_uniform<T: Blurrable>(
    values: &[T],
    iterations: u32,
    influence: f32,
    ends: Ends,
    cyclic: bool,
) -> Vec<T> {
    binomial(values, iterations, &|_| influence, ends, cyclic)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Um dente isolado numa reta é achatado, e as PONTAS não andam.
    #[test]
    fn the_kernel_flattens_a_spike_and_pins_the_ends() {
        let mut v: Vec<f32> = (0..7).map(|_| 0.0).collect();
        v[3] = 1.0;
        v[0] = 5.0; // a ponta: tem de ficar ONDE ESTÁ
        let out = binomial_uniform(&v, 2, 1.0, Ends::Anchored, false);
        assert_eq!(out[0], 5.0, "a ponta ancorada nao pode andar");
        assert_eq!(out[6], 0.0, "a outra ponta idem");
        assert!(out[3] < 1.0 && out[3] > 0.0, "o dente achatou: {}", out[3]);
        assert!(out[2] > 0.0, "e espalhou para o vizinho");
    }

    /// **A influência é da SAÍDA, não da entrada:** um ponto com influência 0 fica
    /// intacto mesmo que o vizinho dele seja alisado.
    #[test]
    fn zero_influence_leaves_the_point_untouched() {
        let v: Vec<f32> = vec![0.0, 9.0, 0.0, 9.0, 0.0];
        let out = binomial(
            &v,
            3,
            &|i| if i == 3 { 0.0 } else { 1.0 },
            Ends::Anchored,
            false,
        );
        assert_eq!(out[3], 9.0, "influencia 0 = ponto intacto");
        assert!(out[1] < 9.0, "o vizinho influenciado alisou");
    }

    /// Meia influência = meio caminho (a mistura é linear).
    #[test]
    fn half_influence_is_half_way_to_the_smoothed_value() {
        let v: Vec<f32> = vec![0.0, 4.0, 0.0, 4.0, 0.0];
        let full = binomial_uniform(&v, 2, 1.0, Ends::Anchored, false);
        let half = binomial_uniform(&v, 2, 0.5, Ends::Anchored, false);
        for i in 1..4 {
            let want = v[i] + (full[i] - v[i]) * 0.5;
            assert!(
                (half[i] - want).abs() < 1e-6,
                "ponto {i}: {} != {want}",
                half[i]
            );
        }
    }

    /// Num traço FECHADO não há ponta: o alisamento atravessa o fecho.
    #[test]
    fn a_cyclic_stroke_smooths_across_the_seam() {
        let v: Vec<f32> = vec![9.0, 0.0, 0.0, 0.0, 0.0];
        let out = binomial_uniform(&v, 1, 1.0, Ends::Anchored, true);
        assert!(out[0] < 9.0, "o ponto do fecho alisou: {}", out[0]);
        assert!(
            out[4] > 0.0,
            "e vazou para o vizinho DO OUTRO LADO do fecho"
        );
    }

    /// **O alisamento não encolhe um traço aberto** (`Anchored`): o comprimento da
    /// corda ponta-a-ponta é preservado exatamente.
    #[test]
    fn smoothing_a_wobbly_line_does_not_shorten_it() {
        let pts: Vec<Vec2> = (0..21)
            .map(|i| {
                let x = i as f32;
                let wobble = if i % 2 == 0 { 0.4 } else { -0.4 };
                Vec2::new(x, wobble)
            })
            .collect();
        let out = binomial_uniform(&pts, 4, 1.0, Ends::Anchored, false);
        assert_eq!(out[0], pts[0], "a ponta inicial nao andou");
        assert_eq!(out[20], pts[20], "a ponta final nao andou");
        // E o tremor sumiu: a soma dos |desvios| em y cai muito.
        let before: f32 = pts.iter().map(|p| p.y.abs()).sum();
        let after: f32 = out.iter().map(|p| p.y.abs()).sum();
        assert!(
            after < before * 0.5,
            "o tremor tem de cair: {before} -> {after}"
        );
    }
}
