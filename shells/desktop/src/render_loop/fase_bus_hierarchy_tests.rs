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

/// ⭐ **O olho da ÚNICA forma do Player** (cena 120, smoke do dono 09/11), pela porta da UI: o corpo
/// FICA no solver (cai com a gravidade e atravessa o Floor, que já não toca) e o Inspector continua a
/// mostrar o cartão Physics Body. Antes, `main_shape == None` tirava-o do rapier e da §11.
#[test]
fn o_olho_da_unica_forma_do_player_nao_o_tira_do_solver_nem_do_inspector() {
    use ph2d_ecs::{Name, Visibility};
    let mut app = crate::App::new();
    let mut sim = ph2d_ecs::SimWorld::new();
    ph2d_physics_scene::ficha::monta(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    let player = {
        let w = sim.world_mut();
        let mut q = w.query::<(ph2d_ecs::Entity, &Name)>();
        q.iter(w).find(|(_, n)| n.0 == "Player").map(|(e, _)| e)
    }
    .expect("o Player da cena 120");
    let forma = ph2d_physics_ecs::main_shape(sim.world(), player).expect("a forma do Player");
    assert_ne!(forma, player, "opção B: a forma é uma filha");
    assert_eq!(
        ph2d_physics_ecs::shapes_in_order(sim.world(), player).len(),
        1
    );

    let row = NodeId(7_000);
    let mut pd = DrainOut::default();
    let a = EditorAction::Hierarchy(HierRequest::ToggleVisibility { row });
    assert!(app.fase_bus_hierarchy_tree(a, &mut pd).is_none());
    ph2d_ecs::row_flag::aplica(
        sim.world_mut(),
        std::mem::take(&mut pd.visibility_toggle_row),
        std::mem::take(&mut pd.lock_toggle_row),
        std::mem::take(&mut pd.group_toggle_row),
        |r| (r == row).then(|| forma.to_bits()),
    );
    let w = sim.world();
    assert!(
        w.get::<Visibility>(forma).is_some_and(|v| v.hidden),
        "o olho não fechou a forma"
    );
    assert!(
        !w.get::<Visibility>(player).is_some_and(|v| v.hidden),
        "o olho escreveu no corpo"
    );
    assert_eq!(
        ph2d_physics_ecs::main_shape(w, player),
        None,
        "a única forma está desligada"
    );

    let mut bridge = ph2d_physics_ecs::PhysicsBridge::new();
    let base = ph2d_physics_ecs::PhysicsSettings::default();
    bridge.set_settings(ph2d_physics_scene::ficha::regras(base));
    let y = |sim: &ph2d_ecs::SimWorld| {
        ph2d_ecs::world_transform(sim.world(), player).map_or(f32::NAN, |t| t.translation.y)
    };
    let y0 = y(&sim);
    for t in 0..90 {
        bridge.dispatch(&mut sim, true, t);
    }
    let y1 = y(&sim);
    let info = ph2d_app_physics::inspector::body::build_physics_info(
        sim.world(),
        player.to_bits(),
        0,
        0,
        0,
        false,
        0,
        (0.0, 5.0),
        0,
        &ph2d_tags::TagTree::new(),
    )
    .expect("a §11 do Player");
    eprintln!(
        "Player y {y0:.3} -> {y1:.3} em 90 tiques; has_body {}; sem forma {}",
        info.has_body, info.body_without_shape
    );
    assert!(
        bridge.body_pose(player).is_some(),
        "o Player saiu do solver"
    );
    // O topo do Floor é `y = 0`: o Player atravessa-o, porque a forma desligada não toca nada.
    assert!(
        y1 < y0 - 1.0 && y1 < 0.0,
        "o Player não caiu: {y0:.3} -> {y1:.3}"
    );
    assert!(
        info.has_body && !info.body_without_shape,
        "o cartão Physics Body sumiu"
    );
}
