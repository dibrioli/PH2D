//! Os gates da porta que faz a alça REDIMENSIONAR a caixa em vez de ESCALAR a pose.
//!
//! O que só se pode afirmar aqui: que o default segue a HIERARQUIA e que o checkbox o vence nas
//! duas direcções, que o ponto fixo sai do PIVÔ (e não de uma tabela de cantos), que a razão é
//! ABSOLUTA — logo o resultado é um fato do gesto e não de quantos eventos de rato ele durou — e
//! que voltar ao início devolve a geometria **ao bit**.
//!
//! A outra metade — *o braço existe no `advance_gizmo_drag` e NÃO escreve `Transform`* — precisa
//! de `App` + janela, e vive no arch-gate irmão
//! (`tests/a_frames_handle_resizes_it_and_does_not_scale_it.rs`).

use super::*;
use ph2d_ecs::VecFrame;
use ph2d_vec_scene::{VecXforms, rectangle};

use crate::vec_entities::VecEntityMap;

/// Uma moldura de 100×40 na origem, com um filho de 10×10, já sincronizada.
/// Devolve `(sim, scene, map, [moldura, filho])`.
fn frame_and_kid() -> (SimWorld, VecScene, VecEntityMap, [VecPathId; 2]) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let frame_id = scene.push_path(rectangle([0.0, 0.0], [100.0, 40.0]));
    let kid_id = scene.push_path(rectangle([10.0, 10.0], [20.0, 20.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let frame = Entity::from_bits(map[&frame_id]);
    sim.world_mut()
        .entity_mut(frame)
        .insert(VecFrame { clip: false });
    let kid = Entity::from_bits(map[&kid_id]);
    sim.world_mut()
        .entity_mut(kid)
        .insert(ph2d_ecs::ChildOf(frame));
    (sim, scene, map, [frame_id, kid_id])
}

fn box_of(scene: &VecScene, id: VecPathId) -> ([f64; 2], [f64; 2]) {
    scene.path_curve_bbox(id).expect("a forma tem geometria")
}

/// **A moldura E o filho dela redimensionam; o objeto solto ESCALA.**
///
/// ⚠️ O terceiro é o CONTROLO, e é a metade que o Enio nomeou: *o comportamento anterior era
/// correto para objetos de game*. Sem ele o gate não distingue *"a regra segue a hierarquia"* de
/// *"a regra vale sempre"*.
#[test]
fn a_frame_and_its_child_resize_and_a_loose_object_scales() {
    let (sim, _scene, map, ids) = frame_and_kid();
    let frame = Entity::from_bits(map[&ids[0]]);
    let kid = Entity::from_bits(map[&ids[1]]);
    assert_eq!(resizable_frame(&sim, frame), Some(ids[0]));
    assert_eq!(
        resizable_frame(&sim, kid),
        Some(ids[1]),
        "o filho da moldura"
    );

    // O controlo: a MESMA forma, agora fora da moldura.
    let (mut sim2, mut scene2, mut map2, _ids2) = frame_and_kid();
    let loose_id = scene2.push_path(rectangle([0.0, 0.0], [5.0, 5.0]));
    crate::vec_entities::sync(&mut sim2, &mut scene2, &mut map2);
    let loose = Entity::from_bits(map2[&loose_id]);
    assert_eq!(
        resizable_frame(&sim2, loose),
        None,
        "um objeto de game escala, como sempre"
    );
}

/// **O checkbox VENCE o default, e a direcção que importa é DESMARCAR a moldura.**
///
/// ⚠️ É o pedido do Enio feito função: um contentor que é *cenário* e não *layout* quer escalar o
/// conteúdo junto, e sem o override isso ficava inexprimível dentro de uma moldura.
#[test]
fn the_checkbox_overrides_the_default_in_both_directions() {
    let (mut sim, _scene, map, ids) = frame_and_kid();
    let frame = Entity::from_bits(map[&ids[0]]);
    sim.world_mut()
        .entity_mut(frame)
        .insert(ph2d_ecs::VecResizeBox(false));
    assert_eq!(
        resizable_frame(&sim, frame),
        None,
        "a moldura desmarcada volta a escalar tudo"
    );

    // E a outra direcção: uma forma solta MARCADA reescreve a caixa.
    let (mut sim2, mut scene2, mut map2, _ids2) = frame_and_kid();
    let loose_id = scene2.push_path(rectangle([0.0, 0.0], [5.0, 5.0]));
    crate::vec_entities::sync(&mut sim2, &mut scene2, &mut map2);
    let loose = Entity::from_bits(map2[&loose_id]);
    sim2.world_mut()
        .entity_mut(loose)
        .insert(ph2d_ecs::VecResizeBox(true));
    assert_eq!(resizable_frame(&sim2, loose), Some(loose_id));
}

/// **Um GRUPO marcado não tem caixa a reescrever, e por isso escala.**
///
/// ⚠️ Uma entidade sem `VecPathRef` é um contentor sem geometria própria. Sem esta recusa o braço
/// do gizmo receberia um `Some` e não teria o que fotografar.
#[test]
fn a_group_without_geometry_still_scales() {
    let (mut sim, _scene, _map, _ids) = frame_and_kid();
    let group = sim.world_mut().spawn(ph2d_ecs::VecResizeBox(true)).id();
    assert_eq!(resizable_frame(&sim, group), None);
}

/// **O ponto fixo sai do PIVÔ.** Arrastar a alça de cima-direita mantém o canto mínimo; arrastar a
/// de baixo-esquerda mantém o MÁXIMO — sem tabela de cantos e sem análise de qual alça foi pegada.
#[test]
fn the_corner_the_gizmo_pinned_is_the_corner_that_stays() {
    let (_sim, mut scene, _map, ids) = frame_and_kid();
    let x = VecXforms::default();
    // Pivô no canto mínimo (arrastando a alça oposta): o mínimo não se move.
    let s = begin(&scene, &x, 1, ids[0], [0.0, 0.0]).expect("armou");
    apply(&mut scene, &s, 2.0, 1.0);
    let (lo, hi) = box_of(&scene, ids[0]);
    assert!((lo[0] - 0.0).abs() < 1e-9, "o minimo andou: {lo:?}");
    assert!((hi[0] - 200.0).abs() < 1e-6, "o maximo nao seguiu: {hi:?}");

    // Pivô no canto MÁXIMO: agora é ele que fica, e o mínimo é que anda.
    let (_sim2, mut scene2, _m2, ids2) = frame_and_kid();
    let s2 = begin(&scene2, &x, 1, ids2[0], [100.0, 40.0]).expect("armou");
    apply(&mut scene2, &s2, 2.0, 1.0);
    let (lo2, hi2) = box_of(&scene2, ids2[0]);
    assert!((hi2[0] - 100.0).abs() < 1e-9, "o maximo andou: {hi2:?}");
    assert!(
        (lo2[0] + 100.0).abs() < 1e-6,
        "o minimo nao seguiu: {lo2:?}"
    );
}

/// **O CTRL ancora no centro, e isso cai de graça** — o pivô é lido, não enumerado.
///
/// ⚠️ Este é o caso que uma tabela `alça → canto oposto` não teria: o gizmo troca o pivô para o
/// centro no pen-down, e uma segunda resposta local nem saberia que isso aconteceu.
#[test]
fn a_centre_pivot_needs_no_special_case() {
    let (_sim, mut scene, _map, ids) = frame_and_kid();
    let x = VecXforms::default();
    let s = begin(&scene, &x, 1, ids[0], [50.0, 20.0]).expect("armou");
    apply(&mut scene, &s, 2.0, 1.0);
    let (lo, hi) = box_of(&scene, ids[0]);
    assert!(
        (lo[0] + 50.0).abs() < 1e-6,
        "cresceu so' para um lado: {lo:?}"
    );
    assert!(
        (hi[0] - 150.0).abs() < 1e-6,
        "cresceu so' para um lado: {hi:?}"
    );
}

/// **A razão é ABSOLUTA: o resultado é um fato do GESTO, não de quantos eventos ele durou.**
///
/// ⚠️ A mutação que este gate existe para matar é a barata: escalar a geometria VIVA a cada
/// movimento em vez de restaurar o instantâneo. Ela é invisível num teste de um movimento só e
/// multiplica a razão uma vez por evento de rato num arrasto real — o mesmo mal que o depósito do
/// Painter curou quatro vezes (*a lei é fato do CAMINHO, nunca de quão fino ele foi amostrado*).
#[test]
fn ten_moves_to_double_it_double_it_once() {
    let (_sim, mut one, _m, ids) = frame_and_kid();
    let x = VecXforms::default();
    let s = begin(&one, &x, 1, ids[0], [0.0, 0.0]).expect("armou");
    apply(&mut one, &s, 2.0, 2.0);

    let (_sim2, mut many, _m2, ids2) = frame_and_kid();
    let s2 = begin(&many, &x, 1, ids2[0], [0.0, 0.0]).expect("armou");
    // O gizmo entrega uma razão absoluta por CursorMoved; o caminho até 2× não pode contar.
    for k in 1..=10 {
        let f = 1.0 + f64::from(k) / 10.0;
        apply(&mut many, &s2, f, f);
    }
    assert_eq!(
        box_of(&one, ids[0]),
        box_of(&many, ids2[0]),
        "a moldura lembra-se do caminho: a razao esta' a compor por evento"
    );
}

/// **Voltar ao ponto de partida devolve a geometria AO BIT.**
#[test]
fn dragging_back_to_where_it_started_restores_the_geometry_exactly() {
    let (_sim, mut scene, _map, ids) = frame_and_kid();
    let x = VecXforms::default();
    let pristine = scene
        .paths()
        .iter()
        .find(|p| p.id == ids[0])
        .expect("a moldura")
        .clone();
    let s = begin(&scene, &x, 1, ids[0], [0.0, 0.0]).expect("armou");
    apply(&mut scene, &s, 3.7, 0.4);
    apply(&mut scene, &s, 1.0, 1.0);
    assert_eq!(
        scene.paths().iter().find(|p| p.id == ids[0]),
        Some(&pristine),
        "a volta ao inicio e' aproximada — o instantaneo nao esta' a ser restaurado"
    );
}

/// **O FILHO não é tocado.** É o defeito que o Enio reportou, medido: a moldura muda de caixa e a
/// geometria do filho fica exatamente onde estava (quem o move, se ele tiver regra, é o passe de
/// âncoras — e é ele que decide, não o gizmo).
#[test]
fn resizing_the_frame_does_not_touch_the_child_geometry() {
    let (_sim, mut scene, _map, ids) = frame_and_kid();
    let x = VecXforms::default();
    let before = box_of(&scene, ids[1]);
    let s = begin(&scene, &x, 1, ids[0], [0.0, 0.0]).expect("armou");
    apply(&mut scene, &s, 2.5, 0.5);
    assert_eq!(box_of(&scene, ids[1]), before, "o filho esticou");
}

/// **Uma moldura não passa do avesso.** O clamp é o mesmo do `W` do painel; ver [`MIN_RATIO`].
///
/// ⚠️ **O oráculo é o PONTO FIXO ficar do lado em que estava**, e a 1ª versão deste gate
/// sobreviveu à mutação por afirmar as duas coisas erradas: `hi > lo` **não pode falhar** (uma
/// caixa é normalizada por construção) e `hi < 1` passa com a caixa espelhada, porque espelhar
/// leva o máximo exatamente para cima do ponto fixo. *Uma razão negativa não encolhe a caixa —
/// ela a leva para o outro lado da âncora*, e é isso que se mede.
#[test]
fn a_frame_does_not_flip_through_itself() {
    let (_sim, mut scene, _map, ids) = frame_and_kid();
    let x = VecXforms::default();
    let s = begin(&scene, &x, 1, ids[0], [0.0, 0.0]).expect("armou");
    apply(&mut scene, &s, -2.0, 1.0);
    let (lo, hi) = box_of(&scene, ids[0]);
    assert!(
        lo[0].abs() < 1e-9,
        "a moldura atravessou o ponto fixo: {lo:?}..{hi:?}"
    );
    assert!(hi[0] > lo[0] && hi[0] < 1.0, "o clamp nao apertou: {hi:?}");
}

/// **Um instantâneo pertence a UM arrasto.** Sem isto, soltar e voltar a pegar noutra forma
/// restauraria a geometria da anterior por cima dela.
#[test]
fn a_snapshot_belongs_to_the_drag_that_took_it() {
    let (_sim, scene, _map, ids) = frame_and_kid();
    let x = VecXforms::default();
    let s = begin(&scene, &x, 7, ids[0], [0.0, 0.0]).expect("armou");
    assert!(s.is_for(7));
    assert!(!s.is_for(8));
}
