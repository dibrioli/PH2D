//! ⭐⭐⭐ **A ESCADA DO *APLICAR*** — a fixtura ANINHADA e os gates do critério 4 da F5.
//!
//! # ⚠️ A fixtura é a parte cara, e a óbvia está ERRADA
//!
//! A receita está escrita na §F5.4 do plano, e ela custou duas fixturas falsas antes de existir:
//!
//! - ⛔ `make_master` sobre a RAIZ de uma cópia faz uma **VARIANTE** (F5 critério 2), que *segue* a
//!   base. Aninhar é **conter**. São relações diferentes, e a variante passa por todas as réguas
//!   óbvias de aninhamento sem ser uma.
//! - ⛔ A raiz da cópia aninhada acha-se pela definição da F4.3 — *a peça cujo `master` é um
//!   `MasterRoot`* — e **não** exigindo `ObjectInstance`: esse componente só nasce **depois** de
//!   haver uma excepção, então a régua óbvia não acha uma cópia acabada de criar.

use crate::instance_sync::{MasterEcho, sync_instances};
use ph2d_ecs::{ChildOf, Entity, Name, ObjectInstance, SimWorld, Transform};
use ph2d_physics_ecs::PhysicsBridge;
use ph2d_render::{Sprite, WHITE_TILE_KEY};

fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

/// Um passe de sync, com o par de documentos vazio.
fn pass(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    bridge: &PhysicsBridge,
    echo: &mut MasterEcho,
) -> usize {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    sync_instances(
        sim,
        r,
        bridge,
        echo,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
    )
}

/// ⭐⭐⭐ **A CENA ANINHADA** — a Roda, o Carro que a contém, e um Carro na cena.
///
/// ⚠️ **Os passos são a receita da §F5.4, e a ordem é load-bearing**: a Roda tem de ser receita
/// **antes** de o Carro a conter, senão o que o Carro contém é geometria e não uma cópia.
pub(crate) struct Nested {
    /// A receita da Roda, e a peça dela.
    pub(crate) wheel_master: Entity,
    pub(crate) rim_in_wheel: Entity,
    /// A receita do Carro, a cópia da Roda que vive dentro dela, e a peça dessa cópia.
    pub(crate) car_master: Entity,
    pub(crate) wheel_in_car: Entity,
    pub(crate) rim_in_car: Entity,
    /// O Carro da CENA e a peça mais funda dele — a que o artista pinta.
    pub(crate) car_scene: Entity,
    pub(crate) rim_in_scene: Entity,
    /// ⭐ **Uma Roda SOLTA na cena** — a testemunha de *«toda Roda em todo o lado»*. Sem ela os
    /// dois degraus da escada são indistinguíveis: o que os separa não é o que acontece ao Carro,
    /// é o que acontece a quem NÃO está dentro dele.
    pub(crate) wheel_scene: Entity,
    pub(crate) rim_in_wheel_scene: Entity,
}

pub(crate) fn nested_car(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
) -> Nested {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let mut docs = crate::instance_docs::OwnedDocs {
        vec_scene: &mut sc,
        vec_entities: &mut mp,
    };
    // 1. A Roda: uma raiz com uma peça pendurada.
    let wheel = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Wheel")))
        .id();
    sim.world_mut().spawn((
        Transform::IDENTITY,
        Name::new("Rim"),
        Sprite::atlas(WHITE_TILE_KEY, [0.3, 0.3], [1.0, 1.0, 1.0, 1.0]),
        ChildOf(wheel),
    ));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    // 2. A Roda vira receita; fica uma cópia dela no lugar.
    let (wheel_master, wheel_in_car) =
        crate::instance_verbs::make_master(sim, r, wheel, &mut docs).expect("Wheel vira receita");
    // 3. ⛔ NÃO `make_master(wheel_in_car)` — isso faria uma VARIANTE, que SEGUE a base. Aninhar é
    //    CONTER: nasce um pai vazio e a cópia passa a viver debaixo dele.
    let car = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Car")))
        .id();
    sim.world_mut().entity_mut(wheel_in_car).insert(ChildOf(car));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    // 4. O Carro vira receita — e a sub-árvore dele CONTÉM uma cópia da Roda.
    let (car_master, car_scene) =
        crate::instance_verbs::make_master(sim, r, car, &mut docs).expect("Car vira receita");
    // ⚠️ **A cópia aninhada da CENA acha-se pelo ELO** — pela definição da F4.3, e não pelo nome
    // (o `instantiate` dá nomes únicos à cópia) nem pelo `ObjectInstance` (que só nasce com a
    // primeira excepção).
    let rim_in_wheel = piece(sim, wheel_master, "Rim");
    let rim_in_car = piece(sim, wheel_in_car, "Rim");
    let rim_in_scene = copy_of(sim, car_scene, rim_in_car);
    // 5. E uma Roda solta na cena, para se ver quem o degrau escolhido alcança.
    let wheel_scene = crate::instantiate::instantiate_master(
        sim,
        r,
        wheel_master,
        None,
        &mut docs,
        crate::instantiate::ArtLink::Own,
    )
    .expect("uma Roda na cena");
    let rim_in_wheel_scene = copy_of(sim, wheel_scene, rim_in_wheel);
    Nested {
        wheel_master,
        rim_in_wheel,
        car_master,
        wheel_in_car,
        rim_in_car,
        car_scene,
        rim_in_scene,
        wheel_scene,
        rim_in_wheel_scene,
    }
}

/// A entidade debaixo de `root` cujo `InstanceOf` aponta para `origin` — *«a cópia de quem?»*.
fn copy_of(sim: &SimWorld, root: Entity, origin: Entity) -> Entity {
    let want = sim
        .world()
        .get::<ph2d_ecs::StableId>(origin)
        .expect("a origem tem identidade")
        .0;
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        if sim
            .world()
            .get::<ph2d_ecs::InstanceOf>(e)
            .is_some_and(|l| l.master == want)
        {
            return e;
        }
        if let Some(kids) = sim.world().get::<ph2d_ecs::Children>(e) {
            stack.extend(kids.iter().copied());
        }
    }
    panic!("nao ha' copia de {origin:?} debaixo de {root:?}");
}

/// A peça chamada `name` debaixo de `root` (a própria raiz excluída).
fn piece(sim: &SimWorld, root: Entity, name: &str) -> Entity {
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        if e != root && sim.world().get::<Name>(e).is_some_and(|n| n.0 == name) {
            return e;
        }
        if let Some(kids) = sim.world().get::<ph2d_ecs::Children>(e) {
            stack.extend(kids.iter().copied());
        }
    }
    panic!("nao ha' peca chamada {name:?}");
}

fn tint(sim: &SimWorld, e: Entity) -> [f32; 4] {
    sim.world().get::<Sprite>(e).expect("sprite").tint
}

fn paint(sim: &mut SimWorld, e: Entity, c: [f32; 4]) {
    let mut spr = sim.world().get::<Sprite>(e).copied().expect("sprite");
    spr.tint = c;
    sim.world_mut().entity_mut(e).insert(spr);
}

fn overrides(sim: &SimWorld, root: Entity) -> usize {
    sim.world()
        .get::<ObjectInstance>(root)
        .map_or(0, |o| o.overrides.len())
}

/// O *Aplicar* num degrau, com o par de documentos vazio.
fn apply_at(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    echo: &mut MasterEcho,
    clicked: Entity,
    target: Entity,
) -> super::Applied {
    let id = sim
        .world()
        .get::<ph2d_ecs::StableId>(target)
        .expect("a receita tem identidade")
        .0;
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    super::apply_to_level(
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

// ── A ESCADA ───────────────────────────────────────────────────────────────────────────────

/// ⭐⭐ **A escada tem um degrau por RECEITA, da mais externa para a mais interna** — e a da RAIZ
/// tem **um só**, que é o *«Apply All aplica sempre ao mais externo»* do Unity.
///
/// (Mutação: percorrer a cadeia ao contrário ⇒ o Carro deixa de ser o primeiro degrau e o
/// *Aplicar* de sempre passaria a mexer na Roda sem ninguém o pedir.)
#[test]
fn the_ladder_names_one_rung_per_master_outermost_first() {
    let mut sim = SimWorld::new();
    let r = reg();
    let n = nested_car(&mut sim, &r);

    let deep: Vec<String> = super::apply_levels(&mut sim, n.rim_in_scene)
        .into_iter()
        .map(|l| l.name)
        .collect();
    assert_eq!(deep, ["Car", "Wheel"], "a escada da peca funda");

    let root: Vec<String> = super::apply_levels(&mut sim, n.car_scene)
        .into_iter()
        .map(|l| l.name)
        .collect();
    assert_eq!(root, ["Car"], "a raiz alcanca so' a receita directa");

    // ⚠️ Uma cópia NÃO aninhada tem um degrau, e é por isso que a escolha só aparece quando ela
    // existe: uma escada de um degrau não é uma pergunta.
    let loose: Vec<String> = super::apply_levels(&mut sim, n.rim_in_wheel_scene)
        .into_iter()
        .map(|l| l.name)
        .collect();
    assert_eq!(loose, ["Wheel"]);
}

/// ⛔ **O que não é cópia não tem escada** — e responder com uma lista vazia é o que deixa a UI
/// não oferecer a escolha, em vez de a oferecer e recusar.
#[test]
fn what_is_not_a_copy_has_no_ladder() {
    let mut sim = SimWorld::new();
    let r = reg();
    let n = nested_car(&mut sim, &r);
    assert!(super::apply_levels(&mut sim, n.wheel_master).is_empty());
    assert!(super::apply_levels(&mut sim, n.rim_in_wheel).is_empty());
}

// ── O MECANISMO que torna o critério 4 real ────────────────────────────────────────────────

/// ⭐⭐⭐ **UMA EXCEPÇÃO NO MEIO BLOQUEIA A RECEITA DE DENTRO** — a medição que a §F5.4 deixou por
/// fazer, e que decide se a metade que a apaga é precisa.
///
/// Com uma excepção guardada na cópia da Roda que vive **dentro** do Carro, mexer na receita da
/// Roda **não chega** ao Carro: a resposta (1) do passe é *«a instância possui este componente ⇒
/// não se toca»*. ⇒ aplicar à Roda sem apagar essa chave seria o *no-op visível* que a regra do
/// Unity nomeia.
///
/// ⚠️ **O primeiro passe é obrigatório antes de pintar** — sem eco não há atribuição, e a regra do
/// 1.º encontro dá a vitória ao mestre. *Foi isto que fez a sonda anterior ler `overrides = 0` e
/// concluir que a excepção não nascia.*
#[test]
fn an_override_in_the_middle_blocks_the_inner_master() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = MasterEcho::default();
    let n = nested_car(&mut sim, &r);
    pass(&mut sim, &r, &bridge, &mut echo);

    // Pintar DENTRO da receita do Carro regista excepção na cópia da Roda que ela contém.
    paint(&mut sim, n.rim_in_car, [0.9, 0.1, 0.1, 1.0]);
    pass(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(
        overrides(&sim, n.wheel_in_car),
        1,
        "pintar dentro de um mestre regista excepcao na copia aninhada"
    );

    // E agora a receita da Roda mexe-se — e não chega.
    paint(&mut sim, n.rim_in_wheel, [0.1, 0.9, 0.1, 1.0]);
    pass(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(
        tint(&sim, n.rim_in_car),
        [0.9, 0.1, 0.1, 1.0],
        "a excepcao intermedia BLOQUEOU a receita de dentro"
    );
    assert_eq!(
        tint(&sim, n.rim_in_wheel_scene),
        [0.1, 0.9, 0.1, 1.0],
        "e a Roda solta ouviu — o bloqueio e' da copia aninhada, nao do passe"
    );
}

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
    assert_eq!(overrides(&sim, n.car_scene), 1, "a cena capturou a excepcao");

    let done = apply_at(&mut sim, &r, &mut echo, n.rim_in_scene, n.car_master);
    assert_eq!((done.changed, done.left), (1, 0));
    pass(&mut sim, &r, &bridge, &mut echo);

    assert_eq!(tint(&sim, n.rim_in_car), BLUE, "a receita do Carro recebeu");
    assert_eq!(tint(&sim, n.rim_in_scene), BLUE, "e a cena ficou como estava");
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

    assert_eq!(tint(&sim, n.rim_in_wheel), BLUE, "a receita da Roda recebeu");
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
    assert_eq!(tint(&sim, n.rim_in_wheel), BLUE, "a receita da Roda recebeu");
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
