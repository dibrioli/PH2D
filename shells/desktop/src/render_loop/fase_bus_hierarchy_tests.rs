//! ⭐ Vários interruptores da linha num MESMO quadro (o arrasto que pinta, `toggle_paint`) chegam
//! todos ao mundo: o dreno do barramento guarda cada pedido, e a aplicação vira cada um.

use super::DrainOut;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::action_bus::{EditorAction, HierRequest};

#[test]
fn varios_interruptores_no_mesmo_quadro_aplicam_se_todos() {
    let mut app = crate::App::new();
    let mut sim = ph2d_ecs::SimWorld::new();
    let e: Vec<_> = (0..3)
        .map(|_| {
            sim.world_mut()
                .spawn(ph2d_ecs::Visibility { hidden: false })
                .id()
        })
        .collect();
    let row = |i: usize| NodeId(900 + i as u64);
    let mut pd = DrainOut::default();
    for i in 0..3 {
        let a = EditorAction::Hierarchy(HierRequest::ToggleVisibility { row: row(i) });
        assert!(app.fase_bus_hierarchy_tree(a, &mut pd).is_none());
    }
    for i in 0..2 {
        let a = EditorAction::Hierarchy(HierRequest::ToggleLock { row: row(i) });
        assert!(app.fase_bus_hierarchy_tree(a, &mut pd).is_none());
        let a = EditorAction::Hierarchy(HierRequest::ToggleGroup { row: row(i) });
        assert!(app.fase_bus_hierarchy_tree(a, &mut pd).is_none());
    }
    ph2d_ecs::row_flag::aplica(
        sim.world_mut(),
        std::mem::take(&mut pd.visibility_toggle_row),
        std::mem::take(&mut pd.lock_toggle_row),
        std::mem::take(&mut pd.group_toggle_row),
        |r| e.get((r.0 - 900) as usize).map(|x| x.to_bits()),
    );
    let w = sim.world();
    for (i, x) in e.iter().enumerate() {
        assert!(
            w.get::<ph2d_ecs::Visibility>(*x).is_some_and(|v| v.hidden),
            "a linha {i} pediu para esconder e ficou à vista"
        );
        let travada = w.get::<ph2d_ecs::Locked>(*x).is_some();
        let agrupada = w.get::<ph2d_ecs::GroupedChildren>(*x).is_some();
        assert_eq!((travada, agrupada), (i < 2, i < 2), "linha {i}");
    }
}
