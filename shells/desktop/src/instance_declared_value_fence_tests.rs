//! ⛔⛔⛔ **As cercas de «uma receita NÃO é uma cópia»** ([`super`]) — o P0 da auditoria
//! multiagêntica de 2026-08-31.
//!
//! Uma variante é `MasterRoot` **e** `InstanceOf` ao mesmo tempo, e nenhuma função da lei
//! perguntava *«isto é uma receita?»*. Estes três gates moram juntos porque partilham essa
//! premissa — e **separados** dos gates de fluxo porque cada cerca tem de ter o SEU caso: duas
//! cercas juntas escondem-se uma à outra, e uma mutação que apagasse só uma sobreviveria a todo
//! gate que passasse pela outra.
//!
//! As fixturas são as de [`super::tests`] — a mesma base+cópia que o resto da lei usa.

use super::tests::{base_and_copy, name_of, reg};
use ph2d_ecs::{InstanceOf, MasterRoot, Name};

/// ⛔⛔⛔ **UMA RECEITA NUNCA SE SEGUE A SI MESMA** — o P0 da auditoria multiagêntica de 2026-08-31.
///
/// Uma variante é `MasterRoot` **e** `InstanceOf`, e sem a cerca toda a cadeia respondia «sim» sem
/// ninguém perguntar «isto é uma receita?»: o `follow` achava-A como «irmã que declara isto» e o
/// `swap` ligava-lhe o elo a si mesma — a derivação base→variante cortada em silêncio, com um
/// clique de SELEÇÃO como porta, resistente a undo.
///
/// ⚠️ **O oráculo é o MUNDO intacto** (elo + nomes), nunca só o retorno: um `follow` que agisse E
/// devolvesse `false` passaria num gate que só olhasse o bool.
///
/// (Mutações: tirar a cerca do `follow` ⇒ RED aqui; tirar a do `swap` ⇒ RED no gate irmão da
/// porta.)
#[test]
fn a_recipe_never_follows_itself() {
    let (mut sim, base, copy) = base_and_copy("Casa {Size=Small}");
    let mut echo = crate::instance_sync::MasterEcho::default();
    let r = reg();
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let variant = crate::instantiate::instantiate_master(
        &mut sim,
        &r,
        base,
        None,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        crate::instantiate::ArtLink::Own,
    )
    .expect("instanciou");
    sim.world_mut()
        .entity_mut(variant)
        .insert((MasterRoot, Name::new("Casa {Size=Big}")));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    let base_id = sim.world().get::<ph2d_ecs::StableId>(base).expect("sid").0;

    // O gesto do P0: SELECCIONAR a variante (o follow da mudança de seleção corre sobre ela).
    assert!(
        !super::follow(&mut sim, &mut echo, variant),
        "o follow agiu sobre uma RECEITA"
    );
    assert_eq!(
        sim.world().get::<InstanceOf>(variant).map(|l| l.master),
        Some(base_id),
        "o elo da variante mexeu — ela deixou de seguir a base"
    );
    // E o commit de nome sobre ela também não a auto-troca nem toca na base.
    let out = super::apply(&mut sim, &mut echo, variant);
    assert!(
        out.is_none(),
        "apply agiu sobre uma receita como se fosse copia"
    );
    assert_eq!(
        sim.world().get::<InstanceOf>(variant).map(|l| l.master),
        Some(base_id)
    );
    assert_eq!(
        name_of(&sim, base),
        "Casa {Size=Small}",
        "a base foi renomeada por tabela"
    );
    let _ = copy;
}

/// ⛔⛔ **A PORTA do `swap` recusa «tornar-se mestre de si mesma»** — a cerca no lado perigoso.
///
/// ⚠️ **Este gate existe porque duas cercas juntas escondem-se uma à outra**: com a do `follow`
/// no lugar, nenhum chamador de produto alcança o self-swap — e uma mutação que apagasse SÓ a da
/// porta sobreviveria a todos os gates de fluxo. *Sonda de pares é cega ao membro sozinho*; cada
/// cerca tem o seu.
///
/// (Mutação: tirar o `ItselfAsMaster` do `swap` ⇒ RED.)
#[test]
fn the_swap_door_refuses_itself_as_master() {
    let (mut sim, base, _copy) = base_and_copy("Casa {Size=Small}");
    let mut echo = crate::instance_sync::MasterEcho::default();
    let r = reg();
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let variant = crate::instantiate::instantiate_master(
        &mut sim,
        &r,
        base,
        None,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        crate::instantiate::ArtLink::Own,
    )
    .expect("instanciou");
    sim.world_mut()
        .entity_mut(variant)
        .insert((MasterRoot, Name::new("Casa {Size=Big}")));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    let own = sim
        .world()
        .get::<ph2d_ecs::StableId>(variant)
        .expect("sid")
        .0;
    let base_id = sim.world().get::<ph2d_ecs::StableId>(base).expect("sid").0;

    assert!(
        matches!(
            crate::instance_variant::swap(&mut sim, &mut echo, variant, own),
            Err(crate::instance_variant::SwapRefusal::ItselfAsMaster)
        ),
        "a porta aceitou ligar uma variante a si mesma"
    );
    assert_eq!(
        sim.world().get::<InstanceOf>(variant).map(|l| l.master),
        Some(base_id),
        "o elo mexeu mesmo com a recusa"
    );
}

/// ⛔⛔ **A cerca do `follow` tem o SEU caso, e ele não é o do `swap`** — a mutação que a apagava
/// SOBREVIVEU aos doze gates, porque nos estados saudáveis a porta do `swap` (`ItselfAsMaster`)
/// tapa o buraco.
///
/// O caso que só ela cobre: **duas receitas com a MESMA combinação** (um estado degenerado que
/// outras curas tentam impedir — mas *tentam impedir* não é *não existe*: um load velho, um bug
/// vizinho). Sem a cerca, seleccionar a variante fazia o `follow` achar a GÉMEA de combinação e
/// trocar o elo da RECEITA para ela — a variante passava a derivar de outra receita, em silêncio.
///
/// ⚠️ *Uma cerca cuja mutação não mata gate nenhum é uma afirmação sobre nada* — este é o gate
/// que a torna uma afirmação sobre isto.
#[test]
fn the_follow_fence_holds_even_against_a_twin_combination() {
    let (mut sim, base, _copy) = base_and_copy("Casa {Size=Small}");
    let mut echo = crate::instance_sync::MasterEcho::default();
    let r = reg();
    // ⚠️ **A GÉMEA nasce PRIMEIRO** (StableId menor): a lista da família é ordenada por id, e o
    // `.find` sem a cerca visitá-la-ia antes da própria variante — é isso que faz a mutação
    // sangrar AQUI em vez de ser tapada pela recusa `ItselfAsMaster` da porta do swap.
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let twin = crate::instantiate::instantiate_master(
        &mut sim,
        &r,
        base,
        None,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        crate::instantiate::ArtLink::Own,
    )
    .expect("instanciou");
    sim.world_mut()
        .entity_mut(twin)
        .insert((MasterRoot, Name::new("Outra {Size=Big}")));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let variant = crate::instantiate::instantiate_master(
        &mut sim,
        &r,
        base,
        None,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        crate::instantiate::ArtLink::Own,
    )
    .expect("instanciou");
    sim.world_mut()
        .entity_mut(variant)
        .insert((MasterRoot, Name::new("Casa {Size=Big}")));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    let base_id = sim.world().get::<ph2d_ecs::StableId>(base).expect("sid").0;

    // Seleccionar a variante (o follow da mudança de seleção).
    assert!(
        !super::follow(&mut sim, &mut echo, variant),
        "o follow agiu sobre uma RECEITA"
    );
    assert_eq!(
        sim.world().get::<InstanceOf>(variant).map(|l| l.master),
        Some(base_id),
        "a variante passou a derivar da GEMEA — a cerca do follow caiu"
    );
}
