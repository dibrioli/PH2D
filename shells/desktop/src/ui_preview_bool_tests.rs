//! **A BOOLEANA e o modo de PREVIEW** — irmão dos gates do preview pelo teto de 600 LOC do HR-18,
//! e o corte é por ASSUNTO: ali mora *entrar captura, sair devolve o mundo ao bit*; aqui, a única
//! escrita que uma pose faz **fora do próprio id**.

use super::render_loop::ui_preview::UiPreview;
use super::render_loop::ui_state_bridge::UiMachines;
use crate::vec_entities::VecEntityMap;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_ui_state::{ObjectPose, StateRole, StateSets, UiState};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, rectangle};

const HOST: VecPathId = 1;

/// ⭐ **A OPERAÇÃO DO GRUPO VOLTA AO SAIR DA PREVIEW** (auditoria de 2026-08-23).
///
/// ⚠️ **Uma pose escreve FORA do próprio id.** O canal `bool_group_op` faz o `install` escrever o
/// `VecBoolGroup` de uma entidade de GRUPO, que não tem `VecPathId` — então ela **não pode** estar
/// no conjunto capturado, e o gate irmão (`the_snapshot_covers_every_id_the_preview_can_write`)
/// itera ids: ele fica verde sobre uma escrita que não vê.
///
/// O que se afirma aqui é a volta, e ela é por REDUNDÂNCIA: a pose capturada de cada operando
/// carrega a operação do grupo, então reinstalá-la devolve o grupo. **Cobrir** e **conter** não são
/// a mesma coisa, e é a primeira que interessa ao artista.
///
/// ⚠️ Fixture própria, com um grupo booleano de verdade pendurado no hospedeiro: a do irmão não
/// tem grupo nenhum, e sobre ela este gate não teria o que afirmar.
#[test]
fn the_group_operation_comes_back_when_the_preview_leaves() {
    use ph2d_ecs::{Entity, VecBoolGroup};

    let mut sim = SimWorld::default();
    let mut scene = VecScene::default();
    let mut map = VecEntityMap::default();
    for (id, ext) in [(HOST, 9.0f64), (2, 2.0), (3, 1.0)] {
        let mut p: VecPath = rectangle([0.0, 0.0], [ext, ext]);
        p.id = id;
        scene.push_path(p);
        let e = sim
            .world_mut()
            .spawn((
                Name(format!("p{id}")),
                Transform::IDENTITY,
                ph2d_ecs::VecPathRef(id),
            ))
            .id();
        map.insert(id, e.to_bits());
    }
    let group = Entity::from_bits(
        crate::vec_entities::group_entities(&mut sim, &[map[&2], map[&3]], "Bool".into()).unwrap(),
    );
    sim.world_mut()
        .entity_mut(group)
        .insert(VecBoolGroup { op: 0 }); // o mundo está em Union
    crate::vec_transform::reparent_keeping_world(&mut sim, group, Entity::from_bits(map[&HOST]));

    // O Hover autora o grupo em Subtract; o Default, em Union.
    let mut states = StateSets::default();
    for (role, op) in [(StateRole::Default, 0u8), (StateRole::Hover, 1)] {
        let mut st = UiState::new(role);
        st.objects = [2, 3]
            .into_iter()
            .map(|id| ObjectPose {
                bool_group_op: Some(op),
                ..ObjectPose::new(id)
            })
            .collect();
        states.set(HOST, st);
    }

    let mut machines = UiMachines::new();
    let mut pv = UiPreview::default();
    assert!(pv.enter(&mut machines, &states, &mut sim, &mut scene, &map));

    // O rato entra: a máquina vai ao Hover e o grupo passa a Subtract.
    pv.point(&mut machines, &states, &[HOST], false);
    machines.get_mut(&HOST).unwrap().advance(10.0);
    for p in machines[&HOST].pose() {
        crate::vec_ui_state_edit::install(&mut sim, &mut scene, &map, p);
    }
    assert_eq!(
        sim.world().get::<VecBoolGroup>(group).map(|g| g.op),
        Some(1),
        "a fixture não chegou a mexer no grupo: o gate mediria a volta de nada"
    );

    assert!(pv.leave(&mut machines, &mut sim, &mut scene, &map));
    assert_eq!(
        sim.world().get::<VecBoolGroup>(group).map(|g| g.op),
        Some(0),
        "sair da preview deixou a booleana do artista noutra operação: o documento mudou por ele \
         ter olhado"
    );
}
