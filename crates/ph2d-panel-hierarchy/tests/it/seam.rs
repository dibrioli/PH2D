//! Behavioral SEAM test for the Hierarchy panel `apply_event` — proves the
//! panel→bus seam is alive headless (blindagem Fase 0.1).
//!
//! Hierarchy has NO paired tool: its `apply_event` translates widget events
//! into `EditorAction::Hier*` variants that the shell drains off the action
//! bus and applies to the live `bevy_ecs::World`. So the observable effect of
//! a click is *a specific `EditorAction` landing on the bus* — that is what
//! this test asserts.
//!
//! `populate.rs` asserts widgets get registered and `ids/menus.rs` unit-tests
//! the companion-id bit round-trip — but NEITHER proves the wire from a real
//! `WidgetEvent::Click` through `event::apply_event` to `bus_mut().push(...)`
//! is intact. A forgotten arm or a wrong `EditorAction` projection leaves the
//! row painted, clickable and SILENTLY DEAD while every unit test stays green.
//!
//! This test runs the full path the desktop shell runs, headless:
//!   populate → Click(lock-companion) → apply_event → bus → drain → assert.

use ph2d_editor_core::action_bus::{EditorAction, HierRequest};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_panel_hierarchy::HierarchyPanel;
use ph2d_panel_hierarchy::state::HierarchyState;
use ph2d_ui_testkit::MockPanelHost;

/// Click a hierarchy row's lock-toggle companion and prove the panel emits the
/// exact `EditorAction::Hierarchy(HierRequest::ToggleLock { row })` onto the bus.
///
/// The lock companion is the cleanest unconditional driver: `apply_event`
/// recovers the row id from the companion bit (`hier_lock_companion_to_row`)
/// and pushes `HierToggleLock` BEFORE touching the store, selection, context
/// menu, or live-entries snapshot — so no host setup is required to fire it.
#[test]
fn lock_companion_click_emits_hier_toggle_lock() {
    let mut host = MockPanelHost::with_panel::<HierarchyPanel>();
    let mut panel_state = HierarchyState::default();

    // `HIER_PLAYER` is a real fixture row id seeded by `populate`. The shell's
    // dispatcher synthesizes the companion hit id for a click on the row's
    // lock icon; build the same id here.
    let row = ids::HIER_PLAYER;
    let lock_companion = ids::hier_lock_companion(row);

    let outcome = host
        .apply_panel_event::<HierarchyPanel>(&mut panel_state, WidgetEvent::Click(lock_companion));

    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Hierarchy ignored a real lock-companion click — `event.rs` lock-companion arm is missing or broken"
    );

    // The shell drains `EditorAction::Hier*` off the bus each frame and applies
    // it to the ECS world. Assert the EXACT variant the click must produce.
    let actions = host.drained_actions();
    assert!(
        actions.contains(&EditorAction::Hierarchy(HierRequest::ToggleLock { row })),
        "lock click never reached the bus as HierToggleLock {{ row: {row:?} }} — the Hierarchy panel→shell seam is dead. Drained: {actions:?}"
    );
}

/// ⭐ **O botão `Add` do cabeçalho emite `HierAddRoot`** (ADR-0166 / F3).
///
/// ⚠️ Ele era pintado (`paint.rs`) e registado (`populate.rs`) **desde a Fase C.2**, e nada no
/// repo consumia o clique — as duas primeiras perguntas da costura respondiam *sim* e o objeto
/// nunca nascia. *Um botão morto sob o dedo e um botão ausente dão o mesmo report.*
///
/// (Mutação: apagar o braço do `event.rs` ⇒ RED, e o `Ignored` diz exactamente o que aconteceu.)
#[test]
fn the_add_button_emits_hier_add_root() {
    let mut host = MockPanelHost::with_panel::<HierarchyPanel>();
    let mut panel_state = HierarchyState::default();

    let outcome = host.apply_panel_event::<HierarchyPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::HIERARCHY_ADD),
    );

    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "o botao Add do cabecalho voltou a ser inerte — o braco do event.rs sumiu"
    );
    let actions = host.drained_actions();
    assert!(
        actions.contains(&EditorAction::Hierarchy(HierRequest::AddRoot)),
        "o clique no Add nunca chegou ao barramento. Drenado: {actions:?}"
    );
}
