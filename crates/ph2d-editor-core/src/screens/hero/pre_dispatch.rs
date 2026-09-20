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
//! | a **paleta de comandos** (modal de ecrã inteiro) | os ids que a paleta MOSTRA | o painel dono daquele id consome o clique, faz o efeito, e **o modal nunca fecha** |
//!
//! ⚠️ **Um fecho escrito num handler de `chrome/` ficaria morto** precisamente nos treze ids do
//! menu *Window*, que é onde ele mais importa.
//!
//! # ⭐⭐⭐ A quarta linha: um MODAL é dono do ponteiro (report do dono, 2026-09-20)
//!
//! *«AO selecionar o pincel, o modal deveria se fechar automaticamente.»*
//!
//! ⛔ A causa **não** era a paleta: era esta ordem. Um item da paleta de pincéis carrega o
//! **MESMO `NodeId` da ficha do painel** (é isso que faz o *pick* resolver pela lei que já
//! existia), logo o painel da escultura reconhecia-o, **consumia**, trocava o pincel — e o
//! `chrome::dispatch_all`, que é quem fecha a paleta, nunca corria. *Do lado do artista: o pincel
//! muda e o modal fica aberto.*
//!
//! ⭐ É a **mesma lei** que o teclado já paga desde a manhã do mesmo dia
//! (`shells/desktop/tests/it/um_modal_aberto_tem_o_teclado_antes_da_cena_3d.rs`): *enquanto um
//! modal de ecrã inteiro está aberto, a entrada é dele.* Aqui é o ponteiro.
//!
//! ⚠️ **Com a paleta FECHADA isto é inerte por construção** — o `apply` dela devolve `false` no
//! primeiro `if` quando não há modelo —, e é por isso que o hoist não precisa de guarda própria:
//! *uma segunda pergunta «está aberta?» seria a segunda resposta à mesma coisa.*

use super::HeroScreen;
use crate::interaction::WidgetEvent;

/// Corre as duas metades do pré-despacho. Devolve `true` se o evento já está resolvido — nesse
/// caso o `apply_event` **não** deve continuar.
pub(super) fn run(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    // ⭐⭐⭐ **Um MODAL de ecrã inteiro é dono do ponteiro** — primeiro de todos, porque é ele que
    // está por cima de tudo o resto. Fechado, devolve `false` sem tocar em nada.
    if super::chrome::command_palette_pointer(hero, event) {
        return true;
    }
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
