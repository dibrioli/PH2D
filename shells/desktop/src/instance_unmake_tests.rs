//! Os gates de **tirar da biblioteca** ([`super`]).
//!
//! ⚠️ **O oráculo é o que o ARTISTA fica a ver**, nunca o que a função devolve: quantos objectos
//! sobraram na cena, se ainda seguem alguma receita, e se a receita continua a ser uma. Um gate
//! que lesse só o `Unmade` ficaria verde sobre um verbo que devolve o veredicto certo e deixa o
//! mundo errado.

use super::{Unmade, UnmakeRefusal, unmake_master};
use crate::instance_smoke::spawn_master;
use ph2d_ecs::{Entity, InstanceOf, MasterRoot, SimWorld, StableId};

fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

/// Uma receita com `n` cópias na cena.
///
/// ⚠️ **As cópias nascem pela porta do produto** (`instantiate_master`), nunca de um `spawn` com
/// `InstanceOf` escrito à mão: um par montado aqui teria a forma que este ficheiro imagina, e o
/// gate mediria a fixtura em vez do verbo. (A receita vem do `spawn_master`, que é o mesmo
/// construtor que as cenas de smoke usam — ele já nasce `MasterRoot`, e é por isso que o
/// `make_master` recusaria com `AlreadyAMaster`.)
fn library_with(n: usize) -> (SimWorld, Entity, u64) {
    let mut sim = SimWorld::new();
    let master = spawn_master(&mut sim);
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    let master_id = sim
        .world()
        .get::<StableId>(master)
        .expect("a receita tem StableId")
        .0;
    let r = reg();
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let mut docs = crate::instance_docs::OwnedDocs {
        vec_scene: &mut sc,
        vec_entities: &mut mp,
    };
    for _ in 0..n {
        crate::instantiate::instantiate_master(
            &mut sim,
            &r,
            master,
            None,
            &mut docs,
            crate::instantiate::ArtLink::Own,
        )
        .expect("instantiate na fixtura");
        ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    }
    (sim, master, master_id)
}

/// Quantas entidades ainda apontam para `master_id`.
fn links_to(sim: &mut SimWorld, master_id: u64) -> usize {
    let mut q = sim.world_mut().query::<&InstanceOf>();
    q.iter(sim.world())
        .filter(|l| l.master == master_id)
        .count()
}

/// ⭐⭐ **A metade COM cópias: elas ficam, a receita vai.**
///
/// **Mutação que deve sangrar:** trocar o `despawn(root)` por `remove::<MasterRoot>()` — o objecto
/// invisível reapareceria no meio das cópias, que é o *«nasceu um objecto do nada»* que a lei
/// existe para não fazer.
#[test]
fn removing_a_prefab_with_copies_frees_the_copies_and_dissolves_the_recipe() {
    let (mut sim, master, master_id) = library_with(3);
    assert_eq!(links_to(&mut sim, master_id), 3, "a fixtura tem 3 cópias");

    let out = unmake_master(&mut sim, master).expect("a receita existe");
    assert_eq!(out, Unmade::Dissolved { copies: 3 });

    assert!(
        sim.world().get_entity(master).is_err(),
        "a receita invisível foi apagada"
    );
    assert_eq!(
        links_to(&mut sim, master_id),
        0,
        "nenhuma cópia continua a apontar para uma receita que já não existe"
    );
}

/// ⭐⭐⭐ **A metade SEM cópias: a receita VOLTA, e nada é destruído.**
///
/// ⚠️ Esta é a metade que impede o verbo de apagar a última cópia do trabalho do artista — ele leu
/// *«tirar da lista»* e não *«apagar»*.
///
/// **Mutação que deve sangrar:** apagar o ramo `roots.is_empty()` e despawnar sempre.
#[test]
fn removing_a_prefab_with_no_copies_brings_it_back_to_the_canvas() {
    let (mut sim, master, master_id) = library_with(0);
    assert_eq!(links_to(&mut sim, master_id), 0, "a fixtura não tem cópias");

    let out = unmake_master(&mut sim, master).expect("a receita existe");
    assert_eq!(
        out,
        Unmade::Returned {
            root_bits: master.to_bits()
        }
    );

    assert!(
        sim.world().get_entity(master).is_ok(),
        "a última cópia NÃO foi destruída"
    );
    assert!(
        sim.world().get::<MasterRoot>(master).is_none(),
        "e ela deixou de ser receita — é isso que a faz voltar a desenhar"
    );
}

/// ⚠️ **A ORDEM é load-bearing.** Os elos recolhem-se antes de a receita deixar de o ser.
///
/// **Mutação que deve sangrar:** mover o `despawn`/`remove::<MasterRoot>` para ANTES do laço de
/// `detach` — o `instance_root_of` deixaria de resolver, o `detach` devolveria `Err` para todas, e
/// as cópias ficariam com `InstanceOf` a apontar para bits mortos. Este gate lê exactamente isso:
/// o que sobra nas cópias, e não o que a função disse ter feito.
#[test]
fn no_copy_is_left_pointing_at_a_recipe_that_stopped_being_one() {
    let (mut sim, master, master_id) = library_with(2);
    let _ = unmake_master(&mut sim, master).expect("a receita existe");
    assert_eq!(links_to(&mut sim, master_id), 0);
}

// ⛔⛔ **A 1.ª versão deste gate varria o MUNDO INTEIRO** (`q.iter().count() == 0`, *«as peças de
// dentro das cópias também soltaram o elo»*), e a auditoria de 2026-08-30 mostrou que ele
// **consagrava um defeito**: num mundo com um SEGUNDO prefab instanciado dentro de uma cópia do
// primeiro, aquela barra **exigiria** que os elos alheios morressem.
//
// ⚠️ E eles morrem mesmo — é a lei pré-existente do `detach`, que percorre a sub-árvore e tira
// `InstanceOf` de tudo o que o tenha lá dentro. O que esta wave acrescentou foi um verbo de
// **arrumação de biblioteca** a disparar essa demolição, sem que o nome o anuncie. A cura de fundo
// é a F5 (aninhamento de receitas), que é a mesma fronteira que o `VerbRefusal::InsideAnInstance`
// já declara; o que se corrige aqui é a RÉGUA, para ela deixar de pedir o comportamento errado.
//
// ⇒ a barra é `links_to(master_id) == 0`: *nenhuma cópia DESTA receita continua a apontar para ela*.

/// ⭐ **O verbo aceita os DOIS sujeitos** — a receita e uma cópia dela.
///
/// ⚠️ O menu da Hierarquia é uma tabela plana, então este item aparece sobre uma instância. Sem
/// esta metade ele comeria o clique em silêncio em toda linha de instância.
#[test]
fn removing_from_the_library_works_when_a_copy_is_what_was_clicked() {
    let (mut sim, master, master_id) = library_with(2);
    let copy = {
        let mut q = sim.world_mut().query::<(Entity, &InstanceOf)>();
        q.iter(sim.world())
            .find(|(_, l)| l.master == master_id)
            .map(|(e, _)| e)
            .expect("há uma cópia")
    };
    let out = unmake_master(&mut sim, copy).expect("uma cópia endereça a receita dela");
    assert_eq!(out, Unmade::Dissolved { copies: 2 });
    assert!(sim.world().get_entity(master).is_err());
}

/// Um objecto qualquer não está na biblioteca, e o verbo diz isso em vez de fazer alguma coisa.
#[test]
fn a_plain_object_is_not_in_the_library() {
    let mut sim = SimWorld::new();
    let e = spawn_master(&mut sim);
    sim.world_mut().entity_mut(e).remove::<MasterRoot>();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    assert_eq!(
        unmake_master(&mut sim, e),
        Err(UnmakeRefusal::NotInTheLibrary)
    );
}
