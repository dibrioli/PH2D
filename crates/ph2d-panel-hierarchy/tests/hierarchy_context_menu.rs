//! ADR-0029 Phase C.2 — context-menu regression tests for the
//! Hierarchy panel. Ported from `ph2d-editor-core`'s
//! `screens::hero::tests::hier_menu_*` after the dev-dep-cycle gate.

use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use ph2d_editor_core::panel::{ErasedPanel, EventOutcome, Panel, PanelRegistry};
use ph2d_editor_core::screens::hero::ids;
use ph2d_panel_hierarchy::{HierarchyPanel, HierarchyState};
use std::sync::Once;

fn ensure_typed_registry() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let mut reg = PanelRegistry::new_empty();
        reg.push(ErasedPanel::new::<HierarchyPanel>());
        let _ = ph2d_editor_core::panel::install_panel_registry(reg);
    });
}

fn setup_hero() -> HeroScreen {
    ph2d_editor_core::test_support::ensure_panel_registry();
    ensure_typed_registry();
    ph2d_panel_hierarchy::clear_live_hierarchy();
    HeroScreen::new(NodeId(1))
}

fn dispatch(hero: &mut HeroScreen, state: &mut HierarchyState, ev: WidgetEvent) -> bool {
    matches!(
        HierarchyPanel::apply_event(state, hero, ev),
        EventOutcome::Consumed | EventOutcome::Observed,
    )
}

/// Stage a closed HierarchyRow snapshot so `apply_event` can read it
/// via `consume_last_context_menu`. Mirrors what dispatch does on
/// the menu-closing Down → next-frame-Click sequence.
fn stage_hierarchy_row_snapshot(hero: &mut HeroScreen, row: NodeId) {
    hero.store.open_context_menu(ContextMenuRequest {
        x: 0.0,
        y: 0.0,
        kind: ContextMenuKind::HierarchyRow { row },
    });
    hero.store.close_context_menu();
}

#[test]
fn hier_menu_duplicate_sets_pending_duplicate() {
    let mut hero = setup_hero();
    let mut state = HierarchyState::default();
    let row = NodeId(100_500);
    stage_hierarchy_row_snapshot(&mut hero, row);
    let consumed = dispatch(
        &mut hero,
        &mut state,
        WidgetEvent::Click(ids::CTX_MENU_HIER_DUPLICATE),
    );
    assert!(consumed);
    let drained: Vec<_> = hero.bus.drain().collect();
    assert_eq!(drained, vec![EditorAction::HierDuplicate { row }]);
    assert!(hero.store.last_context_menu().is_none());
}

#[test]
fn hier_menu_add_child_sets_pending_add_child() {
    let mut hero = setup_hero();
    let mut state = HierarchyState::default();
    let row = NodeId(100_501);
    stage_hierarchy_row_snapshot(&mut hero, row);
    let consumed = dispatch(
        &mut hero,
        &mut state,
        WidgetEvent::Click(ids::CTX_MENU_HIER_ADD_CHILD),
    );
    assert!(consumed);
    let drained: Vec<_> = hero.bus.drain().collect();
    assert_eq!(drained, vec![EditorAction::HierAddChild { row }]);
}

#[test]
fn hier_menu_reset_transform_sets_pending() {
    let mut hero = setup_hero();
    let mut state = HierarchyState::default();
    let row = NodeId(100_502);
    stage_hierarchy_row_snapshot(&mut hero, row);
    let consumed = dispatch(
        &mut hero,
        &mut state,
        WidgetEvent::Click(ids::CTX_MENU_HIER_RESET_TRANSFORM),
    );
    assert!(consumed);
    let drained: Vec<_> = hero.bus.drain().collect();
    assert_eq!(drained, vec![EditorAction::HierResetTransform { row }]);
}

#[test]
fn hier_menu_delete_sets_pending_delete() {
    let mut hero = setup_hero();
    let mut state = HierarchyState::default();
    let row = NodeId(100_503);
    stage_hierarchy_row_snapshot(&mut hero, row);
    let consumed = dispatch(
        &mut hero,
        &mut state,
        WidgetEvent::Click(ids::CTX_MENU_HIER_DELETE),
    );
    assert!(consumed);
    let drained: Vec<_> = hero.bus.drain().collect();
    assert_eq!(drained, vec![EditorAction::HierDelete { row }]);
}

#[test]
fn hier_menu_click_without_snapshot_consumes_but_no_pending() {
    let mut hero = setup_hero();
    let mut state = HierarchyState::default();
    // Defensive case: stray Click without any prior right-click
    // snapshot still consumes the event so the click doesn't
    // bubble to row selection, but no pending action is raised.
    let consumed = dispatch(
        &mut hero,
        &mut state,
        WidgetEvent::Click(ids::CTX_MENU_HIER_DUPLICATE),
    );
    assert!(consumed);
    assert!(hero.bus.is_empty());
}

#[test]
fn hier_menu_one_action_per_drain() {
    let mut hero = setup_hero();
    let mut state = HierarchyState::default();
    let row = NodeId(100_504);
    stage_hierarchy_row_snapshot(&mut hero, row);
    let _ = dispatch(
        &mut hero,
        &mut state,
        WidgetEvent::Click(ids::CTX_MENU_HIER_DUPLICATE),
    );
    let _ = dispatch(
        &mut hero,
        &mut state,
        WidgetEvent::Click(ids::CTX_MENU_HIER_DELETE),
    );
    let drained: Vec<_> = hero.bus.drain().collect();
    // Only Duplicate — the second Click consumed but found no
    // snapshot to attach a row to, so no Delete variant pushed.
    assert_eq!(drained, vec![EditorAction::HierDuplicate { row }]);
}

/// "Pack into Sheet" — a 2ª porta do verbo do pill `[SHEET]` (Enio, 2026-08-19: *«coloque a mesma
/// função do botão no menu do botão direito do mouse na hierarchy»*).
///
/// ⚠️ O que o teste fixa é o **payload**: a `NodeId` da LINHA, não uma entidade. Quem a resolve em
/// entidade — e quem decide entre "a seleção inteira" e "só esta linha" — é a shell, com a mesma
/// lei do "Merge Sprites" vizinho. Empurrar aqui uma entidade já resolvida obrigaria o painel a
/// conhecer o `bridge`, e é assim que um painel deixa de ser drop-in.
#[test]
fn hier_menu_pack_sheet_raises_pack_with_the_clicked_row() {
    let mut hero = setup_hero();
    let mut state = HierarchyState::default();
    let row = NodeId(100_505);
    stage_hierarchy_row_snapshot(&mut hero, row);
    let consumed = dispatch(
        &mut hero,
        &mut state,
        WidgetEvent::Click(ids::CTX_MENU_HIER_PACK_SHEET),
    );
    assert!(consumed);
    let drained: Vec<_> = hero.bus.drain().collect();
    assert_eq!(drained, vec![EditorAction::HierPackSheet { row }]);
    assert!(hero.store.last_context_menu().is_none());
}

/// **Toda linha do menu de uma row da hierarquia despacha alguma coisa.**
///
/// ⚠️ Este gate existe porque a doença já shipou aqui: o *"Use as Brush Shape"* nasceu pintado e
/// morto, e ninguém deu por isso durante semanas (Enio, 2026-06-25). Uma linha de menu que não
/// despacha é indistinguível de uma linha quebrada — e o custo de descobrir é o artista clicar,
/// não acontecer nada, e concluir que o app está partido.
///
/// ⚠️ **A fonte é a TABELA, nunca uma lista aqui dentro** (`menu_rows(HierarchyRow)` — a mesma que
/// o overlay pinta). Uma linha nova entra neste gate no dia em que é pintada, sem ninguém se
/// lembrar; um gate que precisasse de ser atualizado para cobrir o caso novo não apanharia
/// nenhum caso novo.
#[test]
fn every_hierarchy_row_menu_entry_dispatches_something() {
    use ph2d_editor_core::screens::hero::menu_rows::menu_rows;

    let mut dead: Vec<&str> = Vec::new();
    for (id, label, _) in menu_rows(ContextMenuKind::HierarchyRow { row: NodeId(1) }) {
        let mut hero = setup_hero();
        let mut state = HierarchyState::default();
        stage_hierarchy_row_snapshot(&mut hero, NodeId(100_600));
        let consumed = dispatch(&mut hero, &mut state, WidgetEvent::Click(*id));
        // O Rename é o único que age no store ANTES de empurrar (abre o modo inline) — mas
        // empurra na mesma o `HierRenameSeed`, então a barra de "empurrou algo" serve para as
        // dez linhas sem exceção. *Uma exceção aqui seria o buraco por onde a próxima passa.*
        if !consumed || hero.bus.drain().count() == 0 {
            dead.push(label);
        }
    }
    assert!(
        dead.is_empty(),
        "linhas do menu de contexto da hierarquia que são PINTADAS e não despacham nada — \
         clicar nelas não faz coisa nenhuma: {dead:?}.\n\
         Ligue cada uma em `crates/ph2d-panel-hierarchy/src/event.rs` (a cadeia de `id == \
         ids::CTX_MENU_HIER_*` e o braço que empurra a ação) e drene a ação na shell."
    );
}

/// "Remove from Sheet" — a saída, pela linha clicada (Enio, 2026-08-19).
///
/// ⚠️ O payload é a LINHA, como no irmão "Pack into Sheet": quem resolve em entidade, quem
/// verifica se ela está mesmo numa folha, e quem a devolve à raiz preservando a pose de mundo é a
/// shell. Um painel que soubesse disso deixaria de ser drop-in.
#[test]
fn hier_menu_remove_from_sheet_raises_remove_with_the_clicked_row() {
    let mut hero = setup_hero();
    let mut state = HierarchyState::default();
    let row = NodeId(100_506);
    stage_hierarchy_row_snapshot(&mut hero, row);
    let consumed = dispatch(
        &mut hero,
        &mut state,
        WidgetEvent::Click(ids::CTX_MENU_HIER_REMOVE_FROM_SHEET),
    );
    assert!(consumed);
    let drained: Vec<_> = hero.bus.drain().collect();
    assert_eq!(drained, vec![EditorAction::HierRemoveFromSheet { row }]);
}

/// "Auto-Arrange Pieces" — o verbo que ESTAVA escondido dentro do "Pack into Sheet", agora com
/// item próprio (Enio, 2026-08-19: *"uma opção no menu do botão direito da sheet: arrumar as
/// sprites filhas automaticamente"*).
#[test]
fn hier_menu_arrange_sheet_raises_arrange_with_the_clicked_row() {
    let mut hero = setup_hero();
    let mut state = HierarchyState::default();
    let row = NodeId(100_507);
    stage_hierarchy_row_snapshot(&mut hero, row);
    let consumed = dispatch(
        &mut hero,
        &mut state,
        WidgetEvent::Click(ids::CTX_MENU_HIER_ARRANGE_SHEET),
    );
    assert!(consumed);
    let drained: Vec<_> = hero.bus.drain().collect();
    assert_eq!(drained, vec![EditorAction::HierArrangeSheet { row }]);
}

/// ⚠️ **Os verbos da folha são AÇÕES distintas, um por item.** Enquanto "arrumar" vivia dentro de
/// "criar", o menu tinha dois itens para três coisas — e era o alvo, não o rótulo, que decidia o
/// que acontecia. Este teste é o que impede a fusão de voltar por conveniência: se dois destes
/// items passarem a levantar a mesma ação, ele reprova.
#[test]
fn every_sheet_verb_raises_its_own_action() {
    let rows = [
        ids::CTX_MENU_HIER_PACK_SHEET,
        ids::CTX_MENU_HIER_ARRANGE_SHEET,
        ids::CTX_MENU_HIER_REMOVE_FROM_SHEET,
        ids::CTX_MENU_HIER_BAKE_SHEET,
        ids::CTX_MENU_HIER_EXPORT_SHEET,
    ];
    let mut seen: Vec<EditorAction> = Vec::new();
    for (i, id) in rows.iter().enumerate() {
        let mut hero = setup_hero();
        let mut state = HierarchyState::default();
        let row = NodeId(100_600 + i as u64);
        stage_hierarchy_row_snapshot(&mut hero, row);
        assert!(dispatch(&mut hero, &mut state, WidgetEvent::Click(*id)));
        let mut drained: Vec<_> = hero.bus.drain().collect();
        assert_eq!(drained.len(), 1, "cada item levanta UMA acao");
        let action = drained.remove(0);
        assert!(
            !seen
                .iter()
                .any(|a| std::mem::discriminant(a) == std::mem::discriminant(&action)),
            "dois items da folha levantam a MESMA acao ({action:?}) — o rotulo deixaria de \
             prometer o que o item faz, e qual dos dois corre passaria a depender do alvo"
        );
        seen.push(action);
    }
}
