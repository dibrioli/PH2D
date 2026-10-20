//! **As curvas de um critério** do tutorial do Painter (W7, `docs/Macros/04_plano_W7.md`): a fracção
//! ao longo do assentar, e a da gravada contra a da base.

use super::DELTA_MINIMO;

/// **Uma fracção ao longo do assentar** (W7): `(idade, fracção)` por idade crescente, nunca vazia — a
/// idade em quadros (60 Hz) desde o último `Up` da trilha medida
/// ([`PainterTool::segundos_desde_o_ultimo_up`]).
#[derive(Clone, Debug, PartialEq)]
pub struct Curva(pub Vec<(u32, f32)>);

impl Curva {
    /// A mesma fracção em qualquer idade.
    #[must_use]
    pub fn constante(f: f32) -> Self {
        Self(vec![(0, f)])
    }

    /// A fracção na idade `q`, interpolada entre os pontos; antes do 1.º vale o 1.º, depois do último
    /// o último.
    #[must_use]
    pub fn em(&self, q: u32) -> f32 {
        let i = self.0.partition_point(|&(a, _)| a <= q);
        match (i.checked_sub(1).map(|j| self.0[j]), self.0.get(i)) {
            (Some((a, fa)), Some(&(b, fb))) => {
                #[expect(clippy::cast_precision_loss, reason = "idades em quadros")]
                let x = (q - a) as f32 / (b - a) as f32;
                fa + (fb - fa) * x
            }
            (Some((_, f)), None) | (None, Some(&(_, f))) => f,
            (None, None) => 0.0,
        }
    }

    /// A fracção do último ponto (a água assentada).
    #[must_use]
    pub fn fim(&self) -> f32 {
        self.0.last().map_or(0.0, |p| p.1)
    }

    /// A idade do último ponto.
    #[must_use]
    pub fn ate(&self) -> u32 {
        self.0.last().map_or(0, |p| p.0)
    }
}

/// Quanto dos `novos` de uma pincelada a GRAVAÇÃO pinta, com ela e sem ela, ao longo do assentar —
/// cada curva na idade da SUA trilha.
#[derive(Clone, Debug, PartialEq)]
pub struct Referencia {
    /// A trilha que toca, desde o `Up` da pincelada.
    pub gravada: Curva,
    /// A trilha que toca SEM esta pincelada (o que a água das outras pinta sozinha), desde o `Up` da
    /// anterior que lá ficou.
    pub base: Curva,
}

impl Referencia {
    /// As duas curvas constantes.
    #[must_use]
    pub fn constante(gravada: f32, base: f32) -> Self {
        Self {
            gravada: Curva::constante(gravada),
            base: Curva::constante(base),
        }
    }

    /// O que a pincelada acrescenta à água ASSENTADA (a lei do Δ vazio).
    #[must_use]
    pub fn delta(&self) -> f32 {
        self.gravada.fim() - self.base.fim()
    }

    /// A base e o que a pincelada acrescenta na idade `q` (o acréscimo nunca abaixo de
    /// [`DELTA_MINIMO`]: a guarda da divisão).
    #[must_use]
    pub fn em(&self, q: u32) -> (f32, f32) {
        let b = self.base.em(q);
        (b, (self.gravada.em(q) - b).max(DELTA_MINIMO))
    }

    /// A idade a partir da qual nenhuma das duas muda.
    #[must_use]
    pub fn ate(&self) -> u32 {
        self.gravada.ate().max(self.base.ate())
    }
}
