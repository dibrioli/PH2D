//! Os gates da peça ACRESCENTADA a uma cópia (ADR-0164 / F5.11) — o *Added GameObject*.
//!
//! ⚠️ **O oráculo é a ÁRVORE das OUTRAS cópias depois do passe**, e nunca *«o verbo devolveu 1»*:
//! o gesto existe para a peça chegar às irmãs, e um gate que contasse promoções ficaria verde sobre
//! uma promoção que põe a peça no pai errado — ou que a põe duas vezes na cópia onde se trabalhou.

use super::{AddRefusal, added_pieces, promote};
use crate::instance_docs::OwnedDocs;
use crate::instance_sync::{MasterEcho, sync_instances};
use ph2d_ecs::{ChildOf, Children, Entity, MasterRoot, Name, SimWorld, Transform};
use ph2d_physics_ecs::PhysicsBridge;

fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

fn pass(sim: &mut SimWorld, r: &ph2d_ecs::scene::ComponentRegistry, echo: &mut MasterEcho) {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    sync_instances(
        sim,
        r,
        &PhysicsBridge::new(),
        echo,
        &mut OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
    );
}

fn instantiate(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    master: Entity,
    parent: Option<Entity>,
    link: crate::instantiate::ArtLink,
) -> Entity {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    crate::instantiate::instantiate_master(
        sim,
        r,
        master,
        parent,
        &mut OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        link,
    )
    .expect("instanciou")
}

fn apply(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    e: Entity,
) -> Result<super::Promoted, AddRefusal> {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    promote(
        sim,
        r,
        &mut OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        e,
    )
}

/// Uma receita de uma peça (`Robot` > `Body`), e uma cópia dela.
fn scene() -> (SimWorld, ph2d_ecs::scene::ComponentRegistry, Entity, Entity) {
    let mut sim = SimWorld::new();
    let r = reg();
    let master = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Robot"), MasterRoot))
        .id();
    sim.world_mut().spawn((
        Transform::IDENTITY,
        Name::new("Body"),
        ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
        ChildOf(master),
    ));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    let inst = instantiate(&mut sim, &r, master, None, crate::instantiate::ArtLink::Own);
    (sim, r, master, inst)
}

/// Pendura uma entidade nova debaixo de `parent` — o que um *Add Child* ou um *Duplicate* deixa.
fn hang(sim: &mut SimWorld, parent: Entity, name: &str) -> Entity {
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new(name), ChildOf(parent)))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    e
}

fn names(sim: &SimWorld, root: Entity) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        if e != root
            && let Some(n) = sim.world().get::<Name>(e)
        {
            out.push(n.0.clone());
        }
        if let Some(kids) = sim.world().get::<Children>(e) {
            stack.extend(kids.iter().copied());
        }
    }
    out.sort();
    out
}

/// ⭐⭐⭐ **Uma entidade sem elo dentro de uma cópia é uma peça ACRESCENTADA** — e a lista nomeia a
/// receita que a receberia.
///
/// (Mutação: tirar o `if sim.world().get::<InstanceOf>(kid).is_some() { … continue }` do
/// `added_pieces` ⇒ RED, porque o `Body` passa a entrar na lista.)
#[test]
fn a_piece_the_artist_hung_inside_a_copy_is_listed_as_added() {
    let (mut sim, _r, master, inst) = scene();
    hang(&mut sim, inst, "Hat");
    let rows = added_pieces(&mut sim, inst);
    assert_eq!(
        rows.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(),
        vec!["Hat"],
        "a lista das pecas acrescentadas nao e' exactamente a que o artista pendurou"
    );
    let master_id = sim
        .world()
        .get::<ph2d_ecs::StableId>(master)
        .expect("a receita tem identidade")
        .0;
    assert_eq!(
        rows[0].master, master_id,
        "a linha aponta para outra receita"
    );
    assert_eq!(rows[0].master_name, "Robot", "a linha nao nomeia a receita");
}

/// ⛔ **A peça que a RECEITA DEU nunca é uma peça acrescentada** — é a negação exacta do
/// `is_a_recipe_given_piece`, e as duas leem o mesmo campo.
#[test]
fn a_piece_the_recipe_gave_is_not_listed_as_added() {
    let (mut sim, _r, _master, inst) = scene();
    assert!(
        added_pieces(&mut sim, inst).is_empty(),
        "uma copia intacta ja' aparece com pecas acrescentadas"
    );
}

/// ⭐⭐⭐ **APLICAR põe a peça na receita — e é por isso que as IRMÃS a recebem.**
///
/// ⚠️ **O oráculo é a segunda cópia**, e não o mestre: pôr a peça na receita sem que o passe a
/// materialize seria o gesto pela metade, e a metade que o artista vê é a que falta.
///
/// (Mutação: não chamar o `deep_copy_subtree` no `promote_piece` ⇒ RED.)
#[test]
fn applying_an_added_piece_gives_it_to_every_other_copy() {
    let (mut sim, r, _master, inst) = scene();
    let other = instantiate(
        &mut sim,
        &r,
        _master,
        None,
        crate::instantiate::ArtLink::Own,
    );
    let hat = hang(&mut sim, inst, "Hat");
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    assert!(
        !names(&sim, other).contains(&"Hat".to_string()),
        "a segunda copia ja' tinha o chapeu ANTES de alguem o aplicar"
    );
    apply(&mut sim, &r, hat).expect("promoveu");
    pass(&mut sim, &r, &mut echo);
    assert!(
        names(&sim, other).contains(&"Hat".to_string()),
        "a peca aplicada nao chegou a' outra copia — a receita nao a tem, ou o passe nao a \
         materializa"
    );
}

/// ⭐⭐⭐ **E a cópia onde o artista trabalhou NÃO fica com duas.**
///
/// ⚠️ É esta a metade que o elo compra. Sem ele o passe seguinte vê uma peça do mestre que esta
/// cópia *«não tem»* e materializa uma segunda — **na cópia onde o artista acabou de trabalhar**.
///
/// (Mutação: tirar o `insert(InstanceOf { master: id })` do `promote_piece` ⇒ RED com dois
/// chapéus.)
#[test]
fn the_copy_that_applied_does_not_get_the_piece_twice() {
    let (mut sim, r, _master, inst) = scene();
    let hat = hang(&mut sim, inst, "Hat");
    apply(&mut sim, &r, hat).expect("promoveu");
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    pass(&mut sim, &r, &mut echo);
    let hats = names(&sim, inst)
        .iter()
        .filter(|n| n.as_str() == "Hat")
        .count();
    assert_eq!(
        hats, 1,
        "a copia que aplicou ficou com {hats} chapeus — o elo nao nasceu no original"
    );
}

/// ⛔⛔ **A peça nova da receita NÃO DESENHA.**
///
/// A receita está escondida por não ter nenhuma peça visível fora do `MasterPiece`; uma peça
/// promovida sem a marca apareceria em cima da biblioteca, e o artista veria o chapéu duas vezes na
/// tela — uma vez na cópia dele e outra no ar.
///
/// (Mutação: tirar o `assign_master_pieces` do `promote_piece` ⇒ RED.)
#[test]
fn the_promoted_piece_is_marked_as_a_recipe_piece_so_it_does_not_draw() {
    let (mut sim, r, master, inst) = scene();
    let hat = hang(&mut sim, inst, "Hat");
    apply(&mut sim, &r, hat).expect("promoveu");
    let in_master: Vec<Entity> = sim
        .world()
        .get::<Children>(master)
        .map(|k| k.to_vec())
        .unwrap_or_default();
    let promoted = in_master
        .iter()
        .copied()
        .find(|&e| sim.world().get::<Name>(e).is_some_and(|n| n.0 == "Hat"))
        .expect("a receita ganhou a peca");
    assert!(
        sim.world().get::<ph2d_ecs::MasterPiece>(promoted).is_some(),
        "a peca promovida nao esta' marcada como peca de receita — ela vai DESENHAR"
    );
    assert!(
        sim.world().get::<MasterRoot>(promoted).is_none(),
        "a peca promovida nasceu como uma receita propria"
    );
}

/// ⭐⭐ **Aplicar um FILHO de uma peça acrescentada aplica o TOPO da cadeia.**
///
/// Promover só a de dentro poria na receita uma peça cujo pai lá não existe. ⚠️ A normalização é o
/// que apaga a recusa *«aplique o pai primeiro»* — *uma pergunta que a normalização responde não
/// precisa de uma voz.*
///
/// (Mutação: devolver `entity` no `top_of_added_chain` ⇒ RED.)
#[test]
fn applying_the_child_of_an_added_piece_applies_the_top_of_the_chain() {
    let (mut sim, r, master, inst) = scene();
    let hat = hang(&mut sim, inst, "Hat");
    let feather = hang(&mut sim, hat, "Feather");
    apply(&mut sim, &r, feather).expect("promoveu");
    let in_master = names(&sim, master);
    assert!(
        in_master.contains(&"Hat".to_string()) && in_master.contains(&"Feather".to_string()),
        "a receita ficou com {in_master:?} — o topo da cadeia nao foi o sujeito"
    );
}

/// ⛔ **Uma peça que a receita deu recusa com voz própria** — e não com o mesmo `None` das outras.
#[test]
fn a_recipe_given_piece_is_refused_by_the_verb() {
    let (mut sim, r, _master, inst) = scene();
    let body = sim
        .world()
        .get::<Children>(inst)
        .map(|k| k.to_vec())
        .unwrap_or_default()[0];
    assert_eq!(apply(&mut sim, &r, body), Err(AddRefusal::NotAdded));
    assert_eq!(apply(&mut sim, &r, inst), Err(AddRefusal::NotAdded));
}

/// ⛔⛔ **Uma peça pendurada dentro de uma cópia ANINHADA pertence ao cartão DELA.**
///
/// O [`crate::instance_verbs::instance_root_of`] devolve a raiz mais INTERNA, então listá-la
/// também no cartão de fora daria a mesma linha em dois sítios, com dois destinos diferentes —
/// e o de fora estaria errado.
///
/// (Mutação: tirar o `if !is_nested_root(…)` do `added_pieces` ⇒ RED, com o parafuso a aparecer no
/// carro.)
#[test]
fn a_piece_added_inside_a_nested_copy_belongs_to_the_inner_card() {
    let (mut sim, r, _robot, car_copy) = scene();
    let wheel = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Wheel"), MasterRoot))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    // Uma cópia da Roda ARRASTADA para dentro da cópia do Robô — ela tem elo e é a raiz da cópia
    // dela, que é exactamente a forma que este gate defende.
    let wheel_copy = instantiate(
        &mut sim,
        &r,
        wheel,
        Some(car_copy),
        crate::instantiate::ArtLink::Own,
    );
    hang(&mut sim, wheel_copy, "Bolt");
    assert!(
        !added_pieces(&mut sim, car_copy)
            .iter()
            .any(|a| a.name == "Bolt"),
        "o parafuso da roda apareceu no cartao do robo"
    );
    assert!(
        added_pieces(&mut sim, wheel_copy)
            .iter()
            .any(|a| a.name == "Bolt"),
        "o parafuso nao aparece no cartao da propria roda"
    );
}

/// ⛔ **A marca da ARTE PARTILHADA não entra na receita.**
///
/// O doc do [`ph2d_ecs::LinkedArt`] diz, pelo nome, que *o mestre não a tem* — e o sync vive disso
/// (ela está no `NEVER_PROPAGATES` para o passe não a arrancar da cópia). Uma receita que a
/// carregasse seria uma receita a dizer que é cópia de si mesma.
///
/// (Mutação: tirar o `remove::<LinkedArt>()` do `promote_piece` ⇒ RED.)
#[test]
fn the_shared_art_mark_does_not_travel_into_the_recipe() {
    let (mut sim, r, master, _inst) = scene();
    let linked = instantiate(
        &mut sim,
        &r,
        master,
        None,
        crate::instantiate::ArtLink::Shared,
    );
    let hat = hang(&mut sim, linked, "Hat");
    sim.world_mut().entity_mut(hat).insert(ph2d_ecs::LinkedArt);
    apply(&mut sim, &r, hat).expect("promoveu");
    let promoted = sim
        .world()
        .get::<Children>(master)
        .map(|k| k.to_vec())
        .unwrap_or_default()
        .into_iter()
        .find(|&e| sim.world().get::<Name>(e).is_some_and(|n| n.0 == "Hat"))
        .expect("a receita ganhou a peca");
    assert!(
        sim.world().get::<ph2d_ecs::LinkedArt>(promoted).is_none(),
        "a receita ficou com a marca de arte partilhada"
    );
}
