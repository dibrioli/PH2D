//! Seam de **"Convert to Curves"** — prova, headless e do jeito que o shell o percorre, que o
//! converter ASSA a pilha de efeitos (a metade que faltava; Enio 2026-07-19: *"Convert to
//! Curves não funciona para isso"*). Sem este teste a fiação ficava verde nos unit tests do
//! motor e MORTA no produto.

use super::*;
use crate::vec_entities::VecEntityMap;
use ph2d_vec_edit::{History, PenTool};
use ph2d_vec_scene::{VecPath, VecVertex, VecXforms};

/// O `to_curves` do produto com os canais que só o Offset vivo usa (pen/history/poses) — os
/// testes desta suíte não têm offset armado, então eles ficam vazios.
fn convert(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &mut VecEntityMap,
    ids: &[VecPathId],
) -> Vec<VecPathId> {
    let mut pen = PenTool::new();
    let mut history = History::new();
    crate::vec_convert::to_curves(
        sim,
        scene,
        map,
        &mut pen,
        &mut history,
        &VecXforms::default(),
        ids,
    )
}

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

    let new_sel = convert(&mut sim, &mut scene, &mut map, &[id]);

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

    convert(&mut sim, &mut scene, &mut map, &[id]);

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
                sim.world_mut()
                    .entity_mut(e)
                    .insert(VecConnector::between(1, 2));
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

        convert(&mut sim, &mut scene, &mut map, &[id]);

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

    let new_sel = convert(&mut sim, &mut scene, &mut map, &[id]);

    assert!(new_sel.contains(&id));
    assert_eq!(
        &before,
        scene.path(id).unwrap(),
        "sem forma viva nem efeitos, converter não pode mexer no caminho"
    );
}

/// REPRO (Enio): fillet numa quina de uma forma VIVA, troca de ferramenta, chanfra outra — e
/// entre as duas o frame roda o recook da receita. Se a receita não tiver sido congelada, o
/// `recook_into` reescreve `verts` INTEIRO e o 1º raio evapora.
#[test]
fn repro_two_tools_on_a_live_shape_across_a_recook() {
    use ph2d_vec_scene::ShapeKind;
    let (mut sim, mut scene, map, id) = square_entity();
    let e = entity_of(&map, id);
    let shape = VecShape::Param {
        kind: ShapeKind::Rectangle as u16,
        w: 40.0,
        h: 40.0,
        values: [0.0; ph2d_ecs::MAX_SHAPE_VALUES],
    };
    sim.world_mut().entity_mut(e).insert(shape.clone());
    // O frame cozinha a receita (é o que apaga tudo se a receita sobreviver).
    crate::vec_shape_live::recook_into(&mut scene, id, &shape);

    let mut pen = ph2d_vec_edit::PenTool::new();
    pen.select(Some(id));
    let a = scene.path(id).unwrap().verts[1].anchor;
    let b = scene.path(id).unwrap().verts[2].anchor;

    // OP 1 — Fillet na quina A (o press congela a receita).
    crate::vec_convert::freeze_shape_recipe(&mut sim, &map, id);
    pen.on_press_corner(&mut scene, a, 0.01, false);
    pen.on_drag(&mut scene, [a[0] - 4.0, a[1] + 4.0], &mut |p| p);
    pen.on_release();
    let r1 = scene.path(id).unwrap().verts[1].corner_radius;
    assert!(r1 > 0.0, "op1 arredondou (veio {r1})");

    // O FRAME entre as duas operações: se a receita ainda estivesse lá, o recook rodaria aqui.
    if let Some(s) = sim.world().get::<VecShape>(e).cloned() {
        crate::vec_shape_live::recook_into(&mut scene, id, &s);
    }

    // OP 2 — Chamfer na quina B, com a OUTRA ferramenta.
    pen.on_press_corner(&mut scene, b, 0.01, true);
    pen.on_drag(&mut scene, [b[0] - 4.0, b[1] - 4.0], &mut |p| p);
    pen.on_release();

    let v = &scene.path(id).unwrap().verts;
    eprintln!("r1={} r2={}", v[1].corner_radius, v[2].corner_radius);
    assert!(
        v[1].corner_radius > 0.0,
        "a 1a quina sobreviveu ao 2o gesto"
    );
    assert!(v[2].corner_radius < 0.0, "a 2a chanfrou");
}

/// **Uma receita CONGELADA não é ressuscitada** — o bug que apagava a quina anterior.
///
/// O `make_committed_shape_live` roda todo frame. Enquanto ele lia o `selected()` (que vive para
/// sempre) e se guardava só com *"o componente está presente?"*, congelar a receita de propósito
/// era lido como *"ainda não nasceu"*: o frame seguinte a repunha e chamava o `recook_into`, que
/// faz `p.verts = geom.verts` e **zera todo `corner_radius`**. Sobrevivia só o raio escrito
/// depois do recook — *"a mesma ferramenta desfaz o que tinha feito no outro ponto"* (Enio).
///
/// Agora o nascimento é um EVENTO consumido (`pending_live`). Este gate roda o frame DUAS vezes
/// depois do congelamento, que é onde o defeito aparecia.
#[test]
fn a_frozen_shape_recipe_is_not_resurrected_and_keeps_its_corner_radii() {
    use ph2d_vec_edit::shape::{ShapeConstraint, ShapeTool};
    use ph2d_vec_scene::ShapeKind;

    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let mut tool = ShapeTool::new();

    // O artista DESENHA um retângulo (o caminho real que pede o nascimento vivo).
    let id = tool.on_press(
        &mut scene,
        ShapeKind::Rectangle,
        Default::default(),
        [0.0, 0.0],
        0.01,
        ShapeConstraint::default(),
    );
    tool.on_drag(&mut scene, [40.0, 40.0], ShapeConstraint::default());
    assert!(tool.on_release(&mut scene), "a forma foi commitada");
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);

    // FRAME: ela nasce viva.
    crate::vec_shape_live::make_committed_shape_live(&mut sim, &mut scene, &map, &mut tool);
    let e = entity_of(&map, id);
    assert!(
        sim.world().get::<VecShape>(e).is_some(),
        "pré-condição: a forma nasceu VIVA"
    );

    // O artista congela a receita (é o que o gesto de quina faz) e arredonda DUAS quinas.
    assert!(crate::vec_convert::freeze_shape_recipe(&mut sim, &map, id));
    {
        let p = scene.path_mut(id).unwrap();
        p.verts[1].corner_radius = 5.0;
        p.verts[2].corner_radius = -3.0;
    }

    // DOIS frames depois — é aqui que a receita ressuscitava e varria os raios.
    for _ in 0..2 {
        crate::vec_shape_live::make_committed_shape_live(&mut sim, &mut scene, &map, &mut tool);
    }

    assert!(
        sim.world().get::<VecShape>(e).is_none(),
        "a receita congelada NÃO pode voltar — o artista a descartou de propósito"
    );
    let v = &scene.path(id).unwrap().verts;
    assert_eq!(v[1].corner_radius, 5.0, "a 1a quina sobreviveu ao frame");
    assert_eq!(v[2].corner_radius, -3.0, "e a 2a também");
}

// ---------------------------------------------------------------------------------------------
// W0.2 do plano 25 — **o nó de uma Live Shape** (a auditoria de 5 agentes, item 2).
//
// A nota dizia: *"arrastar âncora de uma Live Shape no modo Node talvez seja aceito e revertido em
// silêncio pelo recook do frame seguinte — repro antes de chamar defeito"*. Os dois gates abaixo
// medem, e a nota estava **meio certa**: o fenômeno existe (a edição é descartada em silêncio) mas o
// MECANISMO não é *"o frame seguinte"* — o `recook_into` não roda por frame, ele roda no
// nascimento da forma e a cada **edição de PARÂMETRO** (`vec_shape_params::edit_selected_shape`,
// os únicos dois chamadores de produção). O nó sobrevive até o artista encostar num slider.
// ---------------------------------------------------------------------------------------------

/// Uma forma VIVA (receita `Param`) de 40×40, já cozida, com a receita pendurada na entidade.
fn live_square() -> (SimWorld, VecScene, VecEntityMap, VecPathId) {
    use ph2d_vec_scene::ShapeKind;
    let (mut sim, mut scene, map, id) = square_entity();
    let shape = VecShape::Param {
        kind: ShapeKind::Rectangle as u16,
        w: 40.0,
        h: 40.0,
        values: [0.0; ph2d_ecs::MAX_SHAPE_VALUES],
    };
    sim.world_mut()
        .entity_mut(entity_of(&map, id))
        .insert(shape.clone());
    crate::vec_shape_live::recook_into(&mut scene, id, &shape);
    (sim, scene, map, id)
}

/// O que um arrasto de nó do modo Node deixa na cena: a âncora `0` deslocada.
fn drag_node_zero(scene: &mut VecScene, id: VecPathId) -> [f64; 2] {
    let p = scene.path_mut(id).expect("a forma existe");
    let moved = [p.verts[0].anchor[0] + 7.0, p.verts[0].anchor[1] - 3.0];
    p.verts[0].anchor = moved;
    moved
}

/// **O FENÔMENO: sem congelar a receita, o nó arrastado é descartado por uma edição de parâmetro.**
///
/// Este gate descreve o motor a fazer o que ele deve (uma receita reescreve a geometria dela) — ele
/// existe para pinar **o preço** dessa verdade quando o artista edita nós de uma forma viva, que é o
/// que justifica o congelamento no press. Ele fica VERDE: é a razão da cura, não a cura.
#[test]
fn a_node_edit_on_a_live_shape_is_wiped_by_the_next_param_edit() {
    let (mut sim, mut scene, map, id) = live_square();
    let moved = drag_node_zero(&mut scene, id);
    assert_eq!(
        scene.path(id).expect("existe").verts[0].anchor,
        moved,
        "o arrasto de no' nao chegou a' cena"
    );
    // O artista encosta num slider de parâmetro da forma (o único gesto que re-cozinha).
    let edited = crate::vec_shape_params::edit_selected_shape(
        &mut sim,
        &mut scene,
        &map,
        &[id],
        |_kind, _values| true,
    );
    assert!(
        edited,
        "a edicao de parametro tem de acontecer nesta fixture"
    );
    assert_ne!(
        scene.path(id).expect("existe").verts[0].anchor,
        moved,
        "a fixture nao contem o fenomeno: o recook nao mexeu na ancora que o no' arrastou"
    );
}

/// **A CURA: com a receita congelada no press, o nó sobrevive.**
///
/// É o mesmo `freeze_shape_recipe` que o par Fillet/Chamfer já chama, pelo mesmo motivo — e é por
/// isso que a wave não inventa política nova: ela dá ao press do Node a política que o press da
/// quina já tinha.
///
/// ⚠️ Mutação que tem de sangrar: tirar o `freeze_shape_recipe` do press do Node (o arch-gate irmão
/// prova que a shell o chama; este prova que chamá-lo RESOLVE).
#[test]
fn freezing_the_recipe_at_the_press_makes_the_node_edit_survive() {
    let (mut sim, mut scene, map, id) = live_square();
    // O press do modo Node: congela a receita ANTES de o pen tocar a geometria.
    assert!(
        crate::vec_convert::freeze_shape_recipe(&mut sim, &map, id),
        "a forma desta fixture tem receita a congelar"
    );
    let moved = drag_node_zero(&mut scene, id);
    // O artista encosta no slider: sem receita, não há o que re-cozinhar.
    crate::vec_shape_params::edit_selected_shape(&mut sim, &mut scene, &map, &[id], |_k, _v| true);
    assert_eq!(
        scene.path(id).expect("existe").verts[0].anchor,
        moved,
        "o no' arrastado foi descartado mesmo com a receita congelada"
    );
}
