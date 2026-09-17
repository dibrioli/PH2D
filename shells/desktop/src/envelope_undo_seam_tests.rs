//! **A COSTURA ENTRE O UNDO DA SHELL E O OVERLAY DO ENVELOPE.**
//!
//! ⚠️ **Este gate vivia na `ph2d-app-vec` e teve de FICAR na shell quando o cacho do envelope saiu**
//! (integração de 2026-09-17). Ele não é um gate do envelope: é um gate da **porta** entre dois
//! subsistemas — o `ProjectState` da shell (que fotografa e repõe o MUNDO INTEIRO: o mundo ECS, a
//! cena vectorial, o `FlipDoc`, as guias, os estados de UI e a biblioteca) e o desenho do overlay,
//! que é da família. *Um gate que atravessa uma porta mora com o que ele exercita* (HOWTO §2.6), e
//! a `ph2d-app-vec` não pode — nem deve — conhecer o undo da shell.
//!
//! ⚠️ **A fixtura é construída pela API PÚBLICA da crate, não copiada dela.** Os quatro auxiliares
//! que a versão de lá usava (`envelope_over`, `pins_mode`, `set_pins`, `pen_with`) são cada um duas
//! ou três linhas sobre portas públicas; o que **não** se duplica é a LEI — a criação do container
//! continua a ter um só dono (`ph2d_app_vec::envelope_live::create`), e é ele que este teste chama.

use ph2d_ecs::{Entity, SimWorld, VecEnvelope};
use ph2d_vec_entities::entities::VecEntityMap;
use ph2d_vec_scene::{VecPathId, VecScene};

/// Um envelope sobre `shapes`, com o `sync` que dá entidade a cada forma.
fn envelope_over(shapes: Vec<ph2d_vec_scene::VecPath>) -> (SimWorld, VecScene, VecEntityMap, u64) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let ids: Vec<VecPathId> = shapes.into_iter().map(|s| scene.push_path(s)).collect();
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
    let container = ph2d_app_vec::envelope_live::create(&mut sim, &mut scene, &map, &ids)
        .expect("create");
    (sim, scene, map, container)
}

/// **O UNDO NÃO PODE FAZER A GAIOLA/OS PINOS SUMIREM** (Enio, smoke da Fatia E).
///
/// O `ProjectState::restore` **despawna e re-spawna** o mundo: *"ids do mundo são novos"*. O recook
/// sobrevive porque varre por QUERY, e por isso a arte continuava deformada — mas **o desenho do
/// overlay é indexado pelos bits da seleção do gizmo**, e esses bits morrem no respawn. Resultado:
/// a ferramenta funcionando e invisível.
///
/// A seleção do PEN é estável (é `VecPathId`, e o snapshot leva a geometria), então a resposta é
/// **re-derivar** os bits dela. Este gate percorre o caminho real: captura → restaura → reconstrói
/// a ponte → `sync_selection` → o overlay tem de voltar a apontar para uma entidade VIVA.
#[test]
fn the_pins_survive_an_undo() {
    use ph2d_ecs::scene::{ComponentRegistry, register_ecs_components};

    let (mut sim, mut scene, mut map, container) =
        envelope_over(vec![ph2d_vec_scene::ellipse([5.0, 5.0], 3.0, 3.0)]);

    // O envelope no gesto Pinos, com um pino pregado — o que uma sessão de cliques produziria.
    ph2d_app_vec::envelope_gesture::set_kind(
        &mut sim,
        container,
        ph2d_ecs::EnvelopeKind::Pins,
    );
    sim.world_mut()
        .get_mut::<VecEnvelope>(Entity::from_bits(container))
        .expect("VecEnvelope")
        .pins = vec![[[4.0, 4.0], [4.5, 4.5]]];

    // A seleção como o produto a tem: o pen com os FILHOS, o gizmo com o container.
    let filhos: Vec<VecPathId> = scene.paths().iter().map(|p| p.id).collect();
    let mut pen = ph2d_vec_edit::PenTool::default();
    pen.select_many(&filhos);
    let mut gizmo = ph2d_editor_core::screens::hero::GizmoStateGroup::default();
    let mut sel = ph2d_app_vec::selection_sync::VecSelSync::default();
    crate::vec_selection::sync_selection(&mut gizmo, &sim, &scene, &map, &mut pen, &mut sel, true);
    assert_eq!(
        gizmo.selection,
        Some(container),
        "fixture morto: o gizmo devia estar no container"
    );
    assert_eq!(
        ph2d_app_vec::envelope_gesture::pins_world(&sim, container).len(),
        1,
        "fixture morto: o pino devia estar desenhável"
    );

    // O undo, pelo caminho REAL: captura, mexe, restaura (ids do mundo ficam NOVOS).
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    let snap = crate::undo::ProjectState::capture(
        // Nada sob condução nesta cena: o ledger vazio é a captura de sempre.
        &ph2d_preview_drive::PreviewDrive::default(),
        &mut sim,
        &scene,
        &ph2d_flip::FlipDoc::new(),
        &ph2d_guides::GuideSet::default(),
        &ph2d_ui_state::StateSets::default(),
        &crate::project_library::LibraryDoc::default(),
        &[],
        &reg,
        &mut ph2d_ecs::scene::incremental::CaptureCache::new(),
        None,
    );
    let (restored_scene, restored_map, _flip, _fm) = snap.restore(&mut sim, &reg);
    scene = restored_scene;
    map = restored_map;

    // O frame seguinte re-sincroniza a seleção — e é AQUI que os bits mortos têm de ser trocados.
    crate::vec_selection::sync_selection(&mut gizmo, &sim, &scene, &map, &mut pen, &mut sel, true);

    let live = gizmo.selection.expect("o gizmo perdeu a seleção no undo");
    assert!(
        sim.world().get_entity(Entity::from_bits(live)).is_ok(),
        "o gizmo ficou com bits de uma entidade MORTA — o overlay não desenha nada"
    );
    assert_eq!(
        ph2d_app_vec::envelope_gesture::pins_world(&sim, live).len(),
        1,
        "o overlay perdeu o pino depois do undo"
    );
}
