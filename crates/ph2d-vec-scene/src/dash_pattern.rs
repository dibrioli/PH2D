//! **O padrão do tracejado** — até [`MAX_DASH_PAIRS`] pares (traço, vão), o *Dashed Line* do painel
//! Stroke do Illustrator (F16c, pedido do dono de 15/10). Plano: `docs/UI_New_and_Simple/vetor_nova_ui/
//! 26_plano_F16_barra_de_controle.md` §4b.
//!
//! Dois tipos, duas unidades:
//! - [`DashPattern`] é o AUTORADO, em **múltiplos da largura** — o que o ficheiro guarda;
//! - [`DashLengths`] é o que um traçador usa, em **comprimento** (já multiplicado pela largura, e
//!   já ajustado ao caminho pelo [`crate::dash_fit`]).

use serde::{Deserialize, Serialize};

/// ⭐ **Três pares**, o número do painel Stroke do Illustrator (*dash · gap ×3*), que é o produto de
/// referência do dono para esta barra. Não é tecto de recurso: um padrão mais longo é exprimível,
/// e subir isto é uma decisão de produto (com degrau no `VEC_SCENE_SCHEMA_VERSION`, porque o array
/// é posicional).
pub const MAX_DASH_PAIRS: usize = 3;

/// **O tracejado autorado** — os pares `[traço, vão]`, em múltiplos da largura do traço.
///
/// ⭐ **Quais pares contam** ([`Self::active`]): o 1.º conta sempre (e com traço `≤ 0` o padrão
/// inteiro é *contínuo* — a lei de antes da F16c, que o slider Dash prometia com o seu `0`); os
/// seguintes contam **enquanto** não forem `[0, 0]` — o 1.º par vazio fecha o padrão, como a casa
/// em branco do Illustrator. Um par `[0, g]` depois do 1.º é legal: com ponta redonda é um ponto.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct DashPattern {
    pub pairs: [[f64; 2]; MAX_DASH_PAIRS],
}

impl DashPattern {
    /// Um par só — o tracejado de antes da F16c, e o que um ficheiro v22 traz.
    #[must_use]
    pub fn single(dash: f64, gap: f64) -> Self {
        let mut pairs = [[0.0; 2]; MAX_DASH_PAIRS];
        pairs[0] = [dash, gap];
        Self { pairs }
    }

    /// Os pares que o padrão de facto percorre — ver a lei no doc do tipo.
    #[must_use]
    pub fn active(&self) -> &[[f64; 2]] {
        let n = 1 + self.pairs[1..]
            .iter()
            .take_while(|[d, g]| *d > 0.0 || *g > 0.0)
            .count();
        &self.pairs[..n]
    }

    /// Todos os pares multiplicados por `k` — o que mantém os COMPRIMENTOS quando a largura muda
    /// por `1/k` (o alinhamento Inner/Outer traça com o dobro da largura).
    #[must_use]
    pub fn scaled(mut self, k: f64) -> Self {
        for p in &mut self.pairs {
            *p = [p[0] * k, p[1] * k];
        }
        self
    }

    /// O padrão traça alguma coisa? `false` ⇒ contínuo.
    #[must_use]
    pub fn is_dashed(&self) -> bool {
        self.pairs[0][0] > 0.0
    }
}

/// **O tracejado em COMPRIMENTO** — `traço, vão, traço, vão, …` no espaço da geometria, pronto para
/// `kurbo::Stroke::with_dashes` e para o `stroke-dasharray` do SVG.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DashLengths {
    v: [f64; 2 * MAX_DASH_PAIRS],
    n: usize,
}

impl DashLengths {
    /// De uma lista de pares em comprimento. ⚠️ Os pares além de [`MAX_DASH_PAIRS`] caem.
    #[must_use]
    pub fn from_pairs(pairs: &[[f64; 2]]) -> Self {
        let mut v = [0.0; 2 * MAX_DASH_PAIRS];
        let n = pairs.len().min(MAX_DASH_PAIRS);
        for (i, [d, g]) in pairs.iter().take(n).enumerate() {
            v[2 * i] = *d;
            v[2 * i + 1] = *g;
        }
        Self { v, n: 2 * n }
    }

    /// Os comprimentos, alternando traço e vão (sempre um número PAR de entradas).
    #[must_use]
    pub fn as_slice(&self) -> &[f64] {
        &self.v[..self.n]
    }

    /// Os pares `[traço, vão]`.
    pub fn pairs(&self) -> impl Iterator<Item = [f64; 2]> + '_ {
        self.as_slice().as_chunks::<2>().0.iter().copied()
    }

    /// O comprimento de UMA volta do padrão.
    #[must_use]
    pub fn period(&self) -> f64 {
        self.as_slice().iter().sum()
    }

    /// O 1.º traço — o que abre (e, num caminho aberto ajustado, fecha) o padrão.
    #[must_use]
    pub fn first_dash(&self) -> f64 {
        self.v[0]
    }

    /// Todos os comprimentos multiplicados por `k` (a razão autorada sobrevive).
    #[must_use]
    pub fn scaled(mut self, k: f64) -> Self {
        for x in &mut self.v[..self.n] {
            *x *= k;
        }
        self
    }
}

/// Um par só, já em comprimento.
impl From<[f64; 2]> for DashLengths {
    fn from(pair: [f64; 2]) -> Self {
        Self::from_pairs(&[pair])
    }
}

impl std::ops::Index<usize> for DashLengths {
    type Output = f64;
    fn index(&self, i: usize) -> &f64 {
        &self.as_slice()[i]
    }
}

impl<const N: usize> PartialEq<[f64; N]> for DashLengths {
    fn eq(&self, other: &[f64; N]) -> bool {
        self.as_slice() == other
    }
}

#[cfg(test)]
#[path = "dash_pattern_tests.rs"]
mod tests;
