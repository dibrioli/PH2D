// ph2d-chrome-sync:z=175 (dispatch priority, ADR-0107; lower = earlier)
//! **O modal de resolução da folha** (Enio 2026-08-19): os cliques nas resoluções guardam a
//! escolha; o "Create Sheet" arma o pedido e fecha o modal. A shell sonda `take_sheet_size_request`
//! e cria a folha com as peças que reservou quando abriu.
//!
//! ⚠️ Irmão do [`super::new_image`], e a `z` fica logo a seguir à dele (170 → 175): as duas caixas
//! de diálogo não partilham id nenhum, então a ordem entre elas não é semântica — o que importa é
//! ficarem ambas ANTES do despacho genérico de linhas de menu, que consumiria o clique.

use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::CTX_MENU_SHEET_SIZE_CREATE {
        hero.store.request_sheet();
        return true;
    }
    if let Some(&(px, _)) = ids::CTX_MENU_SHEET_SIZES.iter().find(|&&(_, b)| b == id) {
        hero.store.set_sheet_size(px);
        return true;
    }
    false
}
