//! **Os FILTROS numa pose de estado de UI** — irmão do [`super::tests`] pelo teto de 600 LOC do
//! HR-18, e o corte é por assunto: aqui mora só a costura do canal que entrou em 2026-08-21.
//!
//! ⚠️ **Fixture PRÓPRIA e mínima, e não a do irmão.** Estes gates falam de UM objeto e do
//! componente que ele carrega; herdar a cena com filho (que existe para provar que o estado é da
//! SUB-ÁRVORE) traria uma premissa que eles não usam — e um gate que depende de coisas que não
//! afirma é um gate que fica verde pelo motivo errado no dia em que a fixture mudar.

use super::*;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_vec_scene::{VecScene, rectangle};

/// Um mundo com UMA forma, e o mapa `VecPathId → entidade`.
fn scene_with_one_shape() -> (SimWorld, VecScene, VecEntityMap, VecPathId) {
    let mut sim = SimWorld::new();
    let mut scene = VecScene::default();
    let id = scene.push_path(rectangle([0.0, 0.0], [2.0, 1.0]));
    let e = sim
        .world_mut()
        .spawn((
            Name("Host".into()),
            Transform::IDENTITY,
            ph2d_ecs::VecPathRef(id),
        ))
        .id();
    let mut map = VecEntityMap::new();
    map.insert(id, e.to_bits());
    (sim, scene, map, id)
}

/// **A COSTURA: o que a captura lê, a instalação escreve de volta.**
///
/// ⚠️ É o gate que impede a metade morta. Um campo novo na pose sem produtor passa em TODOS os
/// gates da `ph2d-ui-state` — eles testam o motor, que já sabia misturar —, e o sintoma só
/// aparece no canvas: o blur do artista **evapora** no primeiro Show, porque o `install` escreve
/// uma pilha vazia que a `capture` nunca preencheu. Foi exactamente assim que a `geometry` ficou
/// uma wave inteira sem animar.
#[test]
fn the_filter_survives_a_capture_and_an_install() {
    use ph2d_ecs::{FxOp, VecFilter};

    let (mut sim, mut scene, map, host) = scene_with_one_shape();
    let he = Entity::from_bits(map[&host]);
    sim.world_mut().entity_mut(he).insert(VecFilter {
        ops: vec![FxOp {
            radius: 12.0,
            ..FxOp::new(FxOp::BLUR)
        }],
    });

    let pose = capture(&sim, &scene, &map, host);
    assert_eq!(
        pose.filters.len(),
        1,
        "a captura nao leu a pilha: a pose nasce sem filtro e o Show apagaria o blur"
    );
    assert!((pose.filters[0].radius - 12.0).abs() < 1e-4);

    // Apaga do mundo e reinstala a partir da pose — o caminho do Show.
    sim.world_mut().entity_mut(he).remove::<VecFilter>();
    install(&mut sim, &mut scene, &map, &pose);
    let back = sim
        .world()
        .get::<VecFilter>(he)
        .expect("a instalacao nao devolveu a pilha ao mundo");
    assert!(
        (back.ops[0].radius - 12.0).abs() < 1e-4,
        "o raio nao voltou"
    );
}

/// **Uma pose SEM filtro REMOVE o componente** — a lei do `VecOffset` (*"um documento não acumula
/// relações inertes"*), e é ela que faz um estado sem filtro devolver a forma byte-idêntica.
///
/// ⚠️ **Vazio ≠ neutro**, e a distinção é load-bearing: a pose do MEIO de uma transição traz
/// degraus de intensidade zero, não uma pilha vazia — se o `install` tratasse os dois como o
/// mesmo caso, o filtro piscaria de volta ao original em cada quadro da animação.
#[test]
fn an_empty_stack_removes_the_component_but_a_neutral_one_does_not() {
    use ph2d_ecs::{FxOp, VecFilter};

    let (mut sim, mut scene, map, host) = scene_with_one_shape();
    let he = Entity::from_bits(map[&host]);
    sim.world_mut()
        .entity_mut(he)
        .insert(VecFilter::single(FxOp::new(FxOp::BLUR)));

    let mut pose = capture(&sim, &scene, &map, host);
    pose.filters.clear();
    install(&mut sim, &mut scene, &map, &pose);
    assert!(
        sim.world().get::<VecFilter>(he).is_none(),
        "pilha vazia tem de REMOVER o componente"
    );

    pose.filters = vec![FxOp::neutral(FxOp::BLUR)];
    install(&mut sim, &mut scene, &map, &pose);
    assert!(
        sim.world().get::<VecFilter>(he).is_some(),
        "um degrau NEUTRO nao e' uma pilha vazia -- e' o quadro do meio da animacao"
    );
}
