//! ⭐⭐ **O menu *Window* alcança os TREZE módulos — e só esta crate consegue prová-lo.**
//!
//! A barra de menus nasceu em 2026-08-30 no lugar dos 29 pills. As treze linhas do menu *Window*
//! são a **única** porta de produto para os módulos desde então: enquanto elas não existiram, o
//! caminho era a tecla `F9`, que devolve o chrome legado — um interruptor de bissecção, não uma
//! porta.
//!
//! ⛔⛔ **Por que o gate mora AQUI e não ao lado dos irmãos dele.** As linhas destes menus são
//! despachadas por **duas cadeias diferentes**, e o `HeroScreen::apply_event` caminha-as por esta
//! ordem: o registo de **painéis** primeiro, o `chrome::dispatch_all` depois.
//!
//! | linha | quem despacha | visível de `ph2d-editor-core`? |
//! |---|---|---|
//! | Vector · Motion · Flip · Physics · Sculpt · Model · Image Tools · Tokens · Authored | `screens/hero/chrome/*_toggle.rs` | ✅ |
//! | **Audio Mixer · Audio Editor · Widget Gallery · Grid Settings** | o `event.rs` do próprio painel | ❌ |
//!
//! O `test_support::ensure_panel_registry` da `editor-core` é um `{}` — o registry vive nesta
//! crate, que depende dela. ⇒ um gate escrito lá acusaria **quatro** linhas correctas de serem
//! botões mortos. *Um gate escrito de uma camada deixa a outra por medir, e as duas metades desta
//! costura vivem em camadas diferentes de propósito.*
//!
//! *Mutação que sangra:* apagar uma linha da tabela `menu_rows(MenuBarWindow)` (o módulo fica
//! inalcançável e o gate da contagem cai), ou apagar o braço `TOPBAR_*` de um `event.rs` de painel.

use ph2d_editor_core::interaction::{ContextMenuKind, WidgetEvent};
use ph2d_editor_core::screens::hero::menu_rows::menu_rows;
use ph2d_editor_core::{HeroScreen, NodeId};

fn hero() -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    HeroScreen::new(NodeId(1))
}

/// **Cada linha do menu *Window* é consumida por alguém.**
///
/// ⚠️ A barra é a do `apply_event` devolver `true` — que é o que separa *«alguém agiu»* de
/// *«ninguém reconheceu o id»*. O `topbar::apply_event` imprime o nome do chip e devolve **false**
/// de propósito, então um id que só chegue lá conta como morto, e é exactamente isso que se quer.
#[test]
fn every_window_menu_row_reaches_a_consumer() {
    let rows = menu_rows(ContextMenuKind::MenuBarWindow);
    assert!(
        rows.len() >= 13,
        "o menu Window encolheu para {} linhas — um módulo ficou sem porta",
        rows.len()
    );
    let mut dead = Vec::new();
    for (id, label, _) in rows {
        // ⚠️ Um `hero` por linha: um toggle deixa estado atrás de si, e o segundo clique da mesma
        // sessão mediria o caminho de FECHO em vez do de abertura.
        let mut h = hero();
        if !h.apply_event(WidgetEvent::Click(*id)) {
            dead.push(*label);
        }
    }
    assert!(
        dead.is_empty(),
        "linhas do menu Window que ninguém despacha (botão morto): {dead:?}"
    );
}

/// ⭐ **E as quatro que só esta crate vê ABREM mesmo um painel** — não basta consumirem.
///
/// ⚠️ Estas quatro são as que a `editor-core` não alcança, então são as únicas que aqui merecem a
/// asserção forte: um `true` devolvido por engano (um braço que consome e não age) é o defeito que
/// o menu *Ficheiro* já teve durante meses.
///
/// ⚠️ **O alvo não é uma string escrita à mão.** A primeira redacção deste gate perguntava por
/// `"audio-mixer"` e o painel chama-se `"audio_mixer"` — um nome errado lê-se exactamente como
/// *«a costura está morta»*, que é a acusação mais cara que um gate pode fazer. A pergunta certa é
/// **derivada**: *que painéis mudaram de visibilidade?* — e a resposta tem de ser exactamente um.
#[test]
fn the_four_panel_dispatched_rows_actually_open_a_panel() {
    for id in [
        ph2d_editor_core::ids::TOPBAR_AUDIO_MIXER,
        ph2d_editor_core::ids::TOPBAR_AUDIO_EDITOR,
        ph2d_editor_core::ids::TOPBAR_WIDGET_GALLERY,
        ph2d_editor_core::ids::TOPBAR_GRID_SETTINGS,
    ] {
        let mut h = hero();
        let before = visibility_census(&h);
        assert!(
            h.apply_event(WidgetEvent::Click(id)),
            "{id:?}: não consumida"
        );
        let after = visibility_census(&h);
        let moved: Vec<&str> = before
            .iter()
            .zip(&after)
            .filter(|((_, b), (_, a))| b != a)
            .map(|((name, _), _)| *name)
            .collect();
        assert_eq!(
            moved.len(),
            1,
            "{id:?}: a linha foi consumida e mexeu em {} painéis (esperado 1): {moved:?}",
            moved.len()
        );
    }
}

/// A visibilidade de **todos** os painéis registados, na ordem do registry.
fn visibility_census(h: &HeroScreen) -> Vec<(&'static str, bool)> {
    let mut out = Vec::new();
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        for p in reg.panels() {
            out.push((p.manifest.id, h.is_panel_visible(p.manifest.id)));
        }
    });
    out
}
