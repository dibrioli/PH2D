//! **A migração da opção B** (GEF02 B2, schema `219 → 220`): num ficheiro `<= 219` a forma de colisão
//! mora no próprio corpo; desde o `220` mora numa filha «Collision Shape». Plano:
//! `docs/GameEngineFuncional/componentes/f2_contorno_e_formas_plano.md` («REVISÃO 08/11»).
//!
//! Lei: por `StableId`, cada corpo (`RigidBody` + `Collider` na mesma entidade) ganha uma filha
//! invisível com `Transform` identidade, PRIMEIRA entre as irmãs (logo a forma principal de
//! [`crate::main_shape`]), que leva o `Collider` e todo o lado-forma ([`LADO_FORMA`]). Nenhum nome
//! muda. Idempotente: um corpo sem `Collider` próprio não é tocado.
//!
//! Instâncias: uma peça com `InstanceOf` nunca é migrada — o mestre é, e a reconciliação materializa
//! a filha na instância. ⛔ Um corpo do mestre cujo lado-forma uma instância SOBREPÕE fica como está
//! (o motor ainda aceita a forma no corpo) e conta em [`Migracao::recusados`]: re-chavear o override
//! perderia o valor da instância, porque a filha materializa-se com o do mestre.

use bevy_ecs::prelude::{Entity, World};
use ph2d_ecs::{
    ChildOf, InstanceOf, Name, ObjectInstance, StableId, Transform, Visibility, ordered_children,
    set_sibling_order,
};

/// O nome da filha que a migração cria (o mesmo do menu Add).
pub const NOME_DA_FORMA: &str = "Collision Shape";

macro_rules! lado_forma {
    ($($t:ident),+ $(,)?) => {
        /// ⭐ **O lado-FORMA, a fonte única:** o `Collider` e cada componente que a ponte lê da forma
        /// (`bridge/bodies.rs`, `signals.rs`, `surfaces.rs`). As linhas `row_of("ph2d::physics::
        /// Collider")` do catálogo são um subconjunto (gate `o_lado_forma_cobre_o_catalogo`).
        pub const LADO_FORMA: &[&str] = &[$(concat!("ph2d::physics::", stringify!($t))),+];

        fn move_lado_forma(world: &mut World, de: Entity, para: Entity) {
            $(
                if let Some(c) = world.entity_mut(de).take::<crate::$t>() {
                    world.entity_mut(para).insert(c);
                }
            )+
        }
    };
}

lado_forma!(
    Collider,
    MaterialCombine,
    OneWayPlatform,
    WalkSurface,
    SignalOnHit,
    SignalOnLeave,
    SignalTagFilter,
    AreaEffector,
    AreaDrag,
    AreaBuoyancy,
    AreaFormDrag,
    AreaTorque,
    AreaForceWorldAxes,
    AreaFalloff,
);

/// A filha «Collision Shape» vazia de `corpo`: invisível, `Transform` identidade, PRIMEIRA entre as
/// irmãs (logo a forma principal de [`crate::main_shape`]). A fonte única da migração e das portas.
fn filha_vazia(world: &mut World, corpo: Entity) -> Entity {
    let irmas = ordered_children(world, corpo);
    let filha = world
        .spawn((
            Name::new(NOME_DA_FORMA),
            Transform::IDENTITY,
            Visibility::hidden(),
            ChildOf(corpo),
        ))
        .id();
    let ordem: Vec<Entity> = std::iter::once(filha).chain(irmas).collect();
    set_sibling_order(world, corpo, &ordem);
    filha
}

/// ⭐ **O corpo nasce com a sua forma** (opção B, B3a): a filha «Collision Shape» de `body` com um
/// `Collider` de forma `shape` (o resto no ponto neutro) e `StableId`. A ÚNICA porta do editor que
/// cria a forma de um corpo — nenhuma põe `Collider` no próprio corpo (gate
/// `nenhuma_porta_do_editor_poe_a_forma_no_corpo`).
pub fn nasce_com_forma(world: &mut World, body: Entity, shape: crate::ColliderShape) -> Entity {
    let filha = filha_vazia(world, body);
    world.entity_mut(filha).insert(crate::Collider {
        shape,
        ..crate::Collider::default()
    });
    ph2d_ecs::assign_missing_stable_ids(world);
    filha
}

/// O que a migração fez: `(StableId do corpo, StableId da filha)` por corpo, na ordem de `StableId`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Migracao {
    pub pares: Vec<(u64, u64)>,
    /// Corpos do mestre deixados como estavam porque uma instância sobrepõe o lado-forma deles.
    pub recusados: usize,
}

/// Move a forma de cada corpo para uma filha «Collision Shape» (ver o módulo). Devolve quantas
/// moveu; o detalhe, em [`migra_formas_para_filhos_com_pares`].
pub fn migra_formas_para_filhos(world: &mut World) -> usize {
    migra_formas_para_filhos_com_pares(world).pares.len()
}

/// [`migra_formas_para_filhos`] com os pares corpo → filha (para re-alvejar a Timeline).
pub fn migra_formas_para_filhos_com_pares(world: &mut World) -> Migracao {
    ph2d_ecs::assign_missing_stable_ids(world);
    let tipos: Vec<u64> = LADO_FORMA
        .iter()
        .map(|n| ph2d_ecs::scene::registry::stable_type_id(n))
        .collect();
    let sobrepostos: std::collections::BTreeSet<u64> = world
        .query::<&ObjectInstance>()
        .iter(world)
        .flat_map(|i| i.overrides.iter())
        .filter(|k| tipos.contains(&k.type_id))
        .map(|k| k.piece)
        .collect();
    let mut corpos: Vec<(u64, Entity)> = world
        .query_filtered::<(Entity, &StableId), (
            bevy_ecs::query::With<crate::RigidBody>,
            bevy_ecs::query::With<crate::Collider>,
            bevy_ecs::query::Without<InstanceOf>,
        )>()
        .iter(world)
        .map(|(e, s)| (s.0, e))
        .collect();
    corpos.sort_unstable_by_key(|&(s, _)| s);
    let mut out = Migracao::default();
    let mut filhas = Vec::with_capacity(corpos.len());
    for (sid, corpo) in corpos {
        if sobrepostos.contains(&sid) {
            out.recusados += 1;
            continue;
        }
        let filha = filha_vazia(world, corpo);
        move_lado_forma(world, corpo, filha);
        filhas.push((sid, filha));
    }
    // Ids na ordem do laço: as filhas nasceram por ordem de `StableId` do corpo, e a varredura
    // atribui pela ordem de spawn.
    ph2d_ecs::assign_missing_stable_ids(world);
    out.pares = filhas
        .into_iter()
        .filter_map(|(sid, f)| world.get::<StableId>(f).map(|s| (sid, s.0)))
        .collect();
    out
}

/// A mesma migração sobre o RETRATO gravado (o load lê o ficheiro antes de haver mundo): monta-o num
/// mundo, migra e regrava. `contador` é o `stable_id_counter` do ficheiro, que só sobe.
///
/// # Errors
/// O retrato não se monta ou não se regrava com este registo.
pub fn migra_formas_no_retrato(
    snap: &mut ph2d_ecs::scene::WorldSnapshot,
    contador: &mut u64,
    reg: &ph2d_ecs::scene::ComponentRegistry,
) -> Result<Migracao, ph2d_ecs::scene::SaveError> {
    let mut sim = ph2d_ecs::SimWorld::new();
    let w = sim.world_mut();
    w.insert_resource(ph2d_ecs::StableIdCounter::new(*contador));
    ph2d_ecs::scene::snapshot_to_world(w, snap, reg)?;
    let m = migra_formas_para_filhos_com_pares(w);
    if m.pares.is_empty() {
        return Ok(m);
    }
    let mut prop = ph2d_ecs::TransformPropagationState::new(w);
    let mut work = ph2d_ecs::WorklistBuf::default();
    let mut novo = ph2d_ecs::scene::WorldSnapshot::new();
    ph2d_ecs::scene::world_to_snapshot(w, &mut prop, &mut work, reg, &mut novo)?;
    *snap = novo;
    if let Some(c) = w.get_resource::<ph2d_ecs::StableIdCounter>() {
        *contador = (*contador).max(c.next_free());
    }
    Ok(m)
}
