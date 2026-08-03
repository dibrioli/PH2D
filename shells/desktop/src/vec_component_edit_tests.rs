//! Gates dos VERBOS de componente (plano UI/UX W5) — a costura painel↔shell↔ECS.
//!
//! Os gates do produtor provam o que se DESENHA; estes provam que o gesto leva a algum lugar — a
//! quarta condição da política de UI (*a SEQUÊNCIA leva a algum lugar*), que não é implicada por
//! nenhuma das outras três.

use super::*;
use ph2d_ecs::{OverrideSlot, VecComponentMain, VecInstance};
use ph2d_vec_scene::rectangle;

/// Uma cena com uma forma, já sincronizada.
fn one_shape() -> (SimWorld, VecScene, VecEntityMap, VecPathId) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(rectangle([0.0, 0.0], [20.0, 10.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    (sim, scene, map, id)
}

/// **A sequência COMPLETA leva a algum lugar**: criar → colocar → a cópia desenha o mestre.
///
/// ⚠️ É o gate da wave. Cada metade sozinha (o componente existe · o botão está registado · o
/// clique chega ao barramento) pode estar verde com o gesto a não produzir nada.
#[test]
fn create_then_place_gives_a_copy_that_draws_the_main() {
    let (mut sim, mut scene, mut map, main_id) = one_shape();
    assert!(create_main(&mut sim, &map, &[main_id]), "Create recusou");
    let new_id = place_instance(&sim, &mut scene, &map, &[main_id]).expect("Place recusou");
    // O `sync` é o que dá entidade ao caminho novo — o mesmo do frame do produto.
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    assert!(arm_instance(
        &mut sim,
        &map,
        new_id,
        main_id,
        place_offset()
    ));
    let xf = crate::vec_transform::build(&sim, &map);
    let mut live = crate::instance_live::InstanceLive::default();
    live.recook(&scene, &sim, &map, &xf);
    let items = live.live().get(&new_id).expect("a cópia desenha o mestre");
    assert_eq!(items.len(), 1, "um mestre de uma peça desenha uma");
}

/// **Uma instância não vira mestre** — o que ela mostra é derivado.
#[test]
fn an_instance_cannot_be_promoted_to_a_main() {
    let (mut sim, _scene, map, id) = one_shape();
    let e = Entity::from_bits(map[&id]);
    sim.world_mut().entity_mut(e).insert(VecInstance::new(999));
    assert!(!create_main(&mut sim, &map, &[id]));
}

/// **Place recusa quem não é mestre** — senão o botão faria uma cópia de uma coisa qualquer.
#[test]
fn place_refuses_a_plain_shape() {
    let (sim, mut scene, map, id) = one_shape();
    assert!(place_instance(&sim, &mut scene, &map, &[id]).is_none());
}

/// **Detach materializa TODAS as peças** — e as parenteia sob a que ficou.
///
/// ⚠️ O gate nasceu contra a v1, que escrevia só a primeira: com um componente de duas peças ela
/// deixava metade da arte para trás **sem erro nenhum**, que é o modo de falha que este ficheiro
/// existe para impedir.
#[test]
fn detach_materialises_every_piece_and_keeps_them_together() {
    let (mut sim, mut scene, mut map, id) = one_shape();
    let e = Entity::from_bits(map[&id]);
    sim.world_mut().entity_mut(e).insert(VecInstance::new(7));
    let drawn = vec![
        rectangle([0.0, 0.0], [4.0, 4.0]),
        rectangle([5.0, 5.0], [9.0, 9.0]),
    ];
    let (root, extra) =
        detach(&mut sim, &mut scene, &map, &[id], Some(&drawn)).expect("Detach recusou");
    assert_eq!(root, id, "a raiz mantém o id (e com ele o z e a seleção)");
    assert_eq!(extra.len(), 1, "a 2ª peça tem de virar caminho");
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    assert!(arm_detached(&mut sim, &map, root, &extra));
    let child = Entity::from_bits(map[&extra[0]]);
    assert_eq!(
        sim.world()
            .get::<ph2d_ecs::ChildOf>(child)
            .map(|c| c.parent()),
        Some(Entity::from_bits(map[&root])),
        "a peça ficou solta na raiz: o Detach desfez o agrupamento"
    );
    assert!(
        sim.world().get::<VecInstance>(e).is_none(),
        "o vínculo tinha de sair"
    );
}

/// **Reset num instância LIMPA não mexe no mundo** — o `post_frame_undo` regista por diff, e um
/// passo vazio é ruído que o artista paga com um Ctrl+Z que não faz nada.
#[test]
fn reset_on_a_clean_instance_is_a_no_op() {
    let (mut sim, _scene, map, id) = one_shape();
    let e = Entity::from_bits(map[&id]);
    sim.world_mut().entity_mut(e).insert(VecInstance::new(7));
    assert!(!reset_overrides(&mut sim, &map, &[id]));
    let mut with = VecInstance::new(7);
    with.set(1, OverrideSlot::Hidden);
    sim.world_mut().entity_mut(e).insert(with);
    assert!(reset_overrides(&mut sim, &map, &[id]));
}

/// **O painel recebe os verbos que fazem sentido, e só eles.**
#[test]
fn the_panel_is_told_which_verbs_make_sense() {
    let (mut sim, _scene, map, id) = one_shape();
    let plain = selected_component(&sim, &map, &[id], &[]).expect("uma forma tem seção");
    assert!(!plain.is_main && !plain.is_instance && !plain.has_overrides);

    let e = Entity::from_bits(map[&id]);
    sim.world_mut().entity_mut(e).insert(VecComponentMain);
    assert!(
        selected_component(&sim, &map, &[id], &[])
            .expect("mestre")
            .is_main
    );

    sim.world_mut().entity_mut(e).remove::<VecComponentMain>();
    let mut inst = VecInstance::new(7);
    inst.set(1, OverrideSlot::Hidden);
    sim.world_mut().entity_mut(e).insert(inst);
    let s = selected_component(&sim, &map, &[id], &[id]).expect("instância");
    assert!(s.is_instance && s.has_overrides && s.main_missing);
}

/// **Sem seleção — ou com duas — a seção não é oferecida.** Um prefab é sobre UMA coisa.
#[test]
fn no_section_without_exactly_one_selected_shape() {
    let (sim, mut scene, mut map, id) = one_shape();
    assert!(selected_component(&sim, &map, &[], &[]).is_none());
    let other = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    let mut sim2 = sim;
    crate::vec_entities::sync(&mut sim2, &mut scene, &mut map);
    assert!(selected_component(&sim2, &map, &[id, other], &[]).is_none());
}

/// **A órfã vem do PRODUTOR** — o painel lê a resposta dele, não uma segunda.
///
/// ⚠️ Sem este gate, alguém "simplificaria" o `selected_component` para perguntar *"o mestre
/// existe?"* por conta própria — e o painel diria que está tudo bem no frame em que o produtor
/// recusasse por pose degenerada ou por laço.
#[test]
fn the_missing_readout_comes_from_the_producers_answer() {
    let (mut sim, _scene, map, id) = one_shape();
    let e = Entity::from_bits(map[&id]);
    // Um mestre que EXISTE — mas o produtor, por outra razão, recusou.
    sim.world_mut().entity_mut(e).insert(VecInstance::new(id));
    let s = selected_component(&sim, &map, &[id], &[id]).expect("instância");
    assert!(s.main_missing, "o painel ignorou a recusa do produtor");
}
