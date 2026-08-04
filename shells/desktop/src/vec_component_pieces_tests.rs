//! Gates das **DIFERENÇAS** de uma instância (plano UI/UX W5b).
//!
//! ⚠️ O keystone é o primeiro: a W5a shipou `OverrideSlot` com gates e **sem porta**, e nenhum
//! gate de então podia ver isso — cada metade estava certa sozinha. Estes provam que o gesto
//! chega ao DESENHO, que é a quarta condição da política de UI.

use super::*;
use ph2d_ecs::{Transform, VecComponentMain};
use ph2d_vec_scene::{Paint, Rgba8, VecXforms, rectangle};

/// A cor de teste do mestre e a que a cópia autora — distintas para o oráculo não empatar.
const MAIN_RED: Rgba8 = Rgba8 {
    r: 200,
    g: 40,
    b: 40,
    a: 255,
};
const OWN_BLUE: [u8; 4] = [40, 90, 220, 255];

fn own_blue() -> Paint {
    Paint::Solid(Rgba8::new(
        OWN_BLUE[0],
        OWN_BLUE[1],
        OWN_BLUE[2],
        OWN_BLUE[3],
    ))
}

/// **Um mestre de DUAS peças** (caixa + etiqueta) e uma instância dele, tudo sincronizado.
///
/// Duas peças e não uma: com uma só, *"a lista é a sub-árvore"* e *"a lista é a raiz"* dão a mesma
/// resposta, e o gate que separa as duas ficaria verde por vácuo.
fn main_with_two_pieces() -> (SimWorld, VecScene, VecEntityMap, VecPathId, VecPathId) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let mut body = rectangle([0.0, 0.0], [20.0, 10.0]);
    body.fill = Some(Paint::Solid(MAIN_RED));
    let body = scene.push_path(body);
    let mut label = rectangle([4.0, 3.0], [16.0, 7.0]);
    label.fill = Some(Paint::Solid(Rgba8::new(240, 240, 240, 255)));
    let label = scene.push_path(label);
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let (be, le) = (
        Entity::from_bits(map[&body]),
        Entity::from_bits(map[&label]),
    );
    crate::vec_transform::reparent_keeping_world(&mut sim, le, be);
    sim.world_mut().entity_mut(be).insert(VecComponentMain);
    (sim, scene, map, body, label)
}

/// Põe uma instância do mestre `main` e devolve o caminho dela.
fn place(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &mut VecEntityMap,
    main: VecPathId,
) -> VecPathId {
    let id =
        crate::vec_component_edit::place_instance(sim, scene, map, &[main]).expect("Place recusou");
    crate::vec_entities::sync(sim, scene, map);
    crate::vec_component_edit::arm_instance(sim, map, id, main, [30.0, 0.0]);
    id
}

/// O que a instância `at` DESENHA neste estado do mundo.
fn drawn(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    at: VecPathId,
) -> Vec<ph2d_vec_scene::VecPath> {
    let xf: VecXforms = crate::vec_transform::build(sim, map);
    let mut live = crate::instance_live::InstanceLive::default();
    live.recook(scene, sim, map, &xf);
    live.live().get(&at).cloned().unwrap_or_default()
}

fn ent(map: &VecEntityMap, id: VecPathId) -> Entity {
    Entity::from_bits(map[&id])
}

/// A instância viva de `at`.
fn inst_of(sim: &SimWorld, map: &VecEntityMap, at: VecPathId) -> VecInstance {
    sim.world()
        .get::<VecInstance>(ent(map, at))
        .cloned()
        .expect("a instância sumiu")
}

/// A linha da peça de caminho `piece` na lista publicada — pelo ENDEREÇO, nunca pelo nome.
fn row_of(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    main: Entity,
    piece: VecPathId,
) -> usize {
    let (pieces, _) = addressed_pieces(sim, scene, map, main);
    pieces
        .iter()
        .position(|p| *p == piece)
        .expect("a peça não está na lista endereçada")
}

/// **O KEYSTONE: o interruptor é a porta que faltava.** Esconder uma peça muda o DESENHO.
///
/// ⚠️ Antes da W5b nada no editor podia produzir um `OverrideSlot`, e todos os gates do modelo
/// estavam verdes. A mutação que mata este gate é o interruptor não escrever nada — e nenhum gate
/// da W5a a vê.
#[test]
fn the_switch_hides_a_piece_of_this_copy_and_only_of_this_copy() {
    let (mut sim, mut scene, mut map, body, label) = main_with_two_pieces();
    let a = place(&mut sim, &mut scene, &mut map, body);
    let b = place(&mut sim, &mut scene, &mut map, body);
    assert_eq!(
        drawn(&sim, &scene, &map, a).len(),
        2,
        "a cópia desenha as 2"
    );

    let row = row_of(&sim, &scene, &map, ent(&map, body), label);
    assert!(toggle_piece_visible(
        &mut sim,
        &scene,
        &map,
        ent(&map, a),
        row
    ));

    assert_eq!(
        drawn(&sim, &scene, &map, a).len(),
        1,
        "esconder uma peça não mudou o desenho — o interruptor não chegou ao produtor"
    );
    assert_eq!(
        drawn(&sim, &scene, &map, b).len(),
        2,
        "esconder numa cópia escondeu na IRMÃ — o override não é da instância"
    );
}

/// **Esconder uma peça NÃO lhe tira a linha** — senão o gesto seria de mão única.
///
/// ⚠️ A mutação que o mata é a lista passar a ser `visible_pieces`: aí a peça escondida some da
/// lista e o artista perde-a sem um erro, com o *Reset* (que apaga tudo) como única volta.
#[test]
fn a_hidden_piece_keeps_its_row_so_the_switch_has_a_way_back() {
    let (mut sim, mut scene, mut map, body, label) = main_with_two_pieces();
    let a = place(&mut sim, &mut scene, &mut map, body);
    let row = row_of(&sim, &scene, &map, ent(&map, body), label);
    toggle_piece_visible(&mut sim, &scene, &map, ent(&map, a), row);

    let (rows, _) = piece_rows(&sim, &scene, &map, ent(&map, body), &inst_of(&sim, &map, a));
    assert_eq!(rows.len(), 2, "a peça escondida perdeu a própria linha");
    assert!(!rows[row].visible, "a linha não diz que ela está escondida");

    // E a volta: o segundo toque REMOVE o override (não grava um "visível"), senão a cópia
    // carregaria uma diferença que não existe e o *Reset* acenderia sobre uma cópia idêntica.
    toggle_piece_visible(&mut sim, &scene, &map, ent(&map, a), row);
    assert!(
        inst_of(&sim, &map, a).overrides.is_empty(),
        "voltar a mostrar deixou um override para trás"
    );
    assert_eq!(drawn(&sim, &scene, &map, a).len(), 2);
}

/// **A swatch escreve a cor DESTA cópia, e o mestre não se mexe.**
#[test]
fn the_swatch_writes_an_override_the_copy_draws_and_the_main_keeps_its_own() {
    let (mut sim, mut scene, mut map, body, _label) = main_with_two_pieces();
    let a = place(&mut sim, &mut scene, &mut map, body);
    let row = row_of(&sim, &scene, &map, ent(&map, body), body);
    assert!(set_piece_colour(
        &mut sim,
        &scene,
        &map,
        ent(&map, a),
        row,
        OWN_BLUE
    ));

    assert!(
        drawn(&sim, &scene, &map, a)
            .iter()
            .any(|p| p.fill == Some(own_blue())),
        "a cópia não desenhou a cor autorada"
    );
    assert_eq!(
        scene.paths().iter().find(|p| p.id == body).unwrap().fill,
        Some(Paint::Solid(MAIN_RED)),
        "autorar na cópia mexeu no MESTRE"
    );

    // A linha volta a dizer a cor EFETIVA, e marca que ela é da cópia.
    let (rows, _) = piece_rows(&sim, &scene, &map, ent(&map, body), &inst_of(&sim, &map, a));
    assert_eq!(rows[row].colour, OWN_BLUE);
    assert!(rows[row].overridden, "a linha não marca a cor como própria");
}

/// **A MESMA cor duas vezes não é um segundo passo de undo.**
///
/// ⚠️ O picker publica a escolha a CADA frame enquanto está aberto; sem esta recusa o
/// `post_frame_undo` gravaria um passo por frame, e o Ctrl+Z do artista andaria um frame de cada
/// vez. É a mesma classe do `reset_overrides` a recusar a instância limpa.
#[test]
fn writing_the_same_colour_again_is_not_a_second_undo_step() {
    let (mut sim, mut scene, mut map, body, _label) = main_with_two_pieces();
    let a = place(&mut sim, &mut scene, &mut map, body);
    assert!(set_piece_colour(
        &mut sim,
        &scene,
        &map,
        ent(&map, a),
        0,
        OWN_BLUE
    ));
    assert!(
        !set_piece_colour(&mut sim, &scene, &map, ent(&map, a), 0, OWN_BLUE),
        "a mesma cor foi aceite outra vez — um passo de undo por frame"
    );
}

/// **Update Main: a cor sobe ao mestre e a IRMÃ herda.**
#[test]
fn update_main_moves_the_colour_to_the_main_and_the_sister_inherits() {
    let (mut sim, mut scene, mut map, body, _label) = main_with_two_pieces();
    let a = place(&mut sim, &mut scene, &mut map, body);
    let b = place(&mut sim, &mut scene, &mut map, body);
    let row = row_of(&sim, &scene, &map, ent(&map, body), body);
    set_piece_colour(&mut sim, &scene, &map, ent(&map, a), row, OWN_BLUE);
    // A irmã ainda desenha o vermelho do mestre — o controle do gate.
    assert!(
        drawn(&sim, &scene, &map, b)
            .iter()
            .any(|p| p.fill == Some(Paint::Solid(MAIN_RED))),
        "a irmã já desenhava a cor nova antes do Update Main"
    );

    let (taken, refused) = update_main(&mut sim, &mut scene, ent(&map, a));
    assert_eq!((taken, refused), (1, 0));
    assert_eq!(
        scene.paths().iter().find(|p| p.id == body).unwrap().fill,
        Some(own_blue()),
        "o mestre não absorveu a cor"
    );
    assert!(
        drawn(&sim, &scene, &map, b)
            .iter()
            .any(|p| p.fill == Some(own_blue())),
        "a irmã não herdou"
    );
    assert!(
        inst_of(&sim, &map, a).overrides.is_empty(),
        "a diferença absorvida ficou na cópia — ela mostraria o mesmo por dois caminhos"
    );
}

/// **O `Hidden` NÃO sobe, e é CONTADO** — o mestre não tem *"peça escondida"*.
///
/// ⚠️ A metade que importa é a segunda: recusar em silêncio deixaria o *Reset* aceso depois de um
/// Update Main que o artista julga ter absorvido tudo.
#[test]
fn update_main_refuses_a_hidden_piece_and_reports_it() {
    let (mut sim, mut scene, mut map, body, label) = main_with_two_pieces();
    let a = place(&mut sim, &mut scene, &mut map, body);
    let hide_row = row_of(&sim, &scene, &map, ent(&map, body), label);
    let paint_row = row_of(&sim, &scene, &map, ent(&map, body), body);
    toggle_piece_visible(&mut sim, &scene, &map, ent(&map, a), hide_row);
    set_piece_colour(&mut sim, &scene, &map, ent(&map, a), paint_row, OWN_BLUE);

    let (taken, refused) = update_main(&mut sim, &mut scene, ent(&map, a));
    assert_eq!(taken, 1, "a cor devia ter subido");
    assert_eq!(refused, 1, "o Hidden devia ter sido contado como recusado");
    let inst = inst_of(&sim, &map, a);
    assert_eq!(inst.overrides.len(), 1, "o Hidden tinha de FICAR na cópia");
    assert_eq!(inst.overrides[0].slot, OverrideSlot::Hidden);
    assert_eq!(
        drawn(&sim, &scene, &map, a).len(),
        1,
        "a peça escondida voltou a aparecer"
    );
}

/// **Swap: a instância passa a derivar do outro mestre, e os overrides órfãos caem.**
#[test]
fn a_swap_repoints_the_copy_and_drops_the_overrides_the_new_main_cannot_address() {
    let (mut sim, mut scene, mut map, body, _label) = main_with_two_pieces();
    let a = place(&mut sim, &mut scene, &mut map, body);
    let row = row_of(&sim, &scene, &map, ent(&map, body), body);
    set_piece_colour(&mut sim, &scene, &map, ent(&map, a), row, OWN_BLUE);
    // Um SEGUNDO mestre, de uma peça só.
    let green = Rgba8::new(10, 200, 10, 255);
    let mut other = rectangle([0.0, 0.0], [6.0, 6.0]);
    other.fill = Some(Paint::Solid(green));
    let other = scene.push_path(other);
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    sim.world_mut()
        .entity_mut(ent(&map, other))
        .insert(VecComponentMain);

    let (swapped, dropped) =
        swap_main(&mut sim, &scene, &map, ent(&map, a), other).expect("o swap recusou o mestre");
    assert!(swapped);
    assert_eq!(dropped, 1, "o override do mestre antigo tinha de cair");
    let inst = inst_of(&sim, &map, a);
    assert_eq!(inst.main, other);
    assert!(inst.overrides.is_empty());
    let items = drawn(&sim, &scene, &map, a);
    assert_eq!(items.len(), 1, "a cópia devia desenhar o mestre NOVO");
    assert_eq!(items[0].fill, Some(Paint::Solid(green)));
}

/// **Uma forma que não se declara MESTRE não serve de alvo** — o mesmo contrato do produtor.
#[test]
fn a_swap_onto_a_shape_that_is_not_a_main_is_refused() {
    let (mut sim, mut scene, mut map, body, _label) = main_with_two_pieces();
    let a = place(&mut sim, &mut scene, &mut map, body);
    let plain = scene.push_path(rectangle([50.0, 50.0], [60.0, 60.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    assert!(
        swap_main(&mut sim, &scene, &map, ent(&map, a), plain).is_none(),
        "uma forma comum foi aceite como mestre"
    );
    assert_eq!(
        inst_of(&sim, &map, a).main,
        body,
        "a instância trocou de mestre à mesma"
    );
}

/// **O que passa do teto é CONTADO, não descartado em silêncio.**
///
/// ⚠️ Um teto que trunca calado lê-se como *"o mestre só tem estas peças"*, e o artista procura a
/// que falta onde ela não está. As peças de além do teto continuam a DESENHAR e a herdar — o teto
/// é da lista do painel, não do mestre.
#[test]
fn pieces_beyond_the_cap_are_reported_and_still_drawn() {
    let (mut sim, mut scene, mut map, body, _label) = main_with_two_pieces();
    let cap = ph2d_editor::ids::MAX_INSTANCE_PIECES;
    let extra = 3;
    for i in 0..cap + extra {
        #[allow(clippy::cast_precision_loss)]
        let x = i as f64;
        scene.push_path(rectangle([x, 0.0], [x + 0.5, 0.5]));
    }
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    // Todas penduradas no mestre — a sub-árvore passa do teto. (As duas primeiras são a caixa e a
    // etiqueta, que já estão na árvore.)
    let ids: Vec<VecPathId> = scene.paths().iter().map(|p| p.id).skip(2).collect();
    for id in ids {
        crate::vec_transform::reparent_keeping_world(&mut sim, ent(&map, id), ent(&map, body));
    }
    let a = place(&mut sim, &mut scene, &mut map, body);
    let total = 2 + cap + extra;
    let (rows, beyond) = piece_rows(&sim, &scene, &map, ent(&map, body), &inst_of(&sim, &map, a));
    assert_eq!(rows.len(), cap, "a lista passou do próprio teto");
    assert_eq!(beyond, total - cap, "o excedente não foi contado");
    assert_eq!(
        drawn(&sim, &scene, &map, a).len(),
        total,
        "uma peça além do teto deixou de DESENHAR — o teto é da lista, não do mestre"
    );
}

/// **A linha endereça a PEÇA, não o índice** — o que faz o override sobreviver ao mestre mudar.
#[test]
fn the_row_stores_the_piece_of_the_main_not_the_row_index() {
    let (mut sim, mut scene, mut map, body, label) = main_with_two_pieces();
    let a = place(&mut sim, &mut scene, &mut map, body);
    let row = row_of(&sim, &scene, &map, ent(&map, body), label);
    set_piece_colour(&mut sim, &scene, &map, ent(&map, a), row, OWN_BLUE);
    assert_eq!(
        inst_of(&sim, &map, a).overrides[0].sub,
        label,
        "o guardado não é o id da peça no MESTRE"
    );

    // O mestre MUDA (a etiqueta anda) — o override continua a apontar a mesma peça.
    if let Ok(mut em) = sim.world_mut().get_entity_mut(ent(&map, label)) {
        let mut t = em.get::<Transform>().copied().unwrap_or_default();
        t.translation.x += 5.0;
        em.insert(t);
    }
    let (rows, _) = piece_rows(&sim, &scene, &map, ent(&map, body), &inst_of(&sim, &map, a));
    assert!(
        rows[row].overridden,
        "editar o mestre soltou o override da peça"
    );
}
