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

pub(super) fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

/// Um passe de sync, com o par de documentos vazio.
pub(super) fn pass(
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
pub(super) struct Nested {
    /// A receita da Roda, e a peça dela.
    pub(super) wheel_master: Entity,
    pub(super) rim_in_wheel: Entity,
    /// A receita do Carro, a cópia da Roda que vive dentro dela, e a peça dessa cópia.
    pub(super) car_master: Entity,
    pub(super) wheel_in_car: Entity,
    pub(super) rim_in_car: Entity,
    /// O Carro da CENA e a peça mais funda dele — a que o artista pinta.
    pub(super) car_scene: Entity,
    pub(super) rim_in_scene: Entity,
    /// ⭐ **Uma Roda SOLTA na cena** — a testemunha de *«toda Roda em todo o lado»*. Sem ela os
    /// dois degraus da escada são indistinguíveis: o que os separa não é o que acontece ao Carro,
    /// é o que acontece a quem NÃO está dentro dele.
    pub(super) wheel_scene: Entity,
    pub(super) rim_in_wheel_scene: Entity,
}

pub(super) fn nested_car(sim: &mut SimWorld, r: &ph2d_ecs::scene::ComponentRegistry) -> Nested {
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
    sim.world_mut()
        .entity_mut(wheel_in_car)
        .insert(ChildOf(car));
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
pub(super) fn copy_of(sim: &SimWorld, root: Entity, origin: Entity) -> Entity {
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
pub(super) fn piece(sim: &SimWorld, root: Entity, name: &str) -> Entity {
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

pub(super) fn tint(sim: &SimWorld, e: Entity) -> [f32; 4] {
    sim.world().get::<Sprite>(e).expect("sprite").tint
}

pub(super) fn paint(sim: &mut SimWorld, e: Entity, c: [f32; 4]) {
    let mut spr = sim.world().get::<Sprite>(e).copied().expect("sprite");
    spr.tint = c;
    sim.world_mut().entity_mut(e).insert(spr);
}

pub(super) fn overrides(sim: &SimWorld, root: Entity) -> usize {
    sim.world()
        .get::<ObjectInstance>(root)
        .map_or(0, |o| o.overrides.len())
}

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

/// ⚠️ **Os gates do VERBO moram no irmão** — corte por ASSUNTO imposto pelo tecto de 600 LOC do
/// shell (`shell_files_respect_hr18_loc_cap`): aqui fica a fixtura aninhada, a ESCADA que se LÊ e o
/// mecanismo que torna o critério 4 real; lá, o que o *Aplicar* FAZ em cada degrau. A fixtura é
/// partilhada, e não duplicada — duas montagens do mesmo aninhamento divergiriam em silêncio.
#[path = "instance_apply_deep_verb_tests.rs"]
mod verb;
