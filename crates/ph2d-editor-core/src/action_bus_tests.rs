//! Os gates do `action_bus` — irmão por LOC.
//!
//! ⚠️ Segue FILHO (`#[path]`), não um módulo separado: os testes usam
//! `use super::*` e alcançam os privados do pai. Cortado quando a §14 do
//! Inspector (W5) levou o arquivo a 708 > 700 — o corte por RESPONSABILIDADE
//! aqui é *o que o barramento É* contra *o que ele PROVA*.
use super::HierRequest;
use super::*;

#[test]
fn new_bus_is_empty() {
    let bus = ActionBus::new();
    assert!(bus.is_empty());
    assert_eq!(bus.len(), 0);
}

#[test]
fn push_then_len_grows() {
    let mut bus = ActionBus::new();
    bus.push(EditorAction::OneShotImageOp {
        tool_id: "trim_transparency",
        entity_bits: 42,
    });
    assert_eq!(bus.len(), 1);
    bus.push(EditorAction::OneShotImageOp {
        tool_id: "make_square",
        entity_bits: 99,
    });
    assert_eq!(bus.len(), 2);
}

#[test]
fn drain_returns_actions_in_push_order_and_empties() {
    // HR-5: actions drain in the exact push order. The shell
    // relies on this for the gizmo's drag-then-release sequence
    // and similar paired-intent cases.
    let mut bus = ActionBus::new();
    bus.push(EditorAction::OneShotImageOp {
        tool_id: "trim_transparency",
        entity_bits: 1,
    });
    bus.push(EditorAction::ActivateTool {
        tool_id: "bgremoval",
    });
    bus.push(EditorAction::OneShotImageOp {
        tool_id: "make_square",
        entity_bits: 2,
    });
    let drained: Vec<_> = bus.drain().collect();
    assert_eq!(drained.len(), 3);
    assert_eq!(
        drained[0],
        EditorAction::OneShotImageOp {
            tool_id: "trim_transparency",
            entity_bits: 1
        }
    );
    assert_eq!(
        drained[1],
        EditorAction::ActivateTool {
            tool_id: "bgremoval"
        }
    );
    assert_eq!(
        drained[2],
        EditorAction::OneShotImageOp {
            tool_id: "make_square",
            entity_bits: 2
        }
    );
    assert!(bus.is_empty(), "bus must be empty after drain");
}

#[test]
fn drain_on_empty_bus_returns_zero_items() {
    let mut bus = ActionBus::new();
    let drained: Vec<_> = bus.drain().collect();
    assert!(drained.is_empty());
    assert!(bus.is_empty());
}

#[test]
fn push_after_drain_starts_fresh_sequence() {
    let mut bus = ActionBus::new();
    bus.push(EditorAction::OneShotImageOp {
        tool_id: "trim_transparency",
        entity_bits: 1,
    });
    let _ = bus.drain().count();
    bus.push(EditorAction::OneShotImageOp {
        tool_id: "make_square",
        entity_bits: 2,
    });
    assert_eq!(bus.len(), 1);
    let drained: Vec<_> = bus.drain().collect();
    assert_eq!(
        drained,
        vec![EditorAction::OneShotImageOp {
            tool_id: "make_square",
            entity_bits: 2
        }]
    );
}

#[test]
fn clear_empties_without_dispatching() {
    let mut bus = ActionBus::new();
    bus.push(EditorAction::OneShotImageOp {
        tool_id: "trim_transparency",
        entity_bits: 1,
    });
    bus.push(EditorAction::ActivateTool {
        tool_id: "bgremoval",
    });
    bus.clear();
    assert!(bus.is_empty());
    // Subsequent drain yields nothing.
    let drained: Vec<_> = bus.drain().collect();
    assert!(drained.is_empty());
}

#[test]
fn editor_action_is_clone_and_partial_eq() {
    // Variants must implement these for test-side equality checks
    // and the shell's `match` clone scenarios. Locking via a
    // structural check so adding a non-Clone payload field fails
    // here loudly.
    fn assert_clone_partialeq<T: Clone + PartialEq>() {}
    assert_clone_partialeq::<EditorAction>();
}

#[test]
fn select_modifier_default_is_replace() {
    assert_eq!(SelectModifier::default(), SelectModifier::Replace);
}

#[test]
fn selection_variants_round_trip_through_bus() {
    let row_a = ph2d_a11y::NodeId(1);
    let row_b = ph2d_a11y::NodeId(2);
    let mut bus = ActionBus::new();
    bus.push(EditorAction::SelectSprite {
        entity_bits: 0xAAAA,
        modifier: SelectModifier::Replace,
    });
    bus.push(EditorAction::Hierarchy(HierRequest::SelectRow {
        row: row_a,
        modifier: SelectModifier::Add,
    }));
    bus.push(EditorAction::Hierarchy(HierRequest::SelectRow {
        row: row_b,
        modifier: SelectModifier::Toggle,
    }));
    bus.push(EditorAction::Hierarchy(HierRequest::RangeSelect {
        row: row_b,
    }));
    bus.push(EditorAction::ClearSelection);
    let drained: Vec<EditorAction> = bus.drain().collect();
    assert_eq!(
        drained,
        vec![
            EditorAction::SelectSprite {
                entity_bits: 0xAAAA,
                modifier: SelectModifier::Replace
            },
            EditorAction::Hierarchy(HierRequest::SelectRow {
                row: row_a,
                modifier: SelectModifier::Add
            }),
            EditorAction::Hierarchy(HierRequest::SelectRow {
                row: row_b,
                modifier: SelectModifier::Toggle
            }),
            EditorAction::Hierarchy(HierRequest::RangeSelect { row: row_b }),
            EditorAction::ClearSelection,
        ]
    );
}
