//! ⭐ **File ▸ Open Recent** (line/BarraDeMenus, onda 3): os projetos que a
//! [`super::menu_memory::MenuMemory`] lembra viram linhas do submenu — o nome do ficheiro e, na
//! coluna da direita, a pasta. Vazio, uma linha cinzenta diz que não há nenhum. Escolher pede à
//! shell para abrir aquele caminho (`file_menu.open_path`); a shell é quem tem o disco.

use super::HeroScreen;
use crate::ids;
use crate::interaction::RecentRow;
use ph2d_a11y::NodeId;
use ph2d_i18n::tr;

/// O rótulo de um projeto: «nome · pasta», com a pasta pessoal como `~`.
#[must_use]
pub fn label_of(path: &str) -> String {
    let p = std::path::Path::new(path);
    let name = p
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
        .to_owned();
    let Some(dir) = p
        .parent()
        .and_then(|d| d.to_str())
        .filter(|d| !d.is_empty())
    else {
        return name;
    };
    let home = std::env::var("HOME").ok().filter(|h| !h.is_empty());
    let dir = match home.as_deref().and_then(|h| dir.strip_prefix(h)) {
        Some(rest) => format!("~{rest}"),
        None => dir.to_owned(),
    };
    format!("{name} \u{00b7} {dir}")
}

/// As linhas que a memória dá agora.
#[must_use]
pub fn rows(projects: &[String]) -> Vec<RecentRow> {
    if projects.is_empty() {
        return vec![RecentRow {
            id: ids::MENUBAR_FILE_RECENT_NONE,
            label: tr("chrome.menu.recent_none").to_owned(),
            path: None,
        }];
    }
    projects
        .iter()
        .enumerate()
        .map(|(i, p)| RecentRow {
            id: ids::recent_project_row(i),
            label: label_of(p),
            path: Some(p.clone()),
        })
        .chain(std::iter::once(RecentRow {
            id: ids::MENUBAR_FILE_RECENT_CLEAR,
            label: tr("chrome.menu.recent_clear").to_owned(),
            path: None,
        }))
        .collect()
}

/// Publica as linhas no store quando a lista mudou (uma vez por quadro, no tique).
pub fn publish(hero: &mut HeroScreen) {
    let projects = hero.menu_memory.projects();
    let same = {
        let now = hero.store.recent_rows();
        let paths = now.iter().filter_map(|r| r.path.as_deref());
        !now.is_empty() && paths.eq(projects.iter().map(String::as_str))
    };
    if !same {
        let rows = rows(projects);
        hero.store.set_recent_rows(rows);
    }
}

/// O clique numa linha do *Open Recent*. `true` se era uma delas.
pub fn apply(hero: &mut HeroScreen, id: NodeId) -> bool {
    if id == ids::MENUBAR_FILE_RECENT_NONE {
        return true;
    }
    if id == ids::MENUBAR_FILE_RECENT_CLEAR {
        hero.menu_memory.clear_projects();
        hero.store.close_context_menu();
        return true;
    }
    let Some(path) = hero
        .store
        .recent_rows()
        .iter()
        .find(|r| r.id == id)
        .and_then(|r| r.path.clone())
    else {
        return false;
    };
    hero.file_menu.open_path = Some(path);
    hero.store.close_context_menu();
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vazio_diz_que_nao_ha_e_cheio_acaba_no_clear() {
        let r = rows(&[]);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].id, ids::MENUBAR_FILE_RECENT_NONE);
        let r = rows(&["/a/b/level.ph2dproj".into(), "/c.ph2dproj".into()]);
        assert_eq!(r.len(), 3);
        assert_eq!(r[0].label, "level \u{00b7} /a/b");
        assert_eq!(r[2].id, ids::MENUBAR_FILE_RECENT_CLEAR);
    }

    /// ⭐ Escolher um recente pede à shell aquele caminho e fecha o menu; o Clear esvazia a lista.
    #[test]
    fn escolher_um_recente_pede_o_caminho_a_shell() {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.menu_memory.note_project("/x/one.ph2dproj");
        hero.menu_memory.note_project("/x/two.ph2dproj");
        publish(&mut hero);
        let first = ids::recent_project_row(0);
        assert!(hero.apply_event(crate::interaction::WidgetEvent::Click(first)));
        assert_eq!(
            hero.file_menu.open_path.as_deref(),
            Some("/x/two.ph2dproj"),
            "o mais novo é a primeira linha"
        );
        assert!(hero.apply_event(crate::interaction::WidgetEvent::Click(
            ids::MENUBAR_FILE_RECENT_CLEAR
        )));
        assert!(hero.menu_memory.projects().is_empty());
        publish(&mut hero);
        assert_eq!(
            hero.store.recent_rows()[0].id,
            ids::MENUBAR_FILE_RECENT_NONE
        );
    }
}
