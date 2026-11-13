//! **Os comandos da BARRA DE MENUS na paleta, com o caminho de onde moram** (onda 2, dono 12/11).
//!
//! Tudo DERIVADO das tabelas da barra: `menu_bar::MENUS` (os cinco títulos), `menu_bar::cascade_of`
//! (linha → submenu) e `menu_rows::menu_rows` (as linhas). O nome de um submenu é o rótulo da linha
//! que o abre — nenhuma tabela nova.

use super::super::{menu_bar, menu_rows};
use crate::ids::MenuRow;
use crate::interaction::ContextMenuKind;
use crate::widget::command_palette::{PATH_SEP, PaletteGroup, PaletteItem, PaletteSub};
use ph2d_tokens::ColorToken;

/// O rótulo de uma linha-cascata lido como NOME de lugar: sem o «›», as reticências e o recuo.
#[must_use]
pub fn place_name(label: &str) -> String {
    let label = label.strip_prefix("\u{2014} ").unwrap_or(label);
    label
        .trim_end_matches(['\u{203a}', '\u{2026}', '.', ' '])
        .to_owned()
}

/// **As folhas de um menu da barra, cada uma com o caminho dos submenus até ela** (`""` = mora no
/// próprio menu; `"Export"`; `"A · B"` para uma cascata dentro de outra). As linhas-cascata não
/// são comandos e ficam de fora — a mesma travessia de `menu_bar::leaf_rows`.
#[must_use]
pub fn leaf_rows_with_path(kind: ContextMenuKind) -> Vec<(MenuRow, String)> {
    fn walk(kind: ContextMenuKind, trail: &str, out: &mut Vec<(MenuRow, String)>) {
        for &row in menu_rows::menu_rows(kind) {
            match menu_bar::cascade_of(row.0) {
                Some(sub) => {
                    let name = place_name(row.1.tr());
                    let deeper = if trail.is_empty() {
                        name
                    } else {
                        format!("{trail}{PATH_SEP}{name}")
                    };
                    walk(sub, &deeper, out);
                }
                None => out.push((row, trail.to_owned())),
            }
        }
    }
    let mut out = Vec::new();
    walk(kind, "", &mut out);
    out
}

/// A linha só ABRE uma lista ancorada a ela (Preferences…, Theme…): não é comando — as folhas
/// dessas listas entram por `menu_rows::TOPBAR_LEAF_MENUS`. `Scenes…` fica: abre a lista de cenas.
#[must_use]
pub fn opens_a_list(id: ph2d_a11y::NodeId) -> bool {
    id != crate::ids::MENUBAR_FILE_SCENES && super::super::menu_cascade::opens_submenu(id)
}

/// Onde um comando mora na barra: o título do menu e o caminho dos submenus abaixo dele.
pub type BarPlace = (String, Option<String>);

/// O lugar do menu `kind` na barra (`MenuBarFileExport` ⇒ `File`, `Export`), pelas cascatas.
#[must_use]
pub fn place_of_menu(kind: ContextMenuKind) -> Option<BarPlace> {
    fn find(at: ContextMenuKind, want: ContextMenuKind, trail: &str) -> Option<String> {
        menu_rows::menu_rows(at).iter().find_map(|row| {
            let sub = menu_bar::cascade_of(row.0)?;
            let name = place_name(row.1.tr());
            let deeper = if trail.is_empty() {
                name
            } else {
                format!("{trail}{PATH_SEP}{name}")
            };
            if sub == want {
                Some(deeper)
            } else {
                find(sub, want, &deeper)
            }
        })
    }
    menu_bar::MENUS.iter().find_map(|(_, title, top)| {
        let title = title.tr().to_owned();
        if *top == kind {
            Some((title, None))
        } else {
            find(*top, kind, "").map(|p| (title, Some(p)))
        }
    })
}

/// O nome da linha `id` do menu `kind` da barra, lido como lugar.
fn row_place(kind: ContextMenuKind, id: ph2d_a11y::NodeId) -> Option<String> {
    menu_rows::menu_rows(kind)
        .iter()
        .find(|r| r.0 == id)
        .map(|r| place_name(r.1.tr()))
}

/// O lugar das listas que uma linha da barra abre (`menu_rows::TOPBAR_LEAF_MENUS`): o tema em
/// `View · Theme`, cada preferência em `Edit · Preferences · <nome>`; o resto em `File`.
#[must_use]
pub fn place_of_leaf_menu(kind: ContextMenuKind) -> BarPlace {
    let top = |k: ContextMenuKind| place_of_menu(k).map_or_else(String::new, |p| p.0);
    let name = menu_rows::menu_title(kind).map(str::to_owned);
    if kind == ContextMenuKind::ThemeSelector {
        let theme = row_place(ContextMenuKind::MenuBarView, crate::ids::MENUBAR_VIEW_THEME);
        return (top(ContextMenuKind::MenuBarView), theme);
    }
    // Os antigos menus do pill File e Open moram em File; os outros são as listas das Preferências.
    if !matches!(kind, ContextMenuKind::SaveMenu | ContextMenuKind::OpenMenu) {
        let pref = row_place(
            ContextMenuKind::MenuBarEdit,
            crate::ids::MENUBAR_EDIT_PREFERENCES,
        )
        .unwrap_or_default();
        let path = match name {
            Some(n) => format!("{pref}{PATH_SEP}{n}"),
            None => pref,
        };
        return (top(ContextMenuKind::MenuBarEdit), Some(path));
    }
    let file = top(ContextMenuKind::MenuBarFile);
    (file.clone(), name.filter(|n| *n != file))
}

/// Põe `item` no lugar `place` dos grupos da barra — uma vez só por id (o primeiro lugar ganha).
pub fn place(groups: &mut Vec<PaletteGroup>, (top, sub): BarPlace, item: PaletteItem) {
    if groups
        .iter()
        .flat_map(|g| &g.subs)
        .flat_map(|s| &s.items)
        .any(|it| it.id == item.id)
    {
        return;
    }
    let g = match groups.iter().position(|g| g.title == top) {
        Some(i) => &mut groups[i],
        None => {
            groups.push(PaletteGroup {
                title: top,
                color: ColorToken::NodeCatOutput,
                subs: Vec::new(),
            });
            groups.last_mut().expect("acabou de entrar")
        }
    };
    match g.subs.iter_mut().find(|s| s.title == sub) {
        Some(s) => s.items.push(item),
        None => g.subs.push(PaletteSub {
            title: sub,
            items: vec![item],
        }),
    }
}

/// Um grupo por menu da barra (File, Edit, View, Window, Help), uma fatia por submenu. ⚠️ A linha
/// que abre ESTA paleta fica de fora: escolhê-la daqui só a reabria.
#[must_use]
pub fn groups() -> Vec<PaletteGroup> {
    let mut out = Vec::new();
    for (_, title, kind) in menu_bar::MENUS {
        let mut subs: Vec<PaletteSub> = Vec::new();
        for ((id, label, _), path) in leaf_rows_with_path(kind) {
            if id == crate::ids::MENUBAR_HELP_COMMANDS || opens_a_list(id) {
                continue;
            }
            let item = PaletteItem {
                badge: None,
                label: super::palette_label(label.tr()),
                id,
                keywords: "",
            };
            let title = (!path.is_empty()).then_some(path);
            match subs.iter_mut().find(|s| s.title == title) {
                Some(sub) => sub.items.push(item),
                None => subs.push(PaletteSub {
                    title,
                    items: vec![item],
                }),
            }
        }
        if !subs.is_empty() {
            out.push(PaletteGroup {
                title: title.tr().into(),
                color: ColorToken::NodeCatOutput,
                subs,
            });
        }
    }
    out
}
