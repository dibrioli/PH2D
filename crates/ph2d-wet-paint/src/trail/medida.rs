//! **As portas de MEDIÇÃO da janela** — o que os gates leem do trail e o produto nunca lê (filho de [`super`],
//! LOC cap).

use super::*;

impl Trail {
    /// **A meia-largura VIVA da janela** — para o gate que julga o cap.
    ///
    /// ⚠️ Só para MEDIÇÃO: o produto nunca a lê, e um gate que a comparasse com
    /// a fórmula estaria espelhando a regra que julga. Quem julga o cap mede o
    /// RETÂNGULO TOCADO (`touched_extent_for_measure`); esta existe só para a
    /// metade oposta — provar que um pincel dentro do teto do modelo de
    /// referência não moveu nada.
    #[must_use]
    pub fn window_half_for_measure(&self) -> i32 {
        self.half
    }

    /// Os planos de pigmento e de água da janela, inteiros — o oráculo do gate
    /// de identidade das duas rotas do depósito (ADR-0175). Só para MEDIÇÃO:
    /// o produto só os lê pelo transfer.
    #[must_use]
    pub fn planes_for_measure(&self) -> (&[f32], &[f32]) {
        (&self.pig, &self.water)
    }

    /// A âncora da janela em células de canvas — com a meia-largura e a
    /// extensão, é o que um oráculo precisa para saber que célula cada texel
    /// da janela cobre (ADR-0175). Só para MEDIÇÃO.
    #[must_use]
    pub fn anchor_for_measure(&self) -> (i32, i32) {
        (self.anchor_x, self.anchor_y)
    }

    /// Os três planos do BICO, inteiros — o oráculo do gate de identidade das
    /// duas rotas do transfer (ADR-0175). Só para MEDIÇÃO.
    #[must_use]
    pub fn tip_planes_for_measure(&self) -> (&[f32], &[f32], &[f32]) {
        (&self.tip_r, &self.tip_g, &self.tip_b)
    }

    /// O retângulo LOCAL que a janela de fato tocou (`None` se nada tocou) — o
    /// oráculo do gate do cap, e ele não conhece constante nenhuma.
    #[must_use]
    pub fn touched_extent_for_measure(&self) -> Option<(i32, i32, i32, i32)> {
        (self.lx0 <= self.lx1 && self.ly0 <= self.ly1)
            .then_some((self.lx0, self.ly0, self.lx1, self.ly1))
    }

    /// A cor LIMPA da janela num texel local (a Shape Color) — só para MEDIÇÃO, como a do bico
    /// abaixo; `None` sem cor na janela ou fora dela.
    #[must_use]
    pub fn tinta_rgb_for_measure(&self, lx: i32, ly: i32) -> Option<[f32; 3]> {
        if !self.tinta_na_janela || lx < 0 || ly < 0 || lx >= self.size || ly >= self.size {
            return None;
        }
        let l = (lx + ly * self.size) as usize;
        Some([self.tinta_r[l], self.tinta_g[l], self.tinta_b[l]])
    }

    /// A cor do BICO num texel local — o oráculo do gate da auto-limpeza.
    ///
    /// ⚠️ Só para MEDIÇÃO, como as duas irmãs acima: o produto nunca lê um
    /// texel do bico por coordenada (ele deposita a janela inteira). Existe
    /// porque a pergunta *"a limpeza cobre a janela VIVA?"* é sobre um plano, e
    /// o único jeito de observá-la sem um oráculo de canvas de três estágios é
    /// ler o plano. Fora da janela devolve `None` em vez de panicar — um gate
    /// que indexa errado deve falhar dizendo isso, não estourar.
    #[must_use]
    pub fn tip_rgb_for_measure(&self, lx: i32, ly: i32) -> Option<[f32; 3]> {
        if lx < 0 || ly < 0 || lx >= self.size || ly >= self.size {
            return None;
        }
        let l = (lx + ly * self.size) as usize;
        Some([self.tip_r[l], self.tip_g[l], self.tip_b[l]])
    }
}
