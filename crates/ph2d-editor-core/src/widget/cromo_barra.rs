//! As barras do cromo no idioma dos cartões (`docs/CartoesUI/15_CHROME_CENSO.md`, M3 a M7): o chão no
//! tom da lista, as abas como segmentos, os chips planos e os painéis flutuantes com a sombra da lista.
//! `None` em [`Barra::de`] = `PH2D_CARDS=0`: cada pintor segue o caminho de antes, ao bit.

use crate::paint::fill_rounded_rect;
use crate::widget::card_ink::CardInk;
use crate::widget::lista_flutuante::cromo_em_cartoes;
use crate::zones::Rect;
use ph2d_card_stack::metrics as m;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Color, VectorScene};

/// A tinta de uma barra do cromo, decidida uma vez por pintor.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Barra {
    ink: CardInk,
}

impl Barra {
    /// A barra no modo vigente; `None` no modo de antes.
    #[must_use]
    pub fn de(theme: Theme) -> Option<Self> {
        cromo_em_cartoes().then(|| Self {
            ink: CardInk::of_list(theme),
        })
    }

    /// A tinta de cartão desta barra, para os widgets partilhados pintados sobre ela.
    #[must_use]
    pub const fn tinta(self) -> CardInk {
        self.ink
    }

    /// O chão: o tom da lista de cartões.
    #[must_use]
    pub fn chao(self) -> Color {
        self.ink.color(ColorToken::KanbanList)
    }

    /// O rótulo solto de um chip.
    #[must_use]
    pub fn rotulo(self) -> Color {
        self.ink.color(ColorToken::CardLabel)
    }

    /// O texto de um campo ou de um nome.
    #[must_use]
    pub fn texto(self) -> Color {
        self.ink.color(ColorToken::KanbanCardFg)
    }

    /// A linha que separa grupos.
    #[must_use]
    pub fn divisoria(self) -> Color {
        self.ink.color(ColorToken::CardFieldHover)
    }

    /// O realce de um chip escolhido: o tom da borda de foco e do glifo.
    #[must_use]
    pub fn acento(self) -> Color {
        self.ink.color(ColorToken::KanbanBlueBold)
    }

    /// A pílula que leva os segmentos de uma fila de abas (`r` é a união das abas).
    pub fn pilula(self, scene: &mut VectorScene, r: Rect) {
        let cor = self.ink.color(ColorToken::CardField);
        fill_rounded_rect(scene, encolhe_y(r, m::FIELD_BORDER), m::FIELD_RADIUS, cor);
    }

    /// Um segmento da pílula, com o fundo que o estado pede; devolve a cor do texto.
    pub fn segmento(self, scene: &mut VectorScene, r: Rect, on: bool, hover: bool) -> Color {
        let miolo = encolhe_y(r, m::FIELD_BORDER + m::SEG_PAD);
        if let Some(fundo) = fundo_de_estado(on, hover) {
            fill_rounded_rect(scene, miolo, m::SEG_RADIUS, self.ink.color(fundo));
        }
        self.ink.color(if on {
            ColorToken::Text1
        } else {
            ColorToken::Text2
        })
    }

    /// Um chip sem caixa em repouso (ferramenta, botão de módulo); devolve a cor do glifo.
    pub fn chip(self, scene: &mut VectorScene, r: Rect, on: bool, hover: bool) -> Color {
        if let Some(fundo) = fundo_de_estado(on, hover) {
            fill_rounded_rect(scene, r, m::FIELD_RADIUS, self.ink.color(fundo));
        }
        if on {
            self.acento()
        } else if hover {
            self.ink.color(ColorToken::Text1)
        } else {
            self.ink.color(ColorToken::Text2)
        }
    }

    /// Uma caixa de campo do cartão (número, opção); o escolhido sobe um tom.
    pub fn campo(self, scene: &mut VectorScene, r: Rect, on: bool, hover: bool) {
        let fundo = fundo_de_estado(on, hover).unwrap_or(ColorToken::CardField);
        fill_rounded_rect(scene, r, m::FIELD_RADIUS, self.ink.color(fundo));
    }

    /// O fundo de uma barra que flutua sobre o canvas: a sombra e o tom da lista, sem filete.
    pub fn painel(self, scene: &mut VectorScene, r: Rect) {
        crate::paint_flight::shadow(
            scene,
            r,
            m::FIELD_RADIUS,
            self.ink.color(ColorToken::KanbanListShadow),
            m::LIST_SHADOW_NEAR,
        );
        fill_rounded_rect(scene, r, m::FIELD_RADIUS, self.chao().with_alpha(1.0));
    }
}

const fn fundo_de_estado(on: bool, hover: bool) -> Option<ColorToken> {
    if on {
        Some(ColorToken::KanbanCardRaised)
    } else if hover {
        Some(ColorToken::CardFieldHover)
    } else {
        None
    }
}

fn encolhe_y(r: Rect, d: f32) -> Rect {
    Rect::new(r.x, r.y + d, r.w, (r.h - 2.0 * d).max(0.0))
}

/// A união de rects de uma fila, para a pílula dela.
#[must_use]
pub fn uniao(rects: impl IntoIterator<Item = Rect>) -> Option<Rect> {
    rects.into_iter().reduce(|a, b| {
        let (x, y) = (a.x.min(b.x), a.y.min(b.y));
        let (fx, fy) = ((a.x + a.w).max(b.x + b.w), (a.y + a.h).max(b.y + b.h));
        Rect::new(x, y, fx - x, fy - y)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::lista_flutuante::force_cromo_on_this_thread;

    #[test]
    fn a_barra_so_existe_em_cartoes() {
        force_cromo_on_this_thread(Some(false));
        assert!(Barra::de(Theme::Dark).is_none());
        force_cromo_on_this_thread(Some(true));
        assert!(Barra::de(Theme::Dark).is_some());
        force_cromo_on_this_thread(None);
    }

    #[test]
    fn a_uniao_abraca_a_fila() {
        let u = uniao([
            Rect::new(10.0, 2.0, 5.0, 4.0),
            Rect::new(20.0, 2.0, 5.0, 4.0),
        ]);
        assert_eq!(u, Some(Rect::new(10.0, 2.0, 15.0, 4.0)));
        assert_eq!(uniao(std::iter::empty::<Rect>()), None);
    }
}
