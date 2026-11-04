//! A superfície do que flutua no cromo: menu, lista aberta e dica (`docs/CartoesUI/15_CHROME_CENSO.md`,
//! M1). É a da lista suspensa de um cartão: sombra de lista, sem filete, um degrau acima do cartão.
//! `PH2D_CARDS=0` devolve a moldura de antes, ao bit.

use crate::paint::{fill_rounded_rect, resolve};
use crate::widget::card_ink::CardInk;
use crate::zones::Rect;
use ph2d_card_stack::metrics as m;
use ph2d_tokens::{ColorToken, Radius, Theme};
use ph2d_vector::{Color, VectorScene};

thread_local! {
    static FORCADO: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// O cromo pinta-se no idioma dos cartões? `PH2D_CARDS=0` = o de antes.
#[must_use]
pub fn cromo_em_cartoes() -> bool {
    FORCADO
        .with(std::cell::Cell::get)
        .unwrap_or_else(|| std::env::var("PH2D_CARDS").as_deref() != Ok("0"))
}

/// Força o modo NESTA thread: os testes de um processo correm em paralelo. `None` volta ao ambiente.
#[doc(hidden)]
pub fn force_cromo_on_this_thread(on: Option<bool>) {
    FORCADO.with(|f| f.set(on));
}

/// O fundo de uma lista aberta na tinta `ink`: a sombra da lista, sem moldura.
pub(crate) fn moldura_de_cartao(scene: &mut VectorScene, r: Rect, ink: CardInk) {
    crate::paint_flight::shadow(
        scene,
        r,
        m::FIELD_RADIUS,
        ink.color(ColorToken::KanbanListShadow),
        m::LIST_SHADOW_FAR,
    );
    fill_rounded_rect(
        scene,
        r,
        m::FIELD_RADIUS,
        ink.color(ColorToken::KanbanCardRaised).with_alpha(1.0),
    );
}

/// O tom em que uma peça flutuante do cromo se pinta, decidido uma vez por pintor.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Superficie {
    theme: Theme,
    cartao: bool,
}

impl Superficie {
    /// A superfície no modo vigente.
    #[must_use]
    pub fn de(theme: Theme) -> Self {
        Self {
            theme,
            cartao: cromo_em_cartoes(),
        }
    }

    /// A tinta de cartão desta superfície, para os widgets partilhados pintados sobre ela
    /// (`card_ink::com`); `None` no modo de antes.
    #[must_use]
    pub fn tinta(self) -> Option<CardInk> {
        self.cartao.then(|| CardInk::of_list(self.theme))
    }

    /// O fundo de uma lista flutuante.
    pub fn moldura(self, scene: &mut VectorScene, r: Rect) {
        if let Some(ink) = self.tinta() {
            return moldura_de_cartao(scene, r, ink);
        }
        self.moldura_de_antes(scene, r, Radius::Md, ColorToken::BgElev);
    }

    /// O fundo de uma dica.
    pub fn moldura_da_dica(self, scene: &mut VectorScene, r: Rect) {
        if let Some(ink) = self.tinta() {
            return moldura_de_cartao(scene, r, ink);
        }
        self.moldura_de_antes(scene, r, Radius::Sm, ColorToken::Bg3);
    }

    fn moldura_de_antes(self, scene: &mut VectorScene, r: Rect, raio: Radius, fundo: ColorToken) {
        let radius = crate::paint::frame_radius(self.theme, raio.px());
        fill_rounded_rect(scene, r, radius, resolve(fundo, self.theme));
        crate::paint::stroke_frame(
            scene,
            r,
            radius,
            self.theme,
            ph2d_tokens::visuals::Feel::Rest,
            1.0,
            resolve(ColorToken::Border, self.theme),
        );
    }

    /// O realce da linha sob o rato.
    pub fn realce(self, scene: &mut VectorScene, r: Rect) {
        if let Some(ink) = self.tinta() {
            let cor = ink.color(ColorToken::CardFieldHover);
            return fill_rounded_rect(scene, r, m::SEG_RADIUS, cor);
        }
        let radius = crate::paint::frame_radius(self.theme, Radius::Sm.px());
        fill_rounded_rect(scene, r, radius, resolve(ColorToken::Bg2, self.theme));
    }

    /// A cor de `token` sobre esta superfície: toda linha que age lê-se em `KanbanCardFg`, a apagada em
    /// `CardLabel`. No modo de antes, o token.
    #[must_use]
    pub fn cor(self, token: ColorToken) -> Color {
        let Some(ink) = self.tinta() else {
            return resolve(token, self.theme);
        };
        ink.color(match token {
            ColorToken::Text1 | ColorToken::Text2 => ColorToken::KanbanCardFg,
            ColorToken::Text3 => ColorToken::CardLabel,
            ColorToken::Accent => ColorToken::KanbanBlueBold,
            ColorToken::Border => ColorToken::CardField,
            other => other,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn caminhos(pinta: impl Fn(Superficie, &mut VectorScene)) -> (u32, u32) {
        let contar = |on| {
            force_cromo_on_this_thread(Some(on));
            let mut scene = VectorScene::new();
            pinta(Superficie::de(Theme::Forge), &mut scene);
            force_cromo_on_this_thread(None);
            scene.inner().encoding().n_paths
        };
        (contar(true), contar(false))
    }

    #[test]
    fn a_moldura_de_cartao_nao_traca_filete_e_a_de_antes_traca() {
        let r = Rect::new(0.0, 0.0, 200.0, 100.0);
        // Cartão: sombra + fundo. Antes, num tema clássico: fundo + filete.
        assert_eq!(caminhos(|s, c| s.moldura(c, r)), (2, 2));
        assert_eq!(caminhos(|s, c| s.moldura_da_dica(c, r)), (2, 2));
    }

    #[test]
    fn os_tons_da_superficie_trocam_com_o_modo() {
        force_cromo_on_this_thread(Some(true));
        let s = Superficie::de(Theme::Dark);
        assert_eq!(
            s.cor(ColorToken::Text2),
            resolve(ColorToken::KanbanCardFg, Theme::Dark)
        );
        assert_eq!(
            s.cor(ColorToken::Text3),
            resolve(ColorToken::CardLabel, Theme::Dark)
        );
        force_cromo_on_this_thread(Some(false));
        let s = Superficie::de(Theme::Dark);
        assert_eq!(
            s.cor(ColorToken::Text2),
            resolve(ColorToken::Text2, Theme::Dark)
        );
        force_cromo_on_this_thread(None);
    }
}
