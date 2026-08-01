//! As **guias** do documento — as linhas de referência que o artista arrasta da régua.
//!
//! # O que uma guia É, e por que isso decide o resto
//!
//! Uma guia é uma reta **alinhada a um eixo**, e portanto uma restrição **1-D**: uma guia
//! horizontal fixa o `y` e **não diz nada** sobre o `x`. Isso a coloca exatamente na espécie
//! de reivindicação que o motor de snap já tinha — o **ALINHAMENTO** por eixo, aquele que se
//! decompõe (o X vem de uma vizinha, o Y vem da guia, e o resultado significa alguma coisa:
//! são duas retas que se cruzam).
//!
//! Não é a espécie de **POSIÇÃO** que a W6.1 acrescentou (pousar SOBRE uma curva vence os dois
//! eixos ou nenhum). Confundir as duas seria fatal na direção mais óbvia: uma guia que
//! reclamasse os dois eixos prenderia o ponto num lugar arbitrário DA linha, que é precisamente
//! o que uma guia não faz.
//!
//! ⚠️ **Guia inclinada não existe aqui, e é decisão.** Uma reta oblíqua é uma restrição 1-D que
//! **não se decompõe em eixos** — encaixar nela move `x` E `y`, e o resultado é uma projeção
//! perpendicular, não um alinhamento. Ela é uma TERCEIRA espécie de reivindicação (a *linha*),
//! com gesto próprio (criar, girar) e matemática própria; enfiá-la neste tipo obrigaria o
//! `pos: f64` a virar `(origem, direção)` e o consumidor de snap a ramificar em duas leis
//! dentro de um laço que hoje tem uma. Está nomeada no handoff, não meio-construída aqui.
//!
//! # Por que uma crate própria
//!
//! Ver o `Cargo.toml`: três consumidores independentes (motor de snap, desenho, arquivo), e
//! nenhum deles é dono do fato. É o precedente da `ph2d-stroke-width`.

#![forbid(unsafe_code)]

/// A que eixo a guia é paralela.
#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GuideAxis {
    /// Uma linha HORIZONTAL: `y` constante. Arrastada da régua de cima.
    Horizontal,
    /// Uma linha VERTICAL: `x` constante. Arrastada da régua da esquerda.
    Vertical,
}

impl GuideAxis {
    /// O índice da coordenada que esta guia **prende** — o eixo que ela reivindica no snap.
    ///
    /// ⚠️ A troca é fácil de fazer ao contrário e o compilador não ajuda (os dois são `usize`):
    /// uma guia **horizontal** é uma linha de `y` constante, logo ela prende o **Y** (índice 1).
    /// O nome do eixo descreve a DIREÇÃO da linha; o índice, a coordenada CONGELADA.
    #[must_use]
    pub fn locked_axis(self) -> usize {
        match self {
            GuideAxis::Horizontal => 1,
            GuideAxis::Vertical => 0,
        }
    }
}

/// Uma guia: a reta `coordenada[locked_axis] == pos`, em **world**.
#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Guide {
    pub axis: GuideAxis,
    /// A coordenada congelada, em world-units — a mesma régua que o Inspector mostra em X/Y.
    pub pos: f64,
}

impl Guide {
    #[must_use]
    pub fn horizontal(y: f64) -> Self {
        Self {
            axis: GuideAxis::Horizontal,
            pos: y,
        }
    }

    #[must_use]
    pub fn vertical(x: f64) -> Self {
        Self {
            axis: GuideAxis::Vertical,
            pos: x,
        }
    }

    /// Distância de `p` até esta reta, em world. É `|p[eixo] − pos|` porque a reta é
    /// paralela a um eixo — a projeção perpendicular é a própria diferença de coordenada.
    #[must_use]
    pub fn distance_to(&self, p: [f64; 2]) -> f64 {
        (p[self.axis.locked_axis()] - self.pos).abs()
    }
}

/// As guias de um documento.
///
/// ⚠️ **Duplicatas são LEGAIS, e é decisão.** Soltar uma guia exatamente sobre outra poderia
/// fundir as duas — e então o artista teria arrastado da régua até o canvas e **nada** teria
/// aparecido, que é a forma exata de um gesto parecer quebrado. Duas guias empilhadas custam
/// 9 bytes e leem certo: arrastar uma revela a outra.
///
/// ⚠️ **Não há teto no número de guias, e o §0 é o motivo.** Um teto tem de dizer de que
/// recurso ele é; aqui não há nenhum — uma guia são 9 bytes serializados, e o custo de consulta
/// é `O(fontes × guias)` com as fontes limitadas pelos 9 pontos-chave da bbox mais as âncoras
/// em gesto. O número medido está em `guides_tests::the_cost_of_a_guide_is_a_comparison`.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GuideSet {
    items: Vec<Guide>,
}

impl GuideSet {
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Guide> {
        self.items.iter()
    }

    #[must_use]
    pub fn get(&self, i: usize) -> Option<Guide> {
        self.items.get(i).copied()
    }

    /// Acrescenta uma guia e devolve o índice dela.
    pub fn push(&mut self, g: Guide) -> usize {
        self.items.push(g);
        self.items.len() - 1
    }

    /// Remove a guia `i`. ⚠️ Usa `remove`, não `swap_remove`: os índices são a identidade de
    /// uma guia durante um arrasto, e trocar a ordem faria o gesto seguinte pegar outra linha.
    pub fn remove(&mut self, i: usize) -> Option<Guide> {
        (i < self.items.len()).then(|| self.items.remove(i))
    }

    /// Move a guia `i` para `pos`. No-op se o índice não existe.
    pub fn set_pos(&mut self, i: usize, pos: f64) {
        if let Some(g) = self.items.get_mut(i) {
            g.pos = pos;
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// A guia mais próxima de `p` dentro de `tol` (world), ou `None`.
    ///
    /// Empate fica com a **mais recente** (`<=` sobre a varredura em ordem), que é a que o
    /// artista acabou de soltar — a única escolha que não parece aleatória quando há duas
    /// empilhadas.
    #[must_use]
    pub fn nearest(&self, p: [f64; 2], tol: f64) -> Option<usize> {
        let mut best: Option<(f64, usize)> = None;
        for (i, g) in self.items.iter().enumerate() {
            let d = g.distance_to(p);
            if d <= tol && best.is_none_or(|(bd, _)| d <= bd) {
                best = Some((d, i));
            }
        }
        best.map(|(_, i)| i)
    }
}

#[cfg(test)]
#[path = "guides_tests.rs"]
mod tests;
