//! ⭐⭐ **O QUE CORRE ANTES DO REGISTRY DE PAINÉIS** — e por que essa ordem é load-bearing.
//!
//! O [`super::HeroScreen::apply_event`] caminha o registry de painéis **antes** do
//! `chrome::dispatch_all`. Isso significa que todo id que um painel reconhece nunca chega ao
//! chrome — e há duas famílias de controlo cujos ids são exactamente esses:
//!
//! | quem | ids | o que aconteceria sem este pré-despacho |
//! |---|---|---|
//! | as linhas do menu *Window* | `TOPBAR_AUDIO_MIXER`, … | o painel consome o clique e **o menu nunca fecha** |
//! | as **abas** de um encaixe | derivados de `Panel::NODE_ID` | o clique cai no painel de baixo em vez de o levantar |
//! | as **abas de LAYOUT** | derivados de `TaskLayout` | o clique cai no painel por baixo da barra |
//!
//! ⚠️ **Um fecho escrito num handler de `chrome/` ficaria morto** precisamente nos treze ids do
//! menu *Window*, que é onde ele mais importa.

use super::HeroScreen;
use crate::interaction::WidgetEvent;

/// Corre as duas metades do pré-despacho. Devolve `true` se o evento já está resolvido — nesse
/// caso o `apply_event` **não** deve continuar.
pub(super) fn run(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    // Uma linha da barra de menus fecha o menu (mas não consome: quem age é o dono do id).
    super::menu_bar::close_on_row_click(hero, event);
    // ⭐ Uma aba de LAYOUT arruma a tela para a tarefa (D7). Antes da aba de painel: os dois ids
    // são derivados e nenhum handler de chrome os alcança.
    if super::layout_tabs::apply_event(hero, event) {
        return true;
    }
    // Uma aba de painel levanta o painel dela, e isso é tudo o que uma aba faz.
    super::slot_tabs::apply_event(hero, event)
}
