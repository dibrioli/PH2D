//! **UMA LINHA DE MENU PINTADA TEM DE ESTAR REGISTADA NO STORE.**
//!
//! # O defeito, e por que ele não dá erro nenhum
//!
//! O despachante de ponteiro rejeita um id que não está no [`WidgetStore`]: sem um estado de
//! `Button`, o id não é `is_focusable`, não fica `active` no Down, e **nunca emite `Click`**. A
//! linha pinta, o rato passa por cima, o artista clica — e não acontece coisa nenhuma.
//!
//! ⚠️ **Nada nesta cadeia falha alto.** Não há erro, não há log, não há aviso: o único sintoma é um
//! botão morto, e a conclusão do artista é que o app está partido.
//!
//! # Como isto foi encontrado, que é o motivo de o gate existir
//!
//! Enio, 2026-08-21: *"Export não abre dialog para salvar."* A linha **"Export Image…"** tinha
//! nascido no mesmo dia, e estava correcta em **dois** dos três sítios que uma linha de menu exige:
//!
//! | sítio | estava? |
//! |---|---|
//! | a **tabela** (`menu_rows`) — o que se pinta | ✅ |
//! | o **guarda** + a cadeia (`ph2d-panel-hierarchy/src/event.rs`) — o que se despacha | ✅ |
//! | o **registo** (`pre_populate`) — o que se pode CLICAR | ❌ |
//!
//! ⚠️ E o gate irmão `every_hierarchy_row_menu_entry_dispatches_something` estava **verde**, porque
//! ele chama o handler **directamente** — salta a camada de ponteiro, que é precisamente onde o
//! clique morria. *Dois gates a medir a mesma linha podem estar os dois certos e deixar o buraco no
//! meio: um mede o que o handler faz, o outro tem de medir se o handler chega a ser chamado.*
//!
//! # Por que a fonte é a TABELA
//!
//! ⛔ Uma lista de ids dentro deste teste teria de ser actualizada para cobrir o caso novo — e um
//! gate que precisa de ser actualizado para apanhar o caso novo não apanha caso novo nenhum. Ele lê
//! `menu_rows` para **todos** os `ContextMenuKind` que têm tabela, por isso uma linha nova entra
//! aqui no dia em que é pintada, sem ninguém se lembrar.

use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::interaction::ContextMenuKind;
use ph2d_editor_core::screens::hero::menu_rows::menu_rows;

/// Os menus cujas linhas são botões simples — clicar despacha uma acção.
///
/// ⚠️ **Os modais NÃO entram**: `NewImageDialog`, `SheetSizeDialog` e `RenamePaletteDialog` pintam
/// campos e radios com o seu próprio registo (o `pre_populate` diz porquê ao lado deles), e exigir
/// deles a mesma forma seria o gate a inventar uma regra que o produto não tem.
fn simple_menus() -> Vec<ContextMenuKind> {
    vec![ContextMenuKind::HierarchyRow {
        row: ph2d_a11y::NodeId(1),
    }]
}

#[test]
fn every_painted_menu_row_is_registered_and_therefore_clickable() {
    // ⚠️ Pelo `HeroScreen::new`, que é quem chama o `pre_populate_store` no produto. Construir um
    // `WidgetStore` à mão e chamar o povoador directamente mediria uma cadeia que o app não usa.
    let hero = HeroScreen::new(NodeId(1));

    let mut dead: Vec<String> = Vec::new();
    for kind in simple_menus() {
        for (id, label, _) in menu_rows(kind) {
            // ⚠️ **A barra é a do `is_focusable` do despachante**
            // (`interaction/dispatch/focus.rs`), e não uma inventada aqui: o que mata o clique é o
            // ramo `None => false` — um id **ausente** do store não fica `active` no Down e o
            // `Click` nunca nasce.
            //
            // ⚠️ **Não se exige `Button`.** As linhas de menu registam-se como `Plain`, que o
            // `is_focusable` aceita — a primeira versão deste gate exigiu `Button` e acusou as
            // dezassete, incluindo as que funcionam há meses. *Um gate que acusa o legítimo é
            // desligado na primeira semana; a barra tem de ser a que o produto de facto usa.*
            if hero.store.get(*id).is_none() {
                dead.push((*label).to_string());
            }
        }
    }
    assert!(
        dead.is_empty(),
        "estas linhas de menu sao PINTADAS mas nao estao registadas no `WidgetStore` — o \
         despachante de ponteiro rejeita o id, o clique nunca vira `Click`, e o artista ve' um \
         botao morto:\n  {}\n\n\
         Registe cada uma em `screens/hero/pre_populate.rs` (a lista de `CTX_MENU_*`).\n\n\
         ⚠️ Uma linha de menu vive em TRES sitios: a tabela (`menu_rows`, o que se pinta), o \
         guarda + a cadeia do painel (o que se despacha) e o registo (o que se pode CLICAR). \
         Faltar um dos tres nao da' erro nenhum -- da' um botao que nao faz nada.",
        dead.join("\n  ")
    );
}
