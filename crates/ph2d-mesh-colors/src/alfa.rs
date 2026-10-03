//! ⭐⭐⭐ **A OPACIDADE de cada amostra** — o canal que faz de um plano uma
//! CAMADA (`docs/3D/30` §11, a W2 das camadas do Painter na peça).
//!
//! ⚠️ **Com o canal presente, as cores do plano são PRÉ-MULTIPLICADAS** pela
//! opacidade (a cor direita × alfa, na unidade do plano: o byte sRGB sobre
//! `255`). É o que deixa toda mistura «por cima» desta casa — `antes·(1 − w) +
//! alvo·w` — valer igual para uma camada: a mesma conta na cor
//! pré-multiplicada, e na opacidade com alvo `1` (ou o alfa do que pousa).
//! Sobre um plano opaco (`alfa = 1`) a cor pré-multiplicada É a direita, e as
//! leis de antes não mudam um bit.
//!
//! ⚠️ **`None` = opaco em toda a parte** — o plano da peça (o composto) nunca
//! tem o canal; só a cópia de trabalho da camada activa durante um traço.

use crate::Tinta;

impl Tinta {
    /// O plano tem o canal de opacidade (é uma camada)?
    #[must_use]
    pub fn tem_alfa(&self) -> bool {
        self.alfa.is_some()
    }

    /// A opacidade de todas as amostras, ou `None` (opaco).
    #[must_use]
    pub fn alfa(&self) -> Option<&[f32]> {
        self.alfa.as_deref()
    }

    /// A opacidade de uma amostra — `1` num plano sem o canal.
    #[must_use]
    pub fn opacidade(&self, idx: usize) -> f32 {
        self.alfa.as_ref().map_or(1.0, |a| a[idx])
    }

    /// Escreve a opacidade de uma amostra. ⚠️ Num plano SEM o canal só aceita
    /// `1` (devolve `false` para outro valor): um plano opaco que ganhasse
    /// transparência por um escritor distraído mudaria de significado — as
    /// cores dele não são pré-multiplicadas.
    pub fn define_opacidade(&mut self, idx: usize, a: f32) -> bool {
        match self.alfa.as_mut() {
            Some(v) => {
                v[idx] = a;
                true
            }
            None => a == 1.0,
        }
    }

    /// Põe (ou tira) o canal — recusado (`false`) com a contagem errada.
    pub fn com_alfa(&mut self, alfa: Option<Vec<f32>>) -> bool {
        if alfa
            .as_ref()
            .is_some_and(|a| a.len() != self.amostras().len())
        {
            return false;
        }
        self.alfa = alfa;
        true
    }

    /// Tira o canal, devolvendo-o.
    pub fn tira_alfa(&mut self) -> Option<Vec<f32>> {
        self.alfa.take()
    }

    /// A opacidade num ponto de uma face triangular — pelos MESMOS pesos da
    /// [`Self::cor_tri`] (a cor pré-multiplicada e a opacidade interpolam
    /// juntas, e só então a cor se divide).
    #[must_use]
    pub fn opacidade_tri(&self, face: usize, cantos: &[u32], bar: [f32; 3]) -> f32 {
        let Some(a) = self.alfa.as_deref() else {
            return 1.0;
        };
        self.pesos_tri(face, cantos, bar)
            .map(|(i, w)| a[i] * w)
            .sum()
    }

    /// A opacidade num ponto de um quad — ver [`Self::opacidade_tri`].
    #[must_use]
    pub fn opacidade_quad(&self, face: usize, cantos: &[u32], uv: [f32; 2]) -> f32 {
        let Some(a) = self.alfa.as_deref() else {
            return 1.0;
        };
        self.pesos_quad(face, cantos, uv)
            .map(|(i, w)| a[i] * w)
            .sum()
    }
}

#[cfg(test)]
#[path = "alfa_tests.rs"]
mod tests;
