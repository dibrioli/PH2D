//! **A BOOLEANA VIVA numa pose de estado de UI** — irmão do [`super::tests`] pelo teto de LOC, e o
//! corte é por assunto: aqui mora só a costura dos dois canais que entraram em 2026-08-23.
//!
//! ⚠️ **Fixture PRÓPRIA**, com um grupo booleano de verdade: os dois canais falam de *"que verbo
//! esta forma manda"* e *"em que operação ela está metida"*, e a segunda pergunta não tem resposta
//! nenhuma numa cena sem grupo. Um gate que herdasse a fixture de uma forma solta ficaria verde
//! sobre um `None` que não prova coisa alguma.

use super::*;
use ph2d_ecs::{Entity, SimWorld, VecBoolGroup, VecBoolOp};
use ph2d_vec_scene::{VecScene, rectangle};

/// Dois retângulos sobrepostos, agrupados, com o grupo em `op`. Devolve `(sim, scene, map, ids em
/// z, grupo)` — a mesma cena mínima que os gates do cozimento usam.
fn scene_with_group(op: u8) -> (SimWorld, VecScene, VecEntityMap, Vec<VecPathId>, Entity) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let a = scene.push_path(rectangle([0.0, 0.0], [2.0, 2.0]));
    let b = scene.push_path(rectangle([1.0, 1.0], [3.0, 3.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let g = Entity::from_bits(
        crate::vec_entities::group_entities(&mut sim, &[map[&a], map[&b]], "Bool".into()).unwrap(),
    );
    sim.world_mut().entity_mut(g).insert(VecBoolGroup { op });
    (sim, scene, map, vec![a, b], g)
}

/// **A COSTURA dos dois canais: o que a captura lê, a instalação escreve de volta.**
///
/// ⚠️ É o gate que impede a metade morta, e esta casa já pagou por ele duas vezes (a `geometry` e
/// os `filters` ficaram, cada uma, uma wave inteira com o motor pronto e sem produtor). Um campo
/// novo na pose sem produtor passa em **todos** os gates da `ph2d-ui-state`.
#[test]
fn both_boolean_channels_survive_a_capture_and_an_install() {
    let (mut sim, mut scene, map, ids, group) = scene_with_group(0); // o grupo em Union
    let cutter = Entity::from_bits(map[&ids[1]]);
    sim.world_mut()
        .entity_mut(cutter)
        .insert(VecBoolOp { op: 1 }); // ...e esta forma em Subtract

    let pose = capture(&sim, &scene, &map, ids[1]);
    assert_eq!(
        pose.bool_op,
        Some(1),
        "a captura não leu o verbo PRÓPRIO: gravar o estado perderia a escolha do artista"
    );
    assert_eq!(
        pose.bool_group_op,
        Some(0),
        "a captura não leu a operação do GRUPO: trocá-la num estado não animaria nada"
    );

    // Apaga os dois do mundo e reinstala a partir da pose — o caminho do Show.
    sim.world_mut().entity_mut(cutter).remove::<VecBoolOp>();
    sim.world_mut()
        .entity_mut(group)
        .insert(VecBoolGroup { op: 3 });
    install(&mut sim, &mut scene, &map, &pose);

    assert_eq!(
        sim.world().get::<VecBoolOp>(cutter).map(|v| v.op),
        Some(1),
        "o Show não devolveu o verbo próprio"
    );
    assert_eq!(
        sim.world().get::<VecBoolGroup>(group).map(|g| g.op),
        Some(0),
        "o Show não devolveu a operação do grupo"
    );
}

/// ⭐ **UM OPERANDO QUE HERDA grava `None` no verbo — e AINDA ASSIM grava o grupo.**
///
/// ⚠️ É a metade que faz o gesto mais natural do artista funcionar: ele seleciona a booleana e
/// clica `Subtract`, sem nunca abrir a fileira por forma. Nenhum operando tem override, então o
/// canal de cima é `None` nos dois estados — e se o canal do GRUPO não fosse gravado, aquele
/// clique não animaria coisa nenhuma, sem nada vermelho em lado nenhum.
///
/// ⚠️ E o `None` do verbo próprio é literal: traduzi-lo aqui para o verbo EFETIVO congelaria, no
/// arquivo, uma herança que o artista ainda pode mudar no grupo.
#[test]
fn an_inheriting_operand_records_none_but_still_records_the_group() {
    let (sim, scene, map, ids, _g) = scene_with_group(2); // Intersect, e ninguém se pronunciou
    let pose = capture(&sim, &scene, &map, ids[1]);
    assert_eq!(pose.bool_op, None, "a herança não é um verbo autorado");
    assert_eq!(
        pose.bool_group_op,
        Some(2),
        "sem o canal do grupo, trocar a operação da booleana num estado seria inanimável"
    );
}

/// **`None` no verbo próprio REMOVE o componente** — a lei do filtro e do `VecOffset`.
///
/// ⚠️ O efeito que importa não é a arrumação: sem a remoção, um estado que devolve a forma à
/// herança deixaria o override do OUTRO estado colado nela, e o grupo passaria a ter uma forma que
/// não obedece a ninguém.
#[test]
fn a_pose_without_a_verb_removes_the_override() {
    let (mut sim, mut scene, map, ids, _g) = scene_with_group(0);
    let cutter = Entity::from_bits(map[&ids[1]]);
    sim.world_mut()
        .entity_mut(cutter)
        .insert(VecBoolOp { op: 3 });

    let bare = ObjectPose {
        bool_group_op: Some(0),
        ..ObjectPose::new(ids[1])
    };
    install(&mut sim, &mut scene, &map, &bare);
    assert!(
        sim.world().get::<VecBoolOp>(cutter).is_none(),
        "o override do outro estado ficou colado na forma"
    );
}

/// ⛔ **UMA POSE QUE NÃO CONHECE GRUPO NENHUM NÃO DESFAZ O GRUPO.**
///
/// ⚠️ É a distinção entre *"não sei"* e *"não tem"*, e lê-la ao contrário destruiria a booleana do
/// artista no primeiro Show — qualquer pose gravada ANTES de ele criar o grupo tem `None` aqui.
/// O gate mora ao lado do de cima de propósito: os dois `None` significam coisas OPOSTAS, e é o
/// tipo de par que alguém uniformiza por simetria.
#[test]
fn a_pose_that_knows_no_group_never_destroys_one() {
    let (mut sim, mut scene, map, ids, group) = scene_with_group(1);
    let bare = ObjectPose::new(ids[1]); // gravada antes de a booleana existir
    install(&mut sim, &mut scene, &map, &bare);
    assert_eq!(
        sim.world().get::<VecBoolGroup>(group).map(|g| g.op),
        Some(1),
        "instalar uma pose sem grupo DESFEZ a booleana"
    );
}

/// **Uma forma fora de booleana nenhuma grava os dois canais vazios** — o controle da inércia.
///
/// ⚠️ Sem ele, um gate que apenas visse os canais preenchidos não distinguiria *"a captura lê o
/// grupo certo"* de *"a captura inventa um grupo"* — e a segunda faria toda forma solta do
/// documento nascer com uma operação booleana pendurada.
#[test]
fn a_shape_outside_any_boolean_records_nothing() {
    let (mut sim, mut scene, mut map, _ids, _g) = scene_with_group(0);
    let lone = scene.push_path(rectangle([9.0, 9.0], [10.0, 10.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let pose = capture(&sim, &scene, &map, lone);
    assert_eq!(pose.bool_op, None);
    assert_eq!(
        pose.bool_group_op, None,
        "uma forma fora da booleana ganhou uma operação que ninguém autorou"
    );
}
