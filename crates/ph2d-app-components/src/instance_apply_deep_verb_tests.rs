//! ⭐⭐⭐ **O QUE O *APLICAR* FAZ EM CADA DEGRAU** — irmão por ASSUNTO do
//! `instance_apply_deep_tests`, pelo tecto de 600 LOC do shell.
//!
//! Lá mora a fixtura aninhada e a escada que se **lê**; aqui, o verbo que **escreve** — as duas
//! metades do critério 4 da F5, cada uma com a mutação que a mata (ver o doc de cada gate).
//!
//! ⚠️ **A fixtura é importada, nunca redeclarada:** ela monta uma Roda dentro de um Carro com um
//! Carro na cena e uma Roda solta, e uma segunda montagem divergiria da primeira em silêncio.

use super::super::{Applied, apply_to_level};
use super::{copy_of, nested_car, overrides, paint, pass, reg, tint};
use crate::instance_sync::MasterEcho;
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::PhysicsBridge;
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// O *Aplicar* num degrau, com o par de documentos vazio.
fn apply_at(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    echo: &mut MasterEcho,
    clicked: Entity,
    target: Entity,
) -> Applied {
    let id = sim
        .world()
        .get::<ph2d_ecs::StableId>(target)
        .expect("a receita tem identidade")
        .0;
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    apply_to_level(
        sim,
        r,
        echo,
        clicked,
        id,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
    )
    .expect("a peca e' de uma copia")
}

const BLUE: [f32; 4] = [0.2, 0.4, 0.9, 1.0];
const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

// ── OS DOIS DEGRAUS ────────────────────────────────────────────────────────────────────────

/// ⭐⭐⭐ **APLICAR AO DEGRAU DE FORA muda todos os Carros e deixa a Roda em paz** — e a excepção
/// **muda de sítio**: ela passa a viver na cópia da Roda que está dentro do Carro, que é
/// literalmente o *«the value becomes an override on the instance of 'Vase' that is inside the
/// 'Table' Prefab»* do Unity.
#[test]
fn applying_to_the_outer_master_leaves_the_inner_recipe_alone() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = MasterEcho::default();
    let n = nested_car(&mut sim, &r);
    pass(&mut sim, &r, &bridge, &mut echo);

    paint(&mut sim, n.rim_in_scene, BLUE);
    pass(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(
        overrides(&sim, n.car_scene),
        1,
        "a cena capturou a excepcao"
    );

    let done = apply_at(&mut sim, &r, &mut echo, n.rim_in_scene, n.car_master);
    assert_eq!((done.changed, done.left), (1, 0));
    pass(&mut sim, &r, &bridge, &mut echo);

    assert_eq!(tint(&sim, n.rim_in_car), BLUE, "a receita do Carro recebeu");
    assert_eq!(
        tint(&sim, n.rim_in_scene),
        BLUE,
        "e a cena ficou como estava"
    );
    assert_eq!(
        tint(&sim, n.rim_in_wheel),
        WHITE,
        "a receita da Roda NAO foi tocada"
    );
    assert_eq!(
        tint(&sim, n.rim_in_wheel_scene),
        WHITE,
        "e nenhuma outra Roda mudou"
    );
    assert_eq!(
        overrides(&sim, n.car_scene),
        0,
        "a excepcao saiu da copia da cena"
    );
    assert_eq!(
        overrides(&sim, n.wheel_in_car),
        1,
        "e passou a ser excepcao da Roda DENTRO do Carro — a regra do Unity"
    );
}

/// ⭐⭐⭐ **APLICAR AO DEGRAU DE DENTRO alcança TODA Roda em todo o lado** — o critério 4 da F5.
///
/// E a prova de que a metade não-opcional está lá: **nenhuma excepção sobra** em degrau nenhum.
/// Sem ela, a chave que ficasse na cópia da Roda dentro do Carro bloquearia o valor que o gesto
/// acabou de aplicar, e o artista veria-o **voltar atrás**.
#[test]
fn applying_to_the_inner_master_reaches_every_copy_of_it() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = MasterEcho::default();
    let n = nested_car(&mut sim, &r);
    pass(&mut sim, &r, &bridge, &mut echo);

    paint(&mut sim, n.rim_in_scene, BLUE);
    pass(&mut sim, &r, &bridge, &mut echo);

    let done = apply_at(&mut sim, &r, &mut echo, n.rim_in_scene, n.wheel_master);
    assert_eq!((done.changed, done.left), (1, 0));
    pass(&mut sim, &r, &bridge, &mut echo);

    assert_eq!(
        tint(&sim, n.rim_in_wheel),
        BLUE,
        "a receita da Roda recebeu"
    );
    assert_eq!(
        tint(&sim, n.rim_in_wheel_scene),
        BLUE,
        "e TODA Roda em todo o lado mudou"
    );
    assert_eq!(tint(&sim, n.rim_in_car), BLUE, "a Roda dentro do Carro");
    assert_eq!(tint(&sim, n.rim_in_scene), BLUE, "e a peca que foi clicada");
    assert_eq!(
        (
            overrides(&sim, n.car_scene),
            overrides(&sim, n.wheel_in_car),
            overrides(&sim, n.wheel_scene)
        ),
        (0, 0, 0),
        "nenhuma excepcao sobrou em degrau nenhum"
    );
}

/// ⭐⭐⭐ **APLICAR À RODA APAGA A EXCEPÇÃO DO CARRO** — a metade que a regra do Unity diz não ser
/// opcional, com o cenário em que ela morde: o artista já tinha mexido na roda **dentro** do
/// Carro, e agora promove a cor da cena até à receita da Roda.
///
/// > *«If Apply to Prefab 'Vase' is chosen and the 'Table' Prefab has an override of the value,
/// > this override in the 'Table' Prefab is **reverted at the same time** so that the property on
/// > the instance retains the value that was just applied.»*
///
/// ⚠️ **A régua forte não é o valor, é QUEM MANDA a seguir**: com a chave intermédia por apagar, o
/// valor até fica certo (o gesto escreve-o em todos os degraus) e a cópia da Roda dentro do Carro
/// fica **surda à receita da Roda para sempre**. ⇒ o gate mexe na Roda **outra vez**, e é essa
/// segunda edição que morre sem a cura.
#[test]
fn applying_to_the_inner_master_clears_the_override_in_the_middle() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = MasterEcho::default();
    let n = nested_car(&mut sim, &r);
    pass(&mut sim, &r, &bridge, &mut echo);

    // O artista mexeu na roda DENTRO do Carro: nasce a excepção intermédia.
    paint(&mut sim, n.rim_in_car, [0.9, 0.1, 0.1, 1.0]);
    pass(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(overrides(&sim, n.wheel_in_car), 1, "a excepcao do meio");

    // E agora pinta a cópia da cena e promove-a até à receita da Roda.
    paint(&mut sim, n.rim_in_scene, BLUE);
    pass(&mut sim, &r, &bridge, &mut echo);
    let done = apply_at(&mut sim, &r, &mut echo, n.rim_in_scene, n.wheel_master);
    assert_eq!((done.changed, done.left), (1, 0));
    pass(&mut sim, &r, &bridge, &mut echo);

    assert_eq!(
        overrides(&sim, n.wheel_in_car),
        0,
        "a excepcao do meio tinha de sair JUNTO"
    );
    assert_eq!(tint(&sim, n.rim_in_wheel), BLUE);
    assert_eq!(tint(&sim, n.rim_in_car), BLUE);
    assert_eq!(tint(&sim, n.rim_in_scene), BLUE);

    // ⭐⭐ A régua que só a cura passa: a receita da Roda volta a MANDAR nos três sítios.
    paint(&mut sim, n.rim_in_wheel, [0.1, 0.9, 0.1, 1.0]);
    pass(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(
        (
            tint(&sim, n.rim_in_car),
            tint(&sim, n.rim_in_scene),
            tint(&sim, n.rim_in_wheel_scene)
        ),
        (
            [0.1, 0.9, 0.1, 1.0],
            [0.1, 0.9, 0.1, 1.0],
            [0.1, 0.9, 0.1, 1.0]
        ),
        "a receita da Roda ficou a mandar em todo o lado"
    );
}

/// ⭐⭐ **E o valor NÃO volta atrás nos passes seguintes** — o *no-op visível* que a regra do Unity
/// nomeia, medido do lado de cá: um passe extra não pode ressuscitar excepção nenhuma.
///
/// ⚠️ **Dois passes, e não um**: o passe não corre em ordem topológica (ele ordena as instâncias
/// por `StableId`), então uma cura que dependesse da ordem passaria no primeiro e falharia no
/// segundo — ou ao contrário.
#[test]
fn the_applied_value_does_not_spring_back_on_the_next_passes() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = MasterEcho::default();
    let n = nested_car(&mut sim, &r);
    pass(&mut sim, &r, &bridge, &mut echo);
    paint(&mut sim, n.rim_in_scene, BLUE);
    pass(&mut sim, &r, &bridge, &mut echo);
    apply_at(&mut sim, &r, &mut echo, n.rim_in_scene, n.wheel_master);

    for round in 0..3 {
        pass(&mut sim, &r, &bridge, &mut echo);
        assert_eq!(tint(&sim, n.rim_in_scene), BLUE, "passe {round}");
        assert_eq!(
            (
                overrides(&sim, n.car_scene),
                overrides(&sim, n.wheel_in_car)
            ),
            (0, 0),
            "passe {round}: uma excepcao renasceu"
        );
    }
}

/// ⭐⭐⭐ **E ELE NÃO ESPERA PELO QUADRO SEGUINTE** — a régua que só uma ordem de passe
/// **não-topológica** consegue fazer.
///
/// O passe ordena as instâncias por `StableId`, e na receita normal de aninhamento a cópia interna
/// nasce **antes** do mestre externo ⇒ a ordem sai topológica **por coincidência**. Uma segunda
/// Roda metida no Carro **depois** de ele já ser receita inverte-a: a cópia da cena é avaliada
/// primeiro, e nesse instante o degrau do meio ainda tem o valor velho.
///
/// ⇒ sem a escrita em cada degrau, o valor aplicado é **substituído pelo antigo durante um quadro**
/// e só volta no seguinte — que é, à letra, o *«the value on the instance would change right after
/// being applied»* que a regra do Unity existe para impedir.
///
/// ⚠️ **Foi uma mutação SOBREVIVENTE que pediu esta fixtura**: apagar a escrita intermédia deixava
/// os outros sete gates verdes, porque em todos eles a ordem por identidade calhava certa. *Uma
/// ordem que só COINCIDE com a certa não é a certa.*
#[test]
fn the_applied_value_lands_in_the_same_pass_even_out_of_topological_order() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = MasterEcho::default();
    let n = nested_car(&mut sim, &r);
    pass(&mut sim, &r, &bridge, &mut echo);

    // Uma SEGUNDA Roda metida no Carro depois de ele já ser receita — identidade mais alta que a
    // do Carro da cena, logo o passe avalia a de fora primeiro.
    let wheel2_in_car = {
        let (mut sc, mut mp) = crate::instance_docs::empty_docs();
        crate::instantiate::instantiate_master(
            &mut sim,
            &r,
            n.wheel_master,
            Some(n.car_master),
            &mut crate::instance_docs::OwnedDocs {
                vec_scene: &mut sc,
                vec_entities: &mut mp,
            },
            crate::instantiate::ArtLink::Own,
        )
        .expect("mais uma Roda dentro do Carro")
    };
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    // O passe estrutural materializa a peça nova na cópia da cena.
    pass(&mut sim, &r, &bridge, &mut echo);
    let rim2_in_car = copy_of(&sim, wheel2_in_car, n.rim_in_wheel);
    let rim2_in_scene = copy_of(&sim, n.car_scene, rim2_in_car);
    assert!(
        id_of(&sim, n.car_scene) < id_of(&sim, wheel2_in_car),
        "a fixtura so' mede o que promete se a ordem do passe for a INVERSA da dependencia"
    );

    paint(&mut sim, rim2_in_scene, BLUE);
    pass(&mut sim, &r, &bridge, &mut echo);
    apply_at(&mut sim, &r, &mut echo, rim2_in_scene, n.wheel_master);

    // ⭐ UM passe, e o valor está lá — não dois.
    pass(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(
        tint(&sim, rim2_in_scene),
        BLUE,
        "o valor aplicado voltou atras durante um quadro"
    );
    assert_eq!(
        tint(&sim, n.rim_in_wheel),
        BLUE,
        "a receita da Roda recebeu"
    );
}

/// O `StableId` de uma entidade — a chave por que o passe ordena as instâncias.
fn id_of(sim: &SimWorld, e: Entity) -> u64 {
    sim.world()
        .get::<ph2d_ecs::StableId>(e)
        .expect("identidade")
        .0
}

/// ⛔ **Uma excepção cuja escada NÃO alcança a receita escolhida fica onde está** — e é CONTADA.
///
/// A carroçaria é peça do Carro e de mais nada; pedir *aplicar à Roda* sobre a instância inteira
/// não a pode empurrar para uma receita de que ela nunca foi cópia. ⛔ Aplicá-la ao degrau mais
/// fundo que houvesse seria escolher pelo artista, que é o que a escada existe para não fazer.
#[test]
fn an_override_whose_ladder_misses_the_target_is_left_and_counted() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = MasterEcho::default();
    let n = nested_car(&mut sim, &r);
    // Uma peça que só o Carro tem.
    let body = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Body"),
            Sprite::atlas(WHITE_TILE_KEY, [1.0, 0.4], WHITE),
            ChildOf(n.car_master),
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    pass(&mut sim, &r, &bridge, &mut echo);
    let body_in_scene = copy_of(&sim, n.car_scene, body);

    paint(&mut sim, body_in_scene, BLUE);
    paint(&mut sim, n.rim_in_scene, BLUE);
    pass(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(overrides(&sim, n.car_scene), 2);

    // O gesto na RAIZ apanha as duas chaves; só uma delas alcança a Roda.
    let done = apply_at(&mut sim, &r, &mut echo, n.car_scene, n.wheel_master);
    assert_eq!(
        (done.changed, done.left),
        (1, 1),
        "a da Roda aplicou-se, a da carrocaria ficou — e foi contada"
    );
    pass(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(tint(&sim, n.rim_in_wheel), BLUE, "a Roda recebeu");
    assert_eq!(tint(&sim, body), WHITE, "a carrocaria do Carro nao");
    assert_eq!(
        overrides(&sim, n.car_scene),
        1,
        "e a excepcao dela continua a ser dela"
    );
}
