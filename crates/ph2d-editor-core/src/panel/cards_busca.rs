//! ⭐ **O campo de BUSCA de um cartão** ([`Widget::Search`]): o campo de texto com o «×» que o
//! esvazia, dentro da caixa, à direita, enquanto há texto (smoke do dono 08/11: *«Falta um x para
//! limpar a busca de atalhos»*). O clique esvazia e o campo continua com o teclado.
//!
//! O painel regista os dois no `populate` ([`regista`]) e passa o clique a [`limpou`] no
//! `apply_event` — o mesmo par do «×» das janelas (`borda::declara_fecho` · `borda::fechou`).

use super::*;
use crate::interaction::InteractiveState;
use crate::widget::{ButtonState, TextInputState};

/// O id do «×» do campo `campo` (derivado dele: um campo, um «×»).
#[must_use]
pub fn limpa_de(campo: NodeId) -> NodeId {
    const SAL_DA_LIMPEZA: u64 = 0x5E4C_A7B1_93D0_26F8;
    NodeId(campo.0 ^ SAL_DA_LIMPEZA)
}

/// Regista o campo `campo` e o «×» dele (no `populate` do painel).
pub fn regista(store: &mut WidgetStore, campo: NodeId) {
    store.register(
        campo,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    let x = limpa_de(campo);
    store.register(
        x,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    store.set_tooltip(x, ph2d_i18n::tr("card.search.clear"));
}

/// O clique no «×» do campo `campo` esvazia-o e devolve-lhe o teclado. `true` se era ele.
pub fn limpou(
    store: &mut WidgetStore,
    ev: &crate::interaction::WidgetEvent,
    campo: NodeId,
) -> bool {
    let hit = matches!(ev, crate::interaction::WidgetEvent::Click(c) if *c == limpa_de(campo));
    if !hit {
        return false;
    }
    if let Some(InteractiveState::TextInput {
        state,
        text,
        caret,
        selection_anchor,
    }) = store.get_mut(campo)
    {
        text.clear();
        *caret = 0;
        *selection_anchor = None;
        *state = TextInputState::Focused;
        store.set_focus(Some(campo));
    }
    true
}

impl CardCtx<'_> {
    /// O campo de texto e, com texto, o «×» por cima da ponta direita (registado depois: ganha).
    pub(super) fn search(&mut self, r: Rect, id: NodeId, placeholder: &str) {
        self.text(r, id, placeholder);
        if self.store.text(id).is_none_or(str::is_empty) {
            return;
        }
        let x = limpa_de(id);
        let lado = r.h.min(m::FIELD_H);
        let ponta = Rect::new(r.x + r.w - lado, r.y + (r.h - lado) * 0.5, lado, lado);
        self.hit_index.register(x, ponta);
        let fundo = if self.store.hover_live(x) > 0.0 {
            ColorToken::CardFieldHover
        } else {
            ColorToken::CardField
        };
        fill_rounded_rect(self.scene, ponta, m::FIELD_RADIUS, self.ink.color(fundo));
        let g = m::HEAD_ICON_GLYPH;
        paint_icon(
            self.scene,
            IconId::Close,
            Rect::new(
                ponta.x + (ponta.w - g) * 0.5,
                ponta.y + (ponta.h - g) * 0.5,
                g,
                g,
            ),
            self.ink.color(ColorToken::Text2),
            StrokeToken::Default.px(),
        );
    }
}
