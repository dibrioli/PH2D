//! Os gates da **ROLAGEM** de uma moldura.
//!
//! O motor já prova que o conteúdo TRANSBORDA (`ph2d-vec-layout::overflow_probe`). O que só se
//! pode afirmar aqui é a costura: que o excedente é MEDIDO e não autorado, que rolar move o
//! conteúdo e **não a moldura**, que o alcance é o excedente (nem mais, nem menos), que o conteúdo
//! a encolher PUXA a rolagem de volta — e que uma moldura que cabe fica **byte-intocada**.

use super::super::*;
use ph2d_ecs::{LayoutDir, VecLayout};
use ph2d_vec_scene::rectangle;

/// Uma coluna de `100×40` com `n` filhos de `100×10`. Com `n = 5` o conteúdo mede 50 e a moldura
/// 40 ⇒ **excedente 10**; com `n = 2` ele cabe, e essa é a fixture do controle.
fn column_of(n: usize) -> (SimWorld, VecScene, VecEntityMap, Entity, Vec<VecPathId>) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let frame_id = scene.push_path(rectangle([0.0, 0.0], [100.0, 40.0]));
    let kids: Vec<VecPathId> = (0..n)
        .map(|_| scene.push_path(rectangle([0.0, 0.0], [100.0, 10.0])))
        .collect();
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let frame = Entity::from_bits(map[&frame_id]);
    sim.world_mut().entity_mut(frame).insert(VecLayout {
        dir: LayoutDir::Column,
        ..Default::default()
    });
    for k in &kids {
        let kid = Entity::from_bits(map[k]);
        sim.world_mut()
            .entity_mut(kid)
            .insert(ph2d_ecs::ChildOf(frame));
    }
    (sim, scene, map, frame, kids)
}

/// Roda o passe num `LayoutLive` que JÁ EXISTE — o deslocamento sobrevive ao `recook`, e um
/// helper que construísse um novo a cada chamada não conseguiria afirmar isso.
fn recook(
    ll: &mut LayoutLive,
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
) -> LiveGeometry {
    let mut live = LiveGeometry::new();
    ll.recook(
        scene,
        sim,
        map,
        &VecXforms::default(),
        &mut live,
        crate::vec_bindings::TokenCtx::factory(),
    );
    live
}

/// O topo (em MUNDO) do que um caminho desenha.
fn top_of(live: &LiveGeometry, scene: &VecScene, id: VecPathId) -> f64 {
    let items = world_of(scene, &VecXforms::default(), live, id);
    bbox_of(&items).expect("desenha").1[1]
}

/// **O excedente é MEDIDO, e é o número certo.** Cinco filhos de 10 numa moldura de 40: 50 − 40.
#[test]
fn the_overflow_is_measured_from_the_content_that_does_not_fit() {
    let (sim, scene, map, frame, _) = column_of(5);
    let mut ll = LayoutLive::default();
    let _ = recook(&mut ll, &sim, &scene, &map);
    let o = ll.overflow_of(frame);
    assert!(
        (o[1] - 10.0).abs() < 1e-6,
        "cinco filhos de 10 numa moldura de 40 transbordam 10, e nao {:.3}",
        o[1]
    );
    assert_eq!(o[0], 0.0, "a largura cabe: nao ha excedente horizontal");
}

/// **O recuo do lado FINAL entra no excedente.**
///
/// ⚠️ Gate próprio porque toda a outra fixture tem recuo zero — e com recuo zero a linha que o
/// soma é indistinguível de não existir. Um card com 8 em baixo cujo último item encosta no fundo
/// tem 8 de conteúdo: sem eles a rolagem para 8 antes do fim, e a lista parece cortada.
#[test]
fn the_far_side_padding_counts_as_content() {
    let (mut sim, scene, map, frame, _) = column_of(5);
    sim.world_mut().entity_mut(frame).insert(VecLayout {
        dir: LayoutDir::Column,
        pad: [0.0, 0.0, 8.0, 0.0],
        ..Default::default()
    });
    let mut ll = LayoutLive::default();
    let _ = recook(&mut ll, &sim, &scene, &map);
    assert!(
        (ll.overflow_of(frame)[1] - 18.0).abs() < 1e-6,
        "50 de filhos + 8 de recuo em baixo contra 40 de moldura = 18, e nao {:.3}",
        ll.overflow_of(frame)[1]
    );
}

/// **O CONTROLE: uma moldura que cabe não reporta excedente e não rola.**
///
/// ⚠️ Sem ele, *"o excedente é 10"* não distingue **transbordou** de *o passe sempre reporta a
/// soma do conteúdo* — e a rolagem passaria a mexer em toda moldura do documento.
#[test]
fn a_frame_that_fits_reports_nothing_and_refuses_to_scroll() {
    let (sim, scene, map, frame, kids) = column_of(2);
    let mut ll = LayoutLive::default();
    let live = recook(&mut ll, &sim, &scene, &map);
    assert_eq!(ll.overflow_of(frame), [0.0, 0.0]);
    let before = top_of(&live, &scene, kids[0]);

    assert!(
        !ll.scroll_by(frame, [0.0, 30.0]),
        "quem nao tem excedente NAO rola — e o chamador conta com isso para deixar a roda passar"
    );
    let live = recook(&mut ll, &sim, &scene, &map);
    assert!(
        (top_of(&live, &scene, kids[0]) - before).abs() < 1e-9,
        "o mundo de quem cabe tem de ficar exactamente onde estava"
    );
}

/// **Rolar move o CONTEÚDO, e não a moldura.**
///
/// As duas metades no mesmo gate porque uma sem a outra é um defeito: filhos que se movem com a
/// moldura junto é *arrastar*, e a moldura a mover-se sozinha é a lista a fugir do lugar.
#[test]
fn scrolling_moves_the_content_and_never_the_frame() {
    let (sim, scene, map, frame, kids) = column_of(5);
    let frame_id = scene.paths()[0].id;
    let mut ll = LayoutLive::default();
    let live = recook(&mut ll, &sim, &scene, &map);
    let (kid_before, frame_before) = (
        top_of(&live, &scene, kids[0]),
        top_of(&live, &scene, frame_id),
    );

    assert!(ll.scroll_by(frame, [0.0, 6.0]));
    let live = recook(&mut ll, &sim, &scene, &map);
    assert!(
        (top_of(&live, &scene, kids[0]) - (kid_before + 6.0)).abs() < 1e-6,
        "rolar 6 para baixo sobe o conteudo 6 no mundo (Y-up): {:.3} contra {:.3}",
        top_of(&live, &scene, kids[0]),
        kid_before + 6.0
    );
    assert!(
        (top_of(&live, &scene, frame_id) - frame_before).abs() < 1e-9,
        "a MOLDURA nao se mexe"
    );
}

/// **O alcance é o excedente — nem mais, nem menos.**
///
/// ⚠️ As duas pontas: o fim da lista é alcançável (senão o último item é inalcançável, que é o
/// defeito que a rolagem existe para curar) e não se passa dele (senão a lista rola para dentro do
/// vazio).
#[test]
fn the_reach_is_exactly_the_overflow_at_both_ends() {
    let (sim, scene, map, frame, _) = column_of(5);
    let mut ll = LayoutLive::default();
    let _ = recook(&mut ll, &sim, &scene, &map);

    assert!(
        ll.scroll_by(frame, [0.0, 1000.0]),
        "rolar demais ainda rola"
    );
    assert!(
        (ll.scroll_of(frame)[1] - 10.0).abs() < 1e-9,
        "e para EXACTAMENTE no excedente: {:?}",
        ll.scroll_of(frame)
    );
    assert!(
        !ll.scroll_by(frame, [0.0, 1.0]),
        "no fim, mais roda nao move nada"
    );

    assert!(ll.scroll_by(frame, [0.0, -1000.0]));
    assert_eq!(ll.scroll_of(frame), [0.0, 0.0], "e o topo e' zero");
    assert!(!ll.scroll_by(frame, [0.0, -1.0]), "no topo, idem");
}

/// **O conteúdo que ENCOLHE puxa a rolagem de volta.**
///
/// ⚠️ É o segundo clamp, o do passe, e ele existe porque o primeiro (o do gesto) mede contra o
/// excedente de ANTES: apagar os filhos deixaria a lista rolada para fora de si mesma, a mostrar
/// vazio, com o número guardado a dizer que está tudo bem.
#[test]
fn content_that_shrinks_pulls_the_scroll_back() {
    let (mut sim, scene, map, frame, kids) = column_of(5);
    let mut ll = LayoutLive::default();
    let _ = recook(&mut ll, &sim, &scene, &map);
    assert!(ll.scroll_by(frame, [0.0, 10.0]));

    // Três filhos saem: o conteúdo passa a caber, e não há mais excedente nenhum.
    for k in &kids[2..] {
        sim.world_mut()
            .entity_mut(Entity::from_bits(map[k]))
            .remove::<ph2d_ecs::ChildOf>();
    }
    let live = recook(&mut ll, &sim, &scene, &map);
    assert_eq!(ll.overflow_of(frame), [0.0, 0.0], "agora cabe");
    let expect = top_of(&live, &scene, scene.paths()[0].id);
    assert!(
        (top_of(&live, &scene, kids[0]) - expect).abs() < 1e-6,
        "o primeiro filho tem de voltar ao topo da moldura, e nao ficar 10 acima dele"
    );
}

/// **Só uma moldura que RECORTA é alvo da roda.**
///
/// ⚠️ As duas metades no mesmo gate: sem recorte o conteúdo excedente está à vista, e rolar
/// moveria formas visíveis sob o cursor sem que nada as escondesse — mas com recorte a roda TEM de
/// pegar, senão a feature não existe. Um gate só com a metade positiva ficaria verde sobre uma
/// rolagem que rouba o zoom em toda moldura do documento.
#[test]
fn only_a_clipping_frame_is_a_wheel_target() {
    let (mut sim, scene, map, frame, _) = column_of(5);
    let mut ll = LayoutLive::default();
    let _ = recook(&mut ll, &sim, &scene, &map);
    assert!(
        ll.scrollable_frame_at([50.0, 20.0]).is_none(),
        "sem VecFrame nao ha' recorte, logo nao ha' alvo"
    );

    sim.world_mut()
        .entity_mut(frame)
        .insert(ph2d_ecs::VecFrame { clip: false });
    let _ = recook(&mut ll, &sim, &scene, &map);
    assert!(
        ll.scrollable_frame_at([50.0, 20.0]).is_none(),
        "moldura que NAO recorta tambem nao: o excedente esta' a' vista"
    );

    sim.world_mut()
        .entity_mut(frame)
        .insert(ph2d_ecs::VecFrame { clip: true });
    let _ = recook(&mut ll, &sim, &scene, &map);
    assert_eq!(
        ll.scrollable_frame_at([50.0, 20.0]),
        Some(frame),
        "e com recorte ela E' o alvo"
    );
    assert!(
        ll.scrollable_frame_at([500.0, 20.0]).is_none(),
        "fora da caixa dela, ninguem"
    );
}

/// **Uma moldura que CABE não é alvo da roda** — o irmão do controle, no eixo do gesto.
#[test]
fn a_frame_that_fits_is_not_a_wheel_target() {
    let (mut sim, scene, map, frame, _) = column_of(2);
    sim.world_mut()
        .entity_mut(frame)
        .insert(ph2d_ecs::VecFrame { clip: true });
    let mut ll = LayoutLive::default();
    let _ = recook(&mut ll, &sim, &scene, &map);
    assert!(
        ll.scrollable_frame_at([50.0, 20.0]).is_none(),
        "quem cabe nao rola — e e' isto que impede a roda de roubar o zoom no resto do documento"
    );
}

/// **Uma moldura ANINHADA rola por dentro, e o deslocamento ACUMULA.**
///
/// ⚠️ É o que separa *uma lista* de *uma lista dentro de um card*: sem a acumulação, rolar o card
/// deixaria a lista interna parada no lugar de origem, atravessando a moldura de fora.
#[test]
fn a_nested_frame_scrolls_inside_the_one_that_contains_it() {
    let (mut sim, mut scene, mut map, outer, _) = column_of(0);
    // Uma lista DENTRO do card: moldura interna de 100×30 com quatro filhos de 10 (conteúdo 40).
    let inner_id = scene.push_path(rectangle([0.0, 0.0], [100.0, 30.0]));
    let leaves: Vec<VecPathId> = (0..4)
        .map(|_| scene.push_path(rectangle([0.0, 0.0], [100.0, 10.0])))
        .collect();
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let inner = Entity::from_bits(map[&inner_id]);
    sim.world_mut()
        .entity_mut(inner)
        .insert(VecLayout {
            dir: LayoutDir::Column,
            ..Default::default()
        })
        .insert(ph2d_ecs::ChildOf(outer));
    for l in &leaves {
        sim.world_mut()
            .entity_mut(Entity::from_bits(map[l]))
            .insert(ph2d_ecs::ChildOf(inner));
    }

    let mut ll = LayoutLive::default();
    let live = recook(&mut ll, &sim, &scene, &map);
    assert!(
        ll.overflow_of(inner)[1] > 0.0,
        "a moldura INTERNA transborda: {:?}",
        ll.overflow_of(inner)
    );
    let before = top_of(&live, &scene, leaves[0]);
    let inner_before = top_of(&live, &scene, inner_id);

    assert!(ll.scroll_by(inner, [0.0, 5.0]));
    let live = recook(&mut ll, &sim, &scene, &map);
    assert!(
        (top_of(&live, &scene, leaves[0]) - (before + 5.0)).abs() < 1e-6,
        "a folha sobe 5"
    );
    assert!(
        (top_of(&live, &scene, inner_id) - inner_before).abs() < 1e-9,
        "e a moldura interna — que e' quem rola — fica onde esta'"
    );
}
