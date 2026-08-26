//! ⭐⭐⭐ **O conjunto de Morph States DENTRO de uma animação de States** (plano 32 W11c) — irmão
//! de [`super`] pelo teto de 600 LOC, e o corte é por ASSUNTO: ali *o que o conjunto faz às formas
//! por si*; aqui *o que o sistema de States, que já existia, consegue fazer com ele*.
//!
//! Enio, 2026-08-26: *"que eu possa usar o state morph nas animações criadas em States."*
//!
//! ⚠️ Submódulo do irmão de propósito: o harness (`world`) é **um só**.

use super::super::{create, upkeep};
use super::world;
use ph2d_ecs::{Entity, SimWorld, VecMorph};
use ph2d_vec_scene::{VecPath, VecScene};

use crate::vec_entities::{VecEntityMap, sync};

/// ⭐⭐⭐ **UMA POSE DE UI GRAVA EM QUE FORMA O CONJUNTO ESTÁ** (plano 32 W11c).
///
/// Enio, 2026-08-26: *"que eu possa usar o state morph nas animações criadas em States."*
///
/// ⚠️ **A forma que a cena MOSTRA é `sources[1]`** — o destino do último voo —, e não `sources[0]`:
/// `t = 1` no par `(A, B)` já **é** a forma B. Gravar a origem faria o `Hover` capturar a forma de
/// onde a máquina veio.
///
/// **Mutação que deve sangrar:** o `capture` gravar `sources[0]`.
#[test]
fn a_ui_pose_records_which_shape_the_set_is_showing() {
    let (mut sim, mut scene, mut map, ids) = world(3);
    let mut pending = create(&sim, &mut scene, &map, &ids, 9);
    sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);
    let host_id = scene.paths().last().unwrap().id;
    let host = Entity::from_bits(map[&host_id]);

    // A cena mostra a PRIMEIRA (o conjunto nasce em `[start, start]`).
    let p0 = crate::vec_ui_state_edit::capture(&sim, &scene, &map, host_id);
    assert_eq!(
        p0.morph_shape,
        Some(ids[0]),
        "a pose tem de gravar a forma que a cena MOSTRA"
    );

    // O motor leva-a à terceira: o par vira `(ids[0], ids[2])`.
    if let Some(mut m) = sim.world_mut().get_mut::<VecMorph>(host) {
        m.sources = [ids[0], ids[2]];
        m.t = 1.0;
    }
    let p1 = crate::vec_ui_state_edit::capture(&sim, &scene, &map, host_id);
    assert_eq!(
        p1.morph_shape,
        Some(ids[2]),
        "⛔ ela gravou a forma de ONDE a maquina veio, e nao a que se ve'"
    );
}

/// ⛔⛔ **UM MORPH SEM MÁQUINA grava `None`** — ele não é um conjunto de estados.
///
/// ⚠️ Um morph autorado à mão (dois operandos, `t` keyado pela timeline) não *está* numa forma:
/// dizer que está faria o `install` prendê-lo lá, **matando a curva** que a timeline conduz.
///
/// **Mutação que deve sangrar:** o `capture` largar a checagem do `VecMorphMachine`.
#[test]
fn a_hand_authored_morph_records_no_shape() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let a = scene.push_path(ph2d_vec_scene::rectangle([0.0, 0.0], [1.0, 1.0]));
    let b = scene.push_path(ph2d_vec_scene::rectangle([3.0, 0.0], [4.0, 1.0]));
    let m = scene.push_path(VecPath::default());
    sync(&mut sim, &mut scene, &mut map);
    // Um Morph COMUM: componente sem máquina, que é como o botão «Morph» o cria.
    sim.world_mut()
        .entity_mut(Entity::from_bits(map[&m]))
        .insert(VecMorph::new(a, b));

    assert_eq!(
        crate::vec_ui_state_edit::capture(&sim, &scene, &map, m).morph_shape,
        None,
        "um morph autorado a' mao nao ESTA' numa forma -- prende-lo mataria o `t` da timeline"
    );

    // ⛔⛔ **E O `install` TAMBÉM NÃO O TOCA** — a outra metade, e ela nasceu de uma mutação que
    // SOBREVIVEU (2026-08-26): a guarda do `VecMorphMachine` no `install` estava **ungated**, e
    // apagá-la deixava a suíte inteira verde.
    //
    // O dano: uma pose com `morph_shape` (gravada sobre um conjunto) instalada sobre um morph
    // autorado à mão prendê-lo-ia num par degenerado — **matando a curva** que a timeline conduz.
    let host = Entity::from_bits(map[&m]);
    let before = sim.world().get::<VecMorph>(host).unwrap().clone();
    let mut pose = crate::vec_ui_state_edit::capture(&sim, &scene, &map, m);
    pose.morph_shape = Some(b);
    crate::vec_ui_state_edit::install(&mut sim, &mut scene, &map, &pose);
    assert_eq!(
        sim.world().get::<VecMorph>(host).unwrap(),
        &before,
        "⛔ o install prendeu um morph autorado a' mao numa forma -- o `t` da timeline morre"
    );
}

/// ⭐⭐ **E O `install` DEVOLVE a forma** — a chegada põe o par em `(shape, shape)`, que é exacta.
///
/// **Mutação que deve sangrar (1):** o `install` não escrever nada — o `Hover` animaria e a
/// chegada deixaria a forma no penúltimo `t`.
///
/// **Mutação que deve sangrar (2):** ele escrever mesmo com `morph_shape == None` — uma pose
/// gravada antes de o objecto ser um conjunto passaria a mandá-lo para a primeira forma.
#[test]
fn installing_a_pose_puts_the_set_exactly_on_its_shape() {
    let (mut sim, mut scene, mut map, ids) = world(3);
    let mut pending = create(&sim, &mut scene, &map, &ids, 9);
    sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);
    let host_id = scene.paths().last().unwrap().id;
    let host = Entity::from_bits(map[&host_id]);

    let mut pose = crate::vec_ui_state_edit::capture(&sim, &scene, &map, host_id);
    pose.morph_shape = Some(ids[2]);
    crate::vec_ui_state_edit::install(&mut sim, &mut scene, &map, &pose);
    let m = sim.world().get::<VecMorph>(host).expect("o morph fica");
    assert_eq!(
        (m.sources, m.t),
        ([ids[2], ids[2]], 0.0),
        "a chegada tem de por o par na forma EXACTA"
    );

    // ⛔ E `None` NÃO escreve: ele é *"esta pose não se pronuncia"*.
    let before = sim.world().get::<VecMorph>(host).unwrap().clone();
    pose.morph_shape = None;
    crate::vec_ui_state_edit::install(&mut sim, &mut scene, &map, &pose);
    assert_eq!(
        sim.world().get::<VecMorph>(host).unwrap(),
        &before,
        "`None` e' «nao me pronuncio» -- escrever por causa dele poria uma pose antiga a mandar"
    );
}
