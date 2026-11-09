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

/// ⭐ **O olho da ÚNICA forma do Player** (cena 120, smoke do dono 09/11), pela CADEIA REAL da UI: o retrato da
/// Hierarquia (`build_hierarchy_snapshot` → `EntityNodeMap::sync_from_snapshot`, o de `publish_hierarchy`) dá a
/// linha que o dono vê logo abaixo de «Player»; o `ToggleVisibility` dela (o clique no olho emite-o — gate
/// `ph2d-panel-hierarchy` `mais_olho_cadeado_e_grupo_respondem_nos_rects_pintados`) passa pelo dreno e pelo
/// `hierarchy::dispatch`, que resolve a linha pela ponte. Depois de 90 tiques de Play o corpo FICA no solver
/// (cai e atravessa o Floor) e o Inspector PINTA o cartão Physics Body. Antes, `main_shape == None` tirava-o do
/// rapier e trocava o cartão pelo «No shape».
#[cfg(feature = "panel-inspector")]
#[test]
fn o_olho_da_unica_forma_do_player_nao_o_tira_do_solver_nem_do_inspector() {
    use ph2d_ecs::{Entity, Name, Visibility};
    let mut app = crate::App::new();
    let mut sim = ph2d_ecs::SimWorld::new();
    ph2d_physics_scene::ficha::monta(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_missing_sibling_order(sim.world_mut());
    let player = {
        let w = sim.world_mut();
        let mut q = w.query::<(Entity, &Name)>();
        q.iter(w).find(|(_, n)| n.0 == "Player").map(|(e, _)| e)
    }
    .expect("o Player da cena 120");
    let forma = ph2d_physics_ecs::main_shape(sim.world(), player).expect("a forma do Player");
    assert_eq!(
        ph2d_physics_ecs::shapes_in_order(sim.world(), player).len(),
        1
    );

    // A linha que o dono vê: a seguinte a «Player», um degrau mais funda.
    let mut live = crate::HeroLive {
        bridge: crate::hero_bridge::EntityNodeMap::new(),
        walk_state: ph2d_ecs::scene::HierarchyWalkState::new(sim.world_mut()),
        walk_scratch: Vec::new(),
        snapshot: ph2d_ecs::scene::HierarchySnapshot::new(),
        z_walk_state: ph2d_ecs::scene::HierarchyWalkState::new(sim.world_mut()),
        z_walk_scratch: Vec::new(),
        z_snapshot: ph2d_ecs::scene::HierarchySnapshot::new(),
    };
    ph2d_ecs::scene::build_hierarchy_snapshot(
        sim.world(),
        &mut live.walk_state,
        &mut live.walk_scratch,
        &mut live.snapshot,
    );
    let (ordem, linhas) = live.bridge.sync_from_snapshot(&live.snapshot);
    let i = ordem
        .iter()
        .position(|id| linhas[id].name == "Player")
        .expect("a linha Player");
    let (pai, row) = (&linhas[&ordem[i]], ordem[i + 1]);
    let linha = &linhas[&row];
    eprintln!(
        "linha abaixo de Player: {:?} (indent {} sob {})",
        linha.name, linha.indent, pai.indent
    );
    assert_eq!(
        linha.indent,
        pai.indent + 1,
        "a linha abaixo é filha do Player"
    );
    assert!(linha.name.starts_with("Collision Shape"), "{}", linha.name);
    assert_eq!(
        live.bridge.entity_for(row),
        Some(forma.to_bits()),
        "hipótese (a): a ponte resolve a linha da forma para outra entidade"
    );

    let mut pd = DrainOut::default();
    let a = EditorAction::Hierarchy(HierRequest::ToggleVisibility { row });
    assert!(app.fase_bus_hierarchy_tree(a, &mut pd).is_none());
    let hero_live = Some(live);
    let mut hero = ph2d_editor_core::HeroScreen::new(NodeId(1));
    let reg = ph2d_ecs::scene::ComponentRegistry::new();
    let _ = crate::render_loop::hierarchy::dispatch(
        std::mem::take(&mut pd.visibility_toggle_row),
        std::mem::take(&mut pd.lock_toggle_row),
        std::mem::take(&mut pd.group_toggle_row),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        &std::collections::BTreeMap::new(),
        None,
        None,
        None,
        None,
        None,
        &mut hero,
        &hero_live,
        &mut sim,
        &mut ph2d_render::Camera2d::default(),
        &mut ph2d_editor_core::ToastQueue::default(),
        ph2d_host::WindowSize {
            width: 800,
            height: 600,
        },
        &mut ph2d_vec_scene::VecScene::new(),
        &mut ph2d_vec_entities::entities::VecEntityMap::new(),
        &mut ph2d_vec_edit::PenTool::default(),
        &mut None,
        &reg,
        &mut ph2d_app_components::instance_sync::MasterEcho::default(),
    );
    let w = sim.world();
    let fechado = |e| w.get::<Visibility>(e).is_some_and(|v| v.hidden);
    eprintln!(
        "olho fechado: forma {} · Player {}",
        fechado(forma),
        fechado(player)
    );
    assert!(fechado(forma), "o olho não fechou a forma");
    assert!(!fechado(player), "o olho escreveu no corpo");
    assert_eq!(ph2d_physics_ecs::main_shape(w, player), None);

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
    let (_, _, pintado) = inspector_da_fisica(info, player.to_bits());
    sai_do_inspector();
    let tem = |id| pintado.iter().any(|(i, r)| *i == id && r.w > 0.0);
    use ph2d_panel_inspector::ids::{INSP_PHYS_ADD, INSP_PHYS_ADD_SHAPE, INSP_PHYS_KIND};
    let (corpo, sem_forma, vazio) = (
        INSP_PHYS_KIND.iter().all(|&k| tem(k)),
        tem(INSP_PHYS_ADD_SHAPE),
        tem(INSP_PHYS_ADD),
    );
    eprintln!(
        "Player y {y0:.3} -> {y1:.3} em 90 tiques; no solver {}; cartão Physics Body (Kind) {corpo} · \
         «No shape» {sem_forma} · «Add physics» {vazio}",
        bridge.body_pose(player).is_some()
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
        corpo && !sem_forma && !vazio,
        "o cartão Physics Body sumiu do Inspector"
    );
}

/// O Inspector em cartões pintado com esta §11 (dobras abertas, 60 quadros para assentar): o host, o estado e os
/// rects pintados. Quem chama fecha com [`sai_do_inspector`] (os retratos são `thread_local`).
#[cfg(feature = "panel-inspector")]
pub(in crate::render_loop) fn inspector_da_fisica(
    info: ph2d_editor_core::InspectorPhysicsInfo,
    bits: u64,
) -> (
    ph2d_ui_testkit::MockPanelHost,
    ph2d_panel_inspector::InspectorState,
    Vec<(NodeId, ph2d_editor_core::zones::Rect)>,
) {
    use ph2d_editor_core::panel::PanelHostInternal;
    use ph2d_editor_core::screens::layout::{CenterSplit, ChromeBands, DockSides, HeroLayout};
    use ph2d_panel_inspector::ids::{CARD_PHYS_LAYERS, CARD_PHYS_SHAPE, INSP_PHYS_FOLDS};
    use ph2d_panel_inspector::{InspectorPanel, InspectorState};
    ph2d_panel_inspector::set_cards_on(true);
    ph2d_panel_inspector::set_current_inspector_sprite(None);
    ph2d_panel_inspector::set_current_inspector_physics(Some(info));
    ph2d_panel_inspector::set_current_inspector_transform(Some(
        ph2d_editor_core::screens::hero::InspectorTransformInfo {
            entity_bits: bits,
            translation: [0.0, 0.0],
            rotation_rad: 0.0,
            scale: [1.0, 1.0],
            skew_rad: [0.0, 0.0],
        },
    ));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<InspectorPanel>();
    for id in INSP_PHYS_FOLDS.into_iter().chain([
        ph2d_editor_core::ids::INSP_LIVE_PHYSICS_SECTION,
        CARD_PHYS_SHAPE,
        CARD_PHYS_LAYERS,
    ]) {
        host.store_mut().set_collapsed(id, false);
    }
    let mut st = InspectorState::default();
    let janela = ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1600.0, 3200.0);
    let layout = || {
        HeroLayout::for_viewport_bands(
            janela,
            false,
            ChromeBands::DEFAULT,
            CenterSplit::None,
            DockSides::BOTH,
        )
    };
    let _ = host.paint_with_layout::<InspectorPanel>(&mut st, layout(), janela);
    host.settle_section_folds();
    let mut r = Vec::new();
    for _ in 0..60 {
        r = ph2d_editor_core::aquecimento::com_passo_fixo(
            ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
            || host.paint_with_layout::<InspectorPanel>(&mut st, layout(), janela),
        );
    }
    (host, st, r)
}

/// Desarma os retratos que [`inspector_da_fisica`] armou.
#[cfg(feature = "panel-inspector")]
pub(in crate::render_loop) fn sai_do_inspector() {
    ph2d_panel_inspector::set_cards_on(false);
    ph2d_panel_inspector::set_current_inspector_physics(None);
    ph2d_panel_inspector::set_current_inspector_transform(None);
}

/// O clique do ponteiro no centro de `r`, pela porta do painel; devolve o que chegou ao barramento.
#[cfg(feature = "panel-inspector")]
pub(in crate::render_loop) fn clica_no_inspector(
    host: &mut ph2d_ui_testkit::MockPanelHost,
    st: &mut ph2d_panel_inspector::InspectorState,
    r: ph2d_editor_core::zones::Rect,
) -> Vec<EditorAction> {
    use ph2d_host::{PointerButton, PointerEvent, PointerKind};
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let _ = host.drained_actions();
    let mut eventos = Vec::new();
    for kind in [PointerKind::Move, PointerKind::Down, PointerKind::Up] {
        eventos.extend(host.dispatch_pointer_event(PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: ph2d_host::PointerSource::Mouse,
            button: PointerButton::Primary,
            timestamp_ns: 0,
        }));
    }
    for ev in eventos {
        let _ = host.apply_panel_event::<ph2d_panel_inspector::InspectorPanel>(st, ev);
    }
    host.drained_actions()
}
