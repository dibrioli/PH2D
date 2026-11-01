//! ⭐⭐ **Qual tinta uma edição atinge** (F44b, dono 01/11: *«todas as linhas iguais»*) e **o olho do
//! chão**.
//!
//! O painel de Fill/Stroke mostra a base e as camadas extra como linhas IGUAIS: cada uma tem o
//! mesmo corpo de ajustes (tipo, degradê, padrão; pontas, junções, tracejado…). O [`PaintSlot`] é o
//! endereço dessa linha, e as portas `slot_*` respondem com a tinta dela — a base ou `paints[i]`.
//!
//! ⭐ **O olho da base GUARDA a tinta à parte** (`fill_hidden`/`stroke_hidden`, v26) e deixa o
//! `fill`/`stroke` a `None`. É por isso que esconder é exacto em TODA a leitura do documento sem
//! tocar nela: quem desenha, testa o clique, mede a caixa ou exporta já trata `None` como «não
//! pinta». Só o EDITOR lê a guardada (`base_*`): ela continua editável escondida, como no Figma.
//! ⚠️ Invariante: nunca as duas `Some` — uma rota que escreva no `fill` de uma base escondida é
//! devolvida à guarda por [`VecPath::settle_hidden`].

use super::PaintKind;
use crate::{Paint, StrokeSpec, VecPath};

/// O endereço de uma linha de Fill/Stroke: a base, ou a camada `i` da pilha (índice do documento).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaintSlot {
    #[default]
    Base,
    Layer(usize),
}

impl VecPath {
    /// O preenchimento de base, à vista ou escondido.
    #[must_use]
    pub fn base_fill(&self) -> Option<&Paint> {
        self.fill.as_ref().or(self.fill_hidden.as_ref())
    }

    pub fn base_fill_mut(&mut self) -> Option<&mut Paint> {
        if self.fill.is_some() {
            self.fill.as_mut()
        } else {
            self.fill_hidden.as_mut()
        }
    }

    /// O traço de base, à vista ou escondido.
    #[must_use]
    pub fn base_stroke(&self) -> Option<&StrokeSpec> {
        self.stroke.as_ref().or(self.stroke_hidden.as_ref())
    }

    pub fn base_stroke_mut(&mut self) -> Option<&mut StrokeSpec> {
        if self.stroke.is_some() {
            self.stroke.as_mut()
        } else {
            self.stroke_hidden.as_mut()
        }
    }

    /// O olho do preenchimento de base: `true` = escondido.
    #[must_use]
    pub fn fill_is_hidden(&self) -> bool {
        self.fill.is_none() && self.fill_hidden.is_some()
    }

    /// O olho do traço de base: `true` = escondido.
    #[must_use]
    pub fn stroke_is_hidden(&self) -> bool {
        self.stroke.is_none() && self.stroke_hidden.is_some()
    }

    /// Esconde/mostra o preenchimento de base. Devolve se mudou.
    pub fn set_fill_hidden(&mut self, hidden: bool) -> bool {
        let (from, to) = if hidden {
            (&mut self.fill, &mut self.fill_hidden)
        } else {
            (&mut self.fill_hidden, &mut self.fill)
        };
        swap_into(from, to)
    }

    /// Esconde/mostra o traço de base. Devolve se mudou.
    pub fn set_stroke_hidden(&mut self, hidden: bool) -> bool {
        let (from, to) = if hidden {
            (&mut self.stroke, &mut self.stroke_hidden)
        } else {
            (&mut self.stroke_hidden, &mut self.stroke)
        };
        swap_into(from, to)
    }

    /// O «−» da base: tira a tinta, à vista ou escondida. Devolve se havia.
    pub fn remove_base_fill(&mut self) -> bool {
        self.fill.take().is_some() | self.fill_hidden.take().is_some()
    }

    pub fn remove_base_stroke(&mut self) -> bool {
        self.stroke.take().is_some() | self.stroke_hidden.take().is_some()
    }

    /// ⚠️ **A guarda da invariante**: uma escrita no `fill`/`stroke` de uma base ESCONDIDA (uma rota
    /// que só conhece o campo à vista) é a edição da tinta escondida — volta para a guarda, e o olho
    /// continua fechado. Devolve se corrigiu alguma.
    pub fn settle_hidden(&mut self) -> bool {
        let mut mudou = false;
        if self.fill.is_some() && self.fill_hidden.is_some() {
            self.fill_hidden = self.fill.take();
            mudou = true;
        }
        if self.stroke.is_some() && self.stroke_hidden.is_some() {
            self.stroke_hidden = self.stroke.take();
            mudou = true;
        }
        mudou
    }

    /// O preenchimento da linha `s` (`None` se a linha não é um preenchimento).
    #[must_use]
    pub fn slot_fill(&self, s: PaintSlot) -> Option<&Paint> {
        match s {
            PaintSlot::Base => self.base_fill(),
            PaintSlot::Layer(i) => match &self.paints.get(i)?.kind {
                PaintKind::Fill(p) => Some(p),
                PaintKind::Stroke(_) => None,
            },
        }
    }

    pub fn slot_fill_mut(&mut self, s: PaintSlot) -> Option<&mut Paint> {
        match s {
            PaintSlot::Base => self.base_fill_mut(),
            PaintSlot::Layer(i) => match &mut self.paints.get_mut(i)?.kind {
                PaintKind::Fill(p) => Some(p),
                PaintKind::Stroke(_) => None,
            },
        }
    }

    /// O traço da linha `s` (`None` se a linha não é um traço).
    #[must_use]
    pub fn slot_stroke(&self, s: PaintSlot) -> Option<&StrokeSpec> {
        match s {
            PaintSlot::Base => self.base_stroke(),
            PaintSlot::Layer(i) => match &self.paints.get(i)?.kind {
                PaintKind::Stroke(sp) => Some(sp),
                PaintKind::Fill(_) => None,
            },
        }
    }

    pub fn slot_stroke_mut(&mut self, s: PaintSlot) -> Option<&mut StrokeSpec> {
        match s {
            PaintSlot::Base => self.base_stroke_mut(),
            PaintSlot::Layer(i) => match &mut self.paints.get_mut(i)?.kind {
                PaintKind::Stroke(sp) => Some(sp),
                PaintKind::Fill(_) => None,
            },
        }
    }
}

/// Move `from` para `to` se `from` tem tinta e `to` está vazio.
fn swap_into<T>(from: &mut Option<T>, to: &mut Option<T>) -> bool {
    if from.is_some() && to.is_none() {
        *to = from.take();
        true
    } else {
        false
    }
}

#[cfg(test)]
#[path = "paint_slot_tests.rs"]
mod tests;
