//! ⭐⭐⭐ **A barra de menus prometeu REALOJAR os menus dos pills — e isto mede a promessa.**
//!
//! Report do Enio (2026-09-04): *«depois de integração funções criadas por outros módulos não
//! aparecem na UI. Exemplo: Exportar Vector SVG.»*
//!
//! # O mecanismo, e por que nenhum gate o viu
//!
//! A retirada dos 29 pills (2026-08-30) apagou o **único** botão que abria o `SaveMenu`, o
//! `OpenMenu`, o `SettingsMenu`, o `ThemeSelector` e a `SceneList`. A barra de menus tomou o lugar
//! deles prometendo que cada row daqueles menus passava a ter casa em *File* / *Edit* / *View*.
//!
//! ⛔ **A prova dessa promessa era uma FRASE numa lista de excepções.** O gate irmão
//! (`every_topbar_verb_has_a_door_that_is_not_the_legacy_key`) isenta o `TOPBAR_SAVE` com o texto
//! *«as duas linhas do `SaveMenu` estão no menu File»* — e o censo dele corre sobre os **ids de
//! pill declarados** em `ids/chrome/topbar.rs`, nunca sobre as **rows** que cada pill abria. ⇒ o
//! dia em que a `line/Vector` juntou o *Export SVG…* ao `SaveMenu` (2026-09-02), a frase passou a
//! descrever três linhas dizendo duas, o verbo ficou alcançável **só pela paleta global**, e a
//! suíte inteira ficou verde. *Uma isenção que descreve uma contagem não sabe que envelheceu.*
//!
//! ⚠️ **É a mesma família de um item de menu MUDO, com o sintoma pior:** um item mudo o artista vê
//! e conclui que agiu; este ele **não vê**, e conclui que a funcionalidade não existe.
//!
//! # A régua: alcance REAL, não uma segunda tabela
//!
//! O conjunto alcançável é derivado **conduzindo o despacho**: parte-se dos menus da barra
//! ([`MENUS`]) e, para cada row, abre-se o menu-pai e **carrega-se na linha** — se ela abrir outro
//! menu, esse entra na fronteira. ⛔ Uma tabela `row → submenu` escrita aqui seria a terceira
//! cópia da cascata (`chrome::menu_bar` tem-na, os `chrome::settings_*` também) e divergiria em
//! silêncio; a cadeia real não pode.

use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use ph2d_editor_core::screens::hero::menu_bar::MENUS;
use ph2d_editor_core::screens::hero::menu_rows::{LEGACY_PILL_MENUS, menu_rows};
use ph2d_editor_core::{HeroScreen, NodeId};

fn hero() -> HeroScreen {
    ph2d_editor_core::test_support::ensure_panel_registry();
    HeroScreen::new(NodeId(1))
}

/// Todo id de row alcançável a partir dos títulos da barra, seguindo as cascatas reais.
///
/// ⚠️ **Um `hero` por clique.** Uma linha de alternância deixa estado atrás de si e a `Reset Panel
/// Layout` mexe na arrumação — reaproveitar a instância mediria o segundo clique de umas e o
/// primeiro de outras.
fn rows_reachable_from_the_bar() -> Vec<NodeId> {
    let mut frontier: Vec<ContextMenuKind> = MENUS.iter().map(|(_, _, k)| *k).collect();
    let mut ids: Vec<NodeId> = Vec::new();
    let mut i = 0;
    while i < frontier.len() {
        let kind = frontier[i];
        i += 1;
        for (id, _, _) in menu_rows(kind) {
            if !ids.contains(id) {
                ids.push(*id);
            }
            let mut h = hero();
            h.store.open_context_menu(ContextMenuRequest {
                x: 100.0,
                y: 100.0,
                kind,
            });
            let _ = h.apply_event(WidgetEvent::Click(*id));
            if let Some(open) = h.store.context_menu()
                && open.kind != kind
                && !frontier.contains(&open.kind)
            {
                frontier.push(open.kind);
            }
        }
    }
    ids
}

/// ⭐⭐⭐ **CENSO: toda row de um menu que a barra substituiu tem casa na barra.**
///
/// **Mutação que deve sangrar:** apagar a linha `Export SVG…` de `ContextMenuKind::MenuBarFile` —
/// é exactamente o estado em que o Enio encontrou o app.
#[test]
fn the_bar_relocated_every_row_of_the_menus_it_replaced() {
    let reachable = rows_reachable_from_the_bar();
    let mut homeless = Vec::new();
    for (_, kind) in LEGACY_PILL_MENUS {
        for (id, label, _) in menu_rows(*kind) {
            if !reachable.contains(id) {
                homeless.push(format!("{label} ({kind:?})"));
            }
        }
    }
    assert!(
        homeless.is_empty(),
        "estas linhas so' existiam no chrome LEGADO e a barra nao as realojou — o artista nao tem \
         como la' chegar (a paleta global nao conta: ela e' busca por nome, nao descoberta): \
         {homeless:?}"
    );
}

/// ⛔ **O controlo negativo do alcance.** Sem ele, um `rows_reachable_from_the_bar` que devolvesse
/// tudo — ou que se enganasse a percorrer a fronteira e devolvesse o mundo — deixaria o censo
/// acima verde para sempre.
#[test]
fn the_reach_is_a_closure_not_the_whole_id_space() {
    let reachable = rows_reachable_from_the_bar();
    assert!(
        reachable.len() >= 40,
        "so' {} rows alcancadas — a travessia das cascatas partiu-se e o censo acima passou a \
         medir quase nada",
        reachable.len()
    );
    for absent in [
        // Uma row de menu de CONTEXTO: ela precisa do sujeito que o clique deu, e por construção
        // nunca pode estar na barra.
        ph2d_editor_core::ids::CTX_MENU_CREATE_NOTE,
        ph2d_editor_core::ids::CTX_MENU_CURVE_HANDLE_FREE,
    ] {
        assert!(
            !reachable.contains(&absent),
            "a travessia chegou a uma row de menu de contexto — ela nao sai da barra, entao o \
             alcance esta' a ser calculado errado"
        );
    }
}
