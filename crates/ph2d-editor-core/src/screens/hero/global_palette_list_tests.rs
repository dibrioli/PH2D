//! A paleta global em LISTA (versão C da barra): o caminho de cada linha, os recentes e o
//! destaque. Irmão de `global_palette_tests` (tecto de linhas).

use super::*;

/// Todos os itens do modelo, com o caminho de cada um.
fn with_paths(m: &PaletteModel) -> Vec<(NodeId, String, String)> {
    use crate::widget::command_palette::path_of;
    m.groups
        .iter()
        .flat_map(|g| g.subs.iter().map(move |s| (g, s)))
        .flat_map(|(g, s)| {
            s.items
                .iter()
                .map(move |it| (it.id, it.label.clone(), path_of(g, s)))
        })
        .collect()
}

/// ⭐ **A paleta global é uma LISTA, e cada folha da barra de menus está nela com o caminho de onde
/// mora** — derivado de `MENUS` + `CASCADES`, através das cascatas (`File · Export`).
///
/// (Mutações: `leaf_rows_with_path` sem atravessar a cascata ⇒ RED no «Export»; emitir as
/// linhas-cascata ⇒ RED; tirar um menu de `groups()` ⇒ RED no alcance.)
#[test]
fn every_bar_menu_leaf_is_in_the_list_with_the_path_it_lives_at() {
    use crate::interaction::ContextMenuKind;
    use crate::screens::hero::menu_bar;
    let m = model(false);
    assert!(
        m.is_list() && !m.two_step(),
        "a paleta global é lista e escolhe num clique"
    );
    let all = with_paths(&m);
    let mut seen = 0usize;
    for (_, title, kind) in menu_bar::MENUS {
        for (id, label, _) in menu_bar::leaf_rows(kind) {
            if id == crate::ids::MENUBAR_HELP_COMMANDS || global_palette_menus::opens_a_list(id) {
                continue;
            }
            assert!(
                all.iter().any(|(it, _, _)| *it == id),
                "a folha {:?} de {} não é alcançável pela paleta",
                label.tr(),
                title.tr()
            );
            seen += 1;
        }
    }
    assert!(seen >= 30, "a barra tem folhas; contou {seen}");
    for (row, _) in menu_bar::CASCADES {
        assert!(
            !all.iter().any(|(it, _, _)| *it == row),
            "uma linha-cascata não é um comando"
        );
    }
    // O caminho de uma folha em cascata: a 1.ª linha do submenu Export mora em «File · Export».
    let export = global_palette_menus::leaf_rows_with_path(ContextMenuKind::MenuBarFile)
        .into_iter()
        .find(|(row, _)| {
            menu_bar::leaf_rows(ContextMenuKind::MenuBarFileExport)
                .iter()
                .any(|r| r.0 == row.0)
        })
        .expect("o File tem o submenu Export");
    assert_eq!(export.1, "Export");
    let (_, _, path) = all
        .iter()
        .find(|(id, _, _)| *id == export.0.0)
        .expect("está no modelo");
    assert_eq!(path, "File \u{00b7} Export");
    // E é esse o caminho que a lista mostra ao digitar o nome dele.
    let name = palette_label(export.0.1.tr());
    let name = crate::widget::command_palette::split_shortcut(&name)
        .0
        .to_owned();
    let view = scoped(&m, &name, None);
    let flat = with_paths(&view.model);
    let (_, _, shown) = flat.iter().find(|(id, _, _)| *id == export.0.0).unwrap();
    assert_eq!(shown, "File \u{00b7} Export");
    assert_eq!(
        global_palette_menus::place_name("Export \u{203a}"),
        "Export"
    );
    assert_eq!(global_palette_menus::place_name("New\u{2026}"), "New");
}

/// ⭐ **«Recent»: o que esta paleta correu por último abre em cima, o mais novo primeiro, sem
/// repetir; um id que o modelo de agora não oferece não aparece.**
///
/// (Mutações: `route_global_pick` sem `remember` ⇒ RED; `remember` sem o `retain` ⇒ RED.)
#[test]
fn what_the_palette_ran_last_opens_on_top() {
    use crate::ids;
    let mut hero = HeroScreen::new(NodeId(1));
    // Um tema que o menu de AGORA oferece (a tabela muda com `ui_is_redesign`).
    let tema = crate::screens::hero::menu_rows::menu_rows(
        crate::interaction::ContextMenuKind::ThemeSelector,
    )[1]
    .0;
    assert!(route_global_pick(&mut hero, tema));
    assert!(route_global_pick(&mut hero, ids::CTX_MENU_PPM_256));
    assert!(route_global_pick(&mut hero, tema));
    assert!(
        !route_global_pick(&mut hero, NodeId(0xDEAD_BEEF)),
        "id de ninguém"
    );
    hero.menu_memory.remember(NodeId(0xDEAD_BEEF));
    assert_eq!(
        hero.menu_memory.recent(),
        [NodeId(0xDEAD_BEEF), tema, ids::CTX_MENU_PPM_256]
    );
    open(&mut hero);
    let m = hero.store.command_palette_model().expect("aberta");
    let view = scoped(m, "", None);
    assert_eq!(view.model.groups[0].title, tr("chrome.palette.recent"));
    let ids_shown = item_ids(&view.model);
    assert_eq!(ids_shown[..2], [tema, ids::CTX_MENU_PPM_256]);
    assert_eq!(
        view.model.groups[0].subs.len(),
        2,
        "o id de ninguém não aparece"
    );
    let mut unique = ids_shown.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), ids_shown.len(), "a lista repetiu um comando");
    assert_eq!(
        hero.store.command_palette_highlight(),
        Some(tema),
        "o destaque abre na 1.ª linha"
    );
}

/// ⭐ **A seleção por teclado rola a lista**: descer até ao fim leva a rolagem com ela; a busca
/// nova devolve o destaque e a rolagem ao topo.
///
/// (Mutação: `drain` sem o `follow` ⇒ RED.)
#[test]
fn the_selected_row_scrolls_into_view() {
    use crate::widget::command_palette::CMD_PALETTE_CARD;
    let mut hero = HeroScreen::new(NodeId(1));
    open(&mut hero);
    let view_h = ph2d_tokens::ROW_H_PX * 4.0;
    hero.store.set_panel_visible_h(CMD_PALETTE_CARD, view_h);
    drain(&mut hero);
    assert_eq!(hero.store.panel_scroll_target(CMD_PALETTE_CARD), 0.0);
    hero.store.step_command_palette_highlight(isize::MAX);
    drain(&mut hero);
    let down = hero.store.panel_scroll_target(CMD_PALETTE_CARD);
    assert!(
        down > view_h,
        "a última linha ficou fora da vista: rolagem {down}"
    );
    // A roda continua livre: sem mudar o destaque, ninguém a puxa de volta.
    hero.store.set_panel_scroll(CMD_PALETTE_CARD, 0.0);
    drain(&mut hero);
    assert_eq!(hero.store.panel_scroll_target(CMD_PALETTE_CARD), 0.0);
    hero.store.step_command_palette_highlight(isize::MAX);
    hero.store.set_panel_scroll(CMD_PALETTE_CARD, down);
    hero.store.command_palette_push_char('e');
    drain(&mut hero);
    let m = hero.store.command_palette_model().expect("aberta");
    let first = item_ids(&scoped(m, "e", None).model)[0];
    assert_eq!(hero.store.command_palette_highlight(), Some(first));
    assert_eq!(hero.store.panel_scroll_target(CMD_PALETTE_CARD), 0.0);
}

/// ⭐ **Uma linha que ABRE um submenu não é um comando da paleta** (Preferences…, Theme…, as
/// cascatas): escolhida daqui abriria uma lista ancorada a uma linha que ninguém pintou. Controlo:
/// `Scenes…` fica (abre a lista de cenas) e as folhas de Theme continuam alcançáveis.
///
/// (Mutação: `opens_a_list` devolver sempre `false` ⇒ RED em Preferences e Theme.)
#[test]
fn no_palette_item_opens_a_submenu() {
    use crate::ids;
    use crate::screens::hero::menu_cascade::opens_submenu;
    let mut hero = HeroScreen::new(NodeId(1));
    open(&mut hero);
    let m = hero.store.command_palette_model().expect("aberta");
    let all = with_paths(m);
    assert!(all.len() >= 30, "a paleta ficou vazia: {}", all.len());
    for (id, label, path) in &all {
        assert!(
            !opens_submenu(*id),
            "«{label}» ({path}) abre um submenu e está na paleta"
        );
    }
    assert!(opens_submenu(ids::MENUBAR_EDIT_PREFERENCES) && opens_submenu(ids::MENUBAR_VIEW_THEME));
    assert!(all.iter().any(|(id, ..)| *id == ids::MENUBAR_FILE_SCENES));
    let tema = crate::screens::hero::menu_rows::menu_rows(
        crate::interaction::ContextMenuKind::ThemeSelector,
    )[0]
    .0;
    assert!(
        all.iter().any(|(id, ..)| *id == tema),
        "as folhas de Theme ficam"
    );
}

/// ⭐ **Digitar e dar Enter no MESMO quadro corre a 1.ª linha filtrada**, não o destaque antigo —
/// o mesmo ao apagar. Controlo: sem mexer na busca, o Enter é do destacado.
///
/// (Mutação: tirar o `drop_list_highlight` do `push_char`/`backspace` ⇒ RED.)
#[test]
fn typing_drops_the_old_highlight_at_once() {
    let mut hero = HeroScreen::new(NodeId(1));
    open(&mut hero);
    let first = |hero: &HeroScreen, q: &str| {
        let m = hero.store.command_palette_model().expect("aberta");
        item_ids(&scoped(m, q, hero.store.command_palette_scopes()).model)[0]
    };
    // Uma letra cuja 1.ª linha filtrada não é a última da lista filtrada (senão não mede nada).
    let q = ['e', 'a', 's', 't']
        .into_iter()
        .find(|c| {
            let m = hero.store.command_palette_model().expect("aberta");
            item_ids(&scoped(m, &c.to_string(), None).model).len() > 2
        })
        .expect("uma busca com várias linhas");
    hero.store.command_palette_push_char(q);
    drain(&mut hero);
    hero.store.step_command_palette_highlight(isize::MAX);
    let antigo = hero
        .store
        .command_palette_enter_target()
        .expect("destacado");
    assert_ne!(
        antigo,
        first(&hero, &q.to_string()),
        "controlo: o destaque desceu"
    );
    assert_eq!(hero.store.command_palette_enter_target(), Some(antigo));
    // Apagar: sem `drain` no meio, o Enter já é da 1.ª linha da lista inteira.
    hero.store.command_palette_backspace();
    assert_eq!(
        hero.store.command_palette_enter_target(),
        Some(first(&hero, ""))
    );
    // Digitar: idem, com o destaque antigo ainda VISÍVEL na lista filtrada.
    drain(&mut hero);
    hero.store.step_command_palette_highlight(isize::MAX);
    let ultimo = hero
        .store
        .command_palette_enter_target()
        .expect("destacado");
    hero.store.command_palette_push_char(q);
    let filtrada = first(&hero, &q.to_string());
    let m = hero.store.command_palette_model().expect("aberta");
    let visiveis = item_ids(&scoped(m, &q.to_string(), None).model);
    if visiveis.contains(&ultimo) {
        assert_ne!(ultimo, filtrada);
    }
    assert_eq!(hero.store.command_palette_enter_target(), Some(filtrada));
}

/// Uma família de documentos com dois comandos de família: um na barra (File ▸ Export) e um só do
/// menu da aba.
struct Kanbans;

const EXPORT_KANBAN: NodeId = NodeId(0x6b61_6e62_616e_0001);
const TAB_ONLY: NodeId = NodeId(0x6b61_6e62_616e_0002);

impl crate::doc_surface::DocumentSurface for Kanbans {
    fn tabs(&self) -> Vec<crate::doc_surface::DocTab> {
        Vec::new()
    }
    fn paint(
        &mut self,
        _: u64,
        _: crate::zones::Rect,
        _: &mut ph2d_vector::VectorScene,
        _: &mut ph2d_text::TextSystem,
        _: &ph2d_tokens::Theme,
    ) {
    }
    fn pointer(
        &mut self,
        _: u64,
        _: crate::doc_surface::PointerInput<'_>,
        _: ph2d_host::PointerKind,
        _: ph2d_host::PointerButton,
        _: f32,
        _: f32,
    ) -> bool {
        false
    }
    fn pointer_move(
        &mut self,
        _: u64,
        _: crate::doc_surface::PointerInput<'_>,
        _: f32,
        _: f32,
    ) -> bool {
        false
    }
    fn wheel(&mut self, _: u64, _: f32, _: f32, _: f32, _: bool) -> bool {
        false
    }
    fn key(
        &mut self,
        _: u64,
        _: &mut ph2d_text::TextSystem,
        _: crate::doc_surface::KeyInput<'_>,
    ) -> crate::doc_surface::KeyOutcome {
        crate::doc_surface::KeyOutcome::default()
    }
    fn compose(&mut self, _: u64, _: crate::doc_surface::Compose<'_>) -> bool {
        false
    }
    fn close(&mut self, _: u64) {}
    fn undo(&mut self, _: u64, _: bool) -> bool {
        false
    }
    fn family_commands(&self) -> Vec<crate::doc_surface::DocCommand> {
        use crate::doc_surface::{DocCommand, DocMenu};
        vec![
            DocCommand {
                id: EXPORT_KANBAN,
                label: "Export Kanban\u{2026}".into(),
                menu: DocMenu::Export,
            },
            DocCommand {
                id: TAB_ONLY,
                label: "Tab Only".into(),
                menu: DocMenu::Tab,
            },
        ]
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// ⭐⭐ **Um comando com linha na barra leva o caminho DELA e aparece uma vez só** (foto 8 da onda
/// 2: «Export Kanban…» vinha como «This tab», em 1.º lugar). E as folhas das Preferências moram em
/// `Edit · Preferences · <lista>`, o tema em `View · Theme` — nunca «Menus · Motion».
///
/// (Mutações: pôr os comandos de família todos em «This tab» ⇒ RED; o caminho das listas pelo
/// título antigo ⇒ RED no Expressive.)
#[test]
fn um_comando_da_barra_leva_o_caminho_dela_e_aparece_uma_vez() {
    let mut hero = HeroScreen::new(NodeId(1));
    hero.documents.install_surface(Box::new(Kanbans));
    let m = build_global_model(&hero);
    let all = with_paths(&m);
    let export: Vec<_> = all.iter().filter(|(id, ..)| *id == EXPORT_KANBAN).collect();
    assert_eq!(export.len(), 1, "Export Kanban… aparece uma vez só");
    assert_eq!(export[0].2, "File \u{00b7} Export");
    let tab = tr("chrome.palette.document");
    assert!(
        !all.iter()
            .any(|(id, _, path)| *path == tab && *id != TAB_ONLY),
        "na aba Scene, «This tab» só tem o que é do menu da aba"
    );
    // Ao digitar, a 1.ª linha de «exp» não é a do Kanban por vir de um grupo de cima.
    let view = scoped(&m, "Export Kanban", None);
    let flat = with_paths(&view.model);
    assert_eq!(
        flat.iter().filter(|(id, ..)| *id == EXPORT_KANBAN).count(),
        1
    );
    // As listas das Preferências e o tema.
    let path_of = |id: NodeId| {
        all.iter()
            .find(|(i, ..)| *i == id)
            .map(|(_, _, p)| p.clone())
    };
    assert_eq!(
        path_of(crate::ids::CTX_MENU_MOTION_EXPRESSIVE).as_deref(),
        Some("Edit \u{00b7} Preferences \u{00b7} Motion")
    );
    let theme_row = crate::screens::hero::menu_rows::menu_rows(
        crate::interaction::ContextMenuKind::ThemeSelector,
    )[0]
    .0;
    assert_eq!(path_of(theme_row).as_deref(), Some("View \u{00b7} Theme"));
    let ids: Vec<NodeId> = all.iter().map(|(id, ..)| *id).collect();
    let mut dedup = ids.clone();
    dedup.sort_by_key(|n| n.0);
    dedup.dedup();
    assert_eq!(dedup.len(), ids.len(), "um id, uma linha");
}
