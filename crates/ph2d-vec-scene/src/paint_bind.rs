//! **A TINTA RESOLVIDA** — o que uma forma bindada a um token desenha NESTE modo.
//!
//! É a costura *fonte autorada ≠ o que o mundo consome* do ADR-0121, agora na TINTA em vez da
//! geometria: o documento guarda o literal, e o que se desenha é derivado. Daí a forma da resposta
//! ser a MESMA — [`crate::VecPath::painted`] devolve um [`Cow`], `Borrowed` quando não há binding,
//! e foi isso que permitiu ligá-la no ponto único de desenho sem mudar um byte do que já existe.
//!
//! # Quem resolve, e por quê não é aqui
//!
//! Esta crate é o MODELO DE DOCUMENTO: ela não conhece `ph2d-tokens`, não conhece tema, e não
//! conhece o ECS onde os bindings moram. Quem resolve é a shell, uma vez por frame, e publica o
//! resultado no [`crate::VecViewState`] — exatamente como já publica *quem está escondido* e *quem
//! recorta*. O renderer consome tinta pronta.
//!
//! ⚠️ **Um número, três consumidores, UMA porta:** a shell (para desenhar), o painel (para a
//! swatch mostrar a cor do token) e o gate perguntam todos à `ph2d_tokens::ColorToken::from_key` +
//! `resolve(theme)`. Uma segunda tabela em qualquer um deles é a swatch que mostra uma cor e a arte
//! que desenha outra — divergência que só aparece num screenshot.

use crate::{Paint, Rgba8, VecPath, VecPathId};

/// A tinta que os tokens desta forma produzem no modo vigente.
///
/// `None` num campo = aquela propriedade **não** está bindada e o literal do documento vale.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BoundPaint {
    /// A forma a que isto se refere.
    pub path: VecPathId,
    /// Cor do preenchimento vinda de um token.
    pub fill: Option<Rgba8>,
    /// Cor do traço vinda de um token.
    pub stroke: Option<Rgba8>,
}

impl BoundPaint {
    /// Nada resolvido ⇒ esta entrada não muda desenho nenhum.
    #[must_use]
    pub fn is_noop(&self) -> bool {
        self.fill.is_none() && self.stroke.is_none()
    }
}

impl VecPath {
    /// **A forma como ela DESENHA neste modo** — o literal, ou o token que o cobre.
    ///
    /// ⚠️ Sem binding devolve [`Cow::Borrowed`]: o mesmo ponteiro, zero cópia, e o desenho é
    /// byte-idêntico ao mundo pré-token. É a propriedade que torna seguro chamar isto no caminho
    /// quente de TODA forma da cena, e é a mesma que o [`VecPath::cooked`](crate::VecPath) usa
    /// para o raio de quina vivo.
    ///
    /// ⚠️ **O preenchimento e o traço não se comportam igual, e o motivo é geometria e não
    /// gosto:** um `Paint::Solid` descreve um preenchimento por INTEIRO, então bindar o
    /// preenchimento de uma forma sem preenchimento é autoria completa e ele nasce. Um traço
    /// precisa também de LARGURA — colori-lo numa forma sem traço obrigaria a inventar um número
    /// que o artista não escreveu —, então o token do traço **pinta o traço que existe** e não
    /// cria nenhum. Bindar sem ver mudança nenhuma seria pior do que a row não estar lá; é por
    /// isso que o painel só oferece a row do traço quando há traço.
    #[must_use]
    pub fn painted<'a>(&'a self, bound: Option<&BoundPaint>) -> std::borrow::Cow<'a, VecPath> {
        let Some(b) = bound.filter(|b| !b.is_noop()) else {
            return std::borrow::Cow::Borrowed(self);
        };
        // Um traço bindado numa forma SEM traço não tem o que colorir — e se ele fosse o único
        // binding, clonar aqui seria uma cópia que não muda um pixel.
        let paints_stroke = b.stroke.is_some() && self.stroke.is_some();
        if b.fill.is_none() && !paints_stroke {
            return std::borrow::Cow::Borrowed(self);
        }
        let mut out = self.clone();
        if let Some(c) = b.fill {
            out.fill = Some(Paint::Solid(c));
        }
        if let (Some(c), Some(s)) = (b.stroke, out.stroke.as_mut()) {
            s.color = c;
        }
        std::borrow::Cow::Owned(out)
    }
}

#[cfg(test)]
#[path = "paint_bind_tests.rs"]
mod tests;
