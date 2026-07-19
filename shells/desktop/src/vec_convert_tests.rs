//! Seam de **"Convert to Curves"** — prova, headless e do jeito que o shell o percorre, que o
//! converter ASSA a pilha de efeitos (a metade que faltava; Enio 2026-07-19: *"Convert to
//! Curves não funciona para isso"*). Sem este teste a fiação ficava verde nos unit tests do
//! motor e MORTA no produto.

use super::*;
use crate::vec_entities::VecEntityMap;
use ph2d_vec_scene::{VecPath, VecVertex};

/// Um quadrado sincronizado numa entidade, com um Zig Zag ATIVO adicionado pelo MESMO caminho
/// do produto (`fx_bridge`) — não um `PathEffect` fabricado à mão.
fn scene_with_effect_path() -> (SimWorld, VecScene, VecEntityMap, VecPathId) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [40.0, 0.0], [40.0, 40.0], [0.0, 40.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    // Um Zig Zag levado ao máximo do 1º parâmetro — ativo, muda a geometria de facto.
    crate::fx_bridge::add(&mut scene, id, 1);
    crate::fx_bridge::set_param(&mut scene, id, 0, 0, 1.0);
    (sim, scene, map, id)
}

/// **Convert to Curves ASSA a pilha de efeitos** de um caminho SEM forma viva (`VecShape`) — o
/// caso que não funcionava, porque o botão só olhava para o shape. Depois de converter, a pilha
/// está vazia e a geometria autorada é a cozida.
#[test]
fn convert_to_curves_bakes_the_effect_stack() {
    let (mut sim, mut scene, mut map, id) = scene_with_effect_path();
    // A aparência (cozida) e a fonte (autorada) DIFEREM — o efeito está mesmo ativo.
    let cooked = scene.path(id).unwrap().cooked().into_owned();
    assert_ne!(
        cooked.verts,
        scene.path(id).unwrap().verts,
        "pré-condição: o Zig Zag tem de mudar a geometria, senão o bake não prova nada"
    );

    let new_sel = crate::vec_convert::to_curves(&mut sim, &mut scene, &mut map, &[id]);

    assert!(
        new_sel.contains(&id),
        "um caminho sem forma viva sobrevive à conversão (fica intacto, só perde os efeitos)"
    );
    let p = scene.path(id).unwrap();
    assert!(
        p.effects.is_empty(),
        "a pilha de efeitos tem de sair VAZIA — é o que Convert to Curves passou a fazer"
    );
    assert_eq!(
        p.verts, cooked.verts,
        "a geometria autorada virou a cozida (Expand Appearance)"
    );
}

/// Um quadrado sincronizado numa entidade, cru. O `decorate` pendura nela o host vivo do caso.
fn square_entity() -> (SimWorld, VecScene, VecEntityMap, VecPathId) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [40.0, 0.0], [40.0, 40.0], [0.0, 40.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    (sim, scene, map, id)
}

fn entity_of(map: &VecEntityMap, id: VecPathId) -> Entity {
    Entity::from_bits(*map.get(&id).expect("path sincronizado"))
}

/// **TODA fonte de geometria viva torna o caminho convertível — e um caminho cru NÃO.**
///
/// É o gate que existe para a próxima fonte: o `convertible` já apodreceu DUAS vezes por
/// enumerar (ficou desligado num caminho só-efeitos, depois num só-quinas), sempre em silêncio.
/// Quem acrescentar uma fonte nova e não a ensinar à porta única quebra aqui.
/// [[feedback_a_condition_that_enumerates_its_readers_rots]]
#[test]
fn every_live_source_makes_the_path_convertible() {
    // O controle POSITIVO: sem nada vivo, o botão NÃO se oferece. Sem esta metade, "é
    // convertível" ficaria verde com a função a devolver `true` para tudo.
    let (sim, scene, map, id) = square_entity();
    assert!(
        !crate::vec_convert::is_convertible(&sim, &map, &scene, id),
        "um caminho CRU não tem o que congelar — o botão não pode se oferecer"
    );

    // 1. Quina viva (ADR-0121) — a fonte que o Enio reportou desligada.
    let (sim, mut scene, map, id) = square_entity();
    scene.path_mut(id).unwrap().verts[1].corner_radius = 8.0;
    assert!(
        crate::vec_convert::is_convertible(&sim, &map, &scene, id),
        "uma quina com raio é geometria viva: Convert to Curves a congela"
    );

    // 2. Pilha de efeitos (ADR-0132).
    let (sim, mut scene, map, id) = square_entity();
    crate::fx_bridge::add(&mut scene, id, 1);
    assert!(crate::vec_convert::is_convertible(&sim, &map, &scene, id));

    // 3. Forma paramétrica — a receita que o `recook_into` reescreve.
    let (mut sim, scene, map, id) = square_entity();
    sim.world_mut()
        .entity_mut(entity_of(&map, id))
        .insert(VecShape::Param {
            kind: 0,
            w: 40.0,
            h: 40.0,
            values: [0.0; ph2d_ecs::MAX_SHAPE_VALUES],
        });
    assert!(crate::vec_convert::is_convertible(&sim, &map, &scene, id));

    // 4. Conector — a rota é derivada da relação.
    let (mut sim, scene, map, id) = square_entity();
    sim.world_mut()
        .entity_mut(entity_of(&map, id))
        .insert(VecConnector::between(1, 2));
    assert!(crate::vec_convert::is_convertible(&sim, &map, &scene, id));

    // 5. Morph — a geometria é função do `t`.
    let (mut sim, scene, map, id) = square_entity();
    sim.world_mut()
        .entity_mut(entity_of(&map, id))
        .insert(VecMorph::new(1, 2));
    assert!(crate::vec_convert::is_convertible(&sim, &map, &scene, id));
}

/// **Convert to Curves ASSA a quina viva** — o bug que o Enio reportou (*"o botão convert to
/// curves fica indisponível para formas modificadas com essas tool"*). Depois de converter, o
/// arredondamento está nos VÉRTICES e não sobra raio nenhum: a operação é IDEMPOTENTE, então o
/// botão se apaga sozinho em vez de ficar aceso para sempre sobre um caminho já assado.
#[test]
fn convert_to_curves_bakes_a_live_corner() {
    let (mut sim, mut scene, mut map, id) = square_entity();
    scene.path_mut(id).unwrap().verts[1].corner_radius = 8.0;
    let cooked = scene.path(id).unwrap().cooked().into_owned();
    assert_ne!(
        cooked.verts,
        scene.path(id).unwrap().verts,
        "pré-condição: a quina tem de arredondar de facto"
    );

    crate::vec_convert::to_curves(&mut sim, &mut scene, &mut map, &[id]);

    let p = scene.path(id).unwrap();
    assert_eq!(p.verts, cooked.verts, "a geometria autorada virou a cozida");
    assert!(
        !p.has_live_geometry(),
        "nada vivo sobra — o bake é idempotente e o botão se apaga"
    );
    assert!(
        !crate::vec_convert::is_convertible(&sim, &map, &scene, id),
        "e a porta única concorda: não há mais o que congelar"
    );
}

/// **Os hosts de RELAÇÃO são soltos** — conector e morph deixam de reescrever a geometria, que
/// congela onde está. Sem isto o Convert to Curves não alcançava nenhum dos dois (*"nao funciona
/// com quase nada"*).
#[test]
fn convert_to_curves_drops_the_relation_hosts() {
    for (name, decorate) in [
        (
            "conector",
            Box::new(|sim: &mut SimWorld, e: Entity| {
                sim.world_mut().entity_mut(e).insert(VecConnector::between(1, 2));
            }) as Box<dyn Fn(&mut SimWorld, Entity)>,
        ),
        (
            "morph",
            Box::new(|sim: &mut SimWorld, e: Entity| {
                sim.world_mut().entity_mut(e).insert(VecMorph::new(1, 2));
            }),
        ),
    ] {
        let (mut sim, mut scene, mut map, id) = square_entity();
        let e = entity_of(&map, id);
        decorate(&mut sim, e);
        assert!(crate::vec_convert::is_convertible(&sim, &map, &scene, id));

        crate::vec_convert::to_curves(&mut sim, &mut scene, &mut map, &[id]);

        assert!(
            !crate::vec_convert::is_convertible(&sim, &map, &scene, id),
            "{name}: a relação foi solta — a geometria congelou onde estava"
        );
    }
}

/// **A ferramenta de quina congela só a RECEITA de uma forma paramétrica.** É o que faz o
/// Fillet/Chamfer funcionar no vértice de uma Shape (Enio: *"nao funciona diretamente nos vertex
/// das shapes"*) sem que o raio seja varrido pelo recook no frame seguinte.
///
/// E **só ela**: um caminho cru não tem receita a largar, e o TEXTO não — congelá-lo é explodir
/// em glyphs, que é outra operação (com seleção nova).
#[test]
fn freezing_a_shape_recipe_unblocks_the_corner_tools_and_touches_nothing_else() {
    let (mut sim, _scene, map, id) = square_entity();
    let e = entity_of(&map, id);
    sim.world_mut().entity_mut(e).insert(VecShape::Param {
        kind: 0,
        w: 40.0,
        h: 40.0,
        values: [0.0; ph2d_ecs::MAX_SHAPE_VALUES],
    });
    assert!(
        crate::corner_handles::has_derived_verts(&sim, &map, id),
        "pré-condição: a forma viva é derivada, e por isso a quina era RECUSADA"
    );

    assert!(crate::vec_convert::freeze_shape_recipe(&mut sim, &map, id));

    assert!(
        !crate::corner_handles::has_derived_verts(&sim, &map, id),
        "congelada a receita, a quina passa a ser editável — é o desbloqueio inteiro"
    );

    // Um caminho CRU não tem receita: congelar é um no-op honesto (nada a largar).
    let (mut sim, _scene, map, id) = square_entity();
    assert!(!crate::vec_convert::freeze_shape_recipe(&mut sim, &map, id));
}

/// Convert to Curves num caminho SEM efeitos é um no-op para a pilha (não há o que assar) e não
/// derruba o caminho — a conversão continua a servir os outros casos (texto/forma) sem estragar
/// um caminho cru.
#[test]
fn convert_to_curves_leaves_a_plain_path_alone() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [40.0, 0.0], [40.0, 40.0], [0.0, 40.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let before = scene.path(id).unwrap().clone();

    let new_sel = crate::vec_convert::to_curves(&mut sim, &mut scene, &mut map, &[id]);

    assert!(new_sel.contains(&id));
    assert_eq!(
        &before,
        scene.path(id).unwrap(),
        "sem forma viva nem efeitos, converter não pode mexer no caminho"
    );
}
