//! ⭐ **«Alt+Enter: show me where it lives»** (versão C, dono 12/11): num resultado da busca, em vez
//! de executar, abre a cascata DE VERDADE até à linha do comando e acende-a — a busca ensina onde
//! o comando mora. Um nível por quadro: cada linha-cascata só se clica depois de pintada (a
//! cascata abre ao lado da linha, e o rectângulo dela só existe no quadro seguinte), como faria o rato.
//!
//! O que não mora nos menus (uma ferramenta, um objeto, um campo) pisca onde está.

use super::HeroScreen;
use crate::ids;
use crate::interaction::{ContextMenuKind, WidgetEvent, WidgetStore};
use ph2d_a11y::NodeId;

/// O submenu que a linha `id` abre, para o caminho (as cascatas da barra, Preferences e Theme).
fn submenu_of(id: NodeId) -> Option<ContextMenuKind> {
    super::menu_bar::cascade_of(id).or(if id == ids::MENUBAR_EDIT_PREFERENCES {
        Some(ContextMenuKind::SettingsMenu)
    } else if id == ids::MENUBAR_VIEW_THEME {
        Some(ContextMenuKind::ThemeSelector)
    } else {
        None
    })
}

/// As linhas que `kind` mostra (a tabela que vale e as linhas com texto do store).
fn rows_of(store: &WidgetStore, kind: ContextMenuKind) -> Vec<NodeId> {
    super::menu_rows::menu_rows(kind)
        .iter()
        .map(|(id, ..)| *id)
        .filter(|id| !store.menu_row_is_hidden(kind, *id))
        .chain(store.doc_menu_rows(kind).map(|(id, _)| id))
        .collect()
}

/// **O caminho até `id`** pelos menus da barra: as linhas-cascata a abrir, da raiz para baixo
/// (`MENUBAR_FILE`, `MENUBAR_FILE_EXPORT_MENU`…). `None` = não mora num menu.
#[must_use]
pub fn path_to(store: &WidgetStore, id: NodeId) -> Option<Vec<NodeId>> {
    fn walk(
        store: &WidgetStore,
        kind: ContextMenuKind,
        id: NodeId,
        trail: &mut Vec<NodeId>,
    ) -> bool {
        for row in rows_of(store, kind) {
            if row == id {
                return true;
            }
            if let Some(sub) = submenu_of(row)
                && !trail.contains(&row)
            {
                trail.push(row);
                if walk(store, sub, id, trail) {
                    return true;
                }
                trail.pop();
            }
        }
        false
    }
    let mut trail = Vec::new();
    walk(store, ContextMenuKind::MenuBarRoot, id, &mut trail).then_some(trail)
}

/// ⭐ Mostra onde `id` mora: abre o «Menu» e planeia o caminho; fora dos menus, pisca onde está.
pub fn show(hero: &mut HeroScreen, id: NodeId) {
    hero.store.close_command_palette();
    match path_to(&hero.store, id) {
        Some(path) => {
            hero.store.close_context_menu();
            super::menu_cascade::toggle_root(hero);
            hero.store.plan_menu_reveal(path, id);
        }
        None => hero.store.flash_anywhere(id),
    }
}

/// Um passo do caminho (uma vez por quadro, no tique): clica a próxima linha-cascata se ela já
/// está pintada; no fim, acende a linha do comando. O menu fechado por outra porta desfaz o plano.
pub fn tick(hero: &mut HeroScreen) {
    let Some((path, row)) = hero.store.menu_reveal_mut().take() else {
        return;
    };
    if hero.store.menu_chain().is_empty() {
        return;
    }
    let Some((&next, rest)) = path.split_first() else {
        if hero.hit_index.rect_for(row).is_some() {
            hero.store.set_hot(Some(row));
            hero.store.flash_anywhere(row);
        } else {
            hero.store.plan_menu_reveal(Vec::new(), row);
        }
        return;
    };
    if hero.hit_index.rect_for(next).is_none() {
        hero.store.plan_menu_reveal(path, row);
        return;
    }
    hero.apply_event(WidgetEvent::Click(next));
    hero.store.set_hot(Some(next));
    hero.store.plan_menu_reveal(rest.to_vec(), row);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn o_caminho_do_export_svg_passa_pelo_file_e_pelo_export() {
        let hero = HeroScreen::new(NodeId(1));
        assert_eq!(
            path_to(&hero.store, ids::CTX_MENU_EXPORT_SVG),
            Some(vec![ids::MENUBAR_FILE, ids::MENUBAR_FILE_EXPORT_MENU])
        );
        assert_eq!(
            path_to(&hero.store, ids::CTX_MENU_SAVE),
            Some(vec![ids::MENUBAR_FILE])
        );
        assert_eq!(path_to(&hero.store, NodeId(0xDEAD)), None);
    }
}
