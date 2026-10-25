//! Os gates da [`super`] — a família LISTA de ponta a ponta (F30 §A1).

use super::*;
use crate::component_registry_for_tests::registo;
use ph2d_component_desc::lista::codigo;
use ph2d_ecs::{Timer, Timers, Transform};

fn timer(nome: &str, s: u64) -> Timer {
    Timer {
        name: nome.into(),
        duration_us: s * 1_000_000,
        ..Timer::default()
    }
}

/// Um objecto com três timers, já com ids.
fn cena() -> (SimWorld, Entity, [ItemId; 3]) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Timers(vec![timer("a", 1), timer("b", 2), timer("c", 3)]),
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let t = sim.world().get::<Timers>(e).expect("timers");
    let ids = [t.0[0].item, t.0[1].item, t.0[2].item];
    (sim, e, ids)
}

fn duracao(id: ItemId) -> FieldKey {
    cl::chave(codigo::TIMERS, 2, 0, id).expect("cabe")
}

/// A curva escreve `x` em `k` pelo funil inteiro (o `CanaisEscritos` e o `aplicar`).
fn curva(sim: &mut SimWorld, reg: &ComponentRegistry, e: Entity, k: FieldKey, x: f32) -> bool {
    sim.world_mut()
        .get_resource_or_insert_with(CanaisEscritos::default)
        .escrever(e.to_bits(), k, x);
    aplicar(
        sim,
        reg,
        &mut PreviewDrive::default(),
        &mut TextosDosCanais::default(),
    )
}

fn duracoes(sim: &SimWorld, e: Entity) -> Vec<(String, u64)> {
    let t = sim.world().get::<Timers>(e).expect("timers");
    t.0.iter()
        .map(|t| (t.name.clone(), t.duration_us))
        .collect()
}

/// ⭐⭐⭐ **A curva conduz a duração de um timer** — e o eco publica o que ficou.
#[test]
fn a_curva_conduz_a_duracao_de_um_timer() {
    let reg = registo();
    let (mut sim, e, ids) = cena();
    assert!(curva(&mut sim, &reg, e, duracao(ids[1]), 5_000_000.0));
    assert_eq!(duracoes(&sim, e)[1], ("b".into(), 5_000_000));
    assert_eq!(duracoes(&sim, e)[0].1, 1_000_000, "o vizinho não mexe");
    publicar(
        &mut sim,
        &reg,
        &[e.to_bits()],
        &mut TextosDosCanais::default(),
    );
    let lidos = sim.world().resource::<CanaisLidos>();
    assert_eq!(lidos.valor(e.to_bits(), duracao(ids[1])), Some(5_000_000.0));
}

/// ⭐⭐⭐ **Apagar um item do meio não muda a chave do vizinho**: a chave de `c` continua a
/// conduzir `c`, e a de `b` fica órfã (não escreve ninguém).
#[test]
fn apagar_um_item_do_meio_nao_muda_a_chave_do_vizinho() {
    let reg = registo();
    let (mut sim, e, ids) = cena();
    sim.world_mut()
        .get_mut::<Timers>(e)
        .expect("timers")
        .0
        .remove(1);
    assert!(curva(&mut sim, &reg, e, duracao(ids[2]), 9_000_000.0));
    assert_eq!(
        duracoes(&sim, e),
        vec![("a".into(), 1_000_000), ("c".into(), 9_000_000)]
    );
    assert!(
        !curva(&mut sim, &reg, e, duracao(ids[1]), 7_000_000.0),
        "órfã"
    );
    assert_eq!(duracoes(&sim, e)[1].1, 9_000_000);
}

/// ⭐⭐ **Reordenar os itens não muda as chaves**: cada chave segue o seu item.
#[test]
fn reordenar_os_itens_nao_muda_as_chaves() {
    let reg = registo();
    let (mut sim, e, ids) = cena();
    sim.world_mut()
        .get_mut::<Timers>(e)
        .expect("timers")
        .0
        .reverse();
    assert!(curva(&mut sim, &reg, e, duracao(ids[0]), 4_000_000.0));
    assert_eq!(
        duracoes(&sim, e)[2],
        ("a".into(), 4_000_000),
        "`a` está no fim agora"
    );
    let mut t = TextosDosCanais::default();
    let pub_ = campos(&reg, sim.world(), e, &mut t);
    assert!(pub_.contains(&(duracao(ids[2]), 3_000_000.0)), "{pub_:?}");
}

/// Um item ainda sem id (acabado de acrescentar) não tem canal até o passe lho dar.
#[test]
fn um_item_sem_id_nao_publica() {
    let reg = registo();
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn(Timers(vec![timer("a", 1)])).id();
    let mut t = TextosDosCanais::default();
    assert!(campos(&reg, sim.world(), e, &mut t).is_empty());
}

/// ⭐ **Toda lista do catálogo publica os campos dos itens** — um objecto com um item de cada.
#[test]
fn toda_lista_do_catalogo_publica_os_campos_dos_itens() {
    use ph2d_ecs::state_machine::{MachineState, Transition};
    let reg = registo();
    let mut sim = SimWorld::new();
    let mut anc = ph2d_ecs::NamedAnchorList::default();
    anc.insert(ph2d_ecs::NamedAnchor::socket("boca"))
        .expect("cabe");
    let e = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Timers(vec![timer("a", 1)]),
            ph2d_ecs::Tweens(vec![ph2d_tween::Tween::default()]),
            ph2d_ecs::StateMachine {
                states: vec![MachineState::default()],
                transitions: vec![Transition::default()],
                initial: 0,
            },
            ph2d_ecs::CounterWatch(vec![ph2d_ecs::CounterWatchRow::default()]),
            ph2d_ecs::SignalActions(vec![ph2d_ecs::SignalAction::default()]),
            ph2d_ecs::SignalOnAction(vec![ph2d_ecs::ActionTriggerRow::default()]),
            ph2d_ecs::ShakeEmitter(vec![ph2d_ecs::ShakeSource::default()]),
            anc,
        ))
        .id();
    sim.world_mut()
        .entity_mut(e)
        .insert(ph2d_physics_ecs::Health {
            resistances: vec![ph2d_physics_ecs::Resistance::default()],
            ..ph2d_physics_ecs::Health::default()
        });
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let mut t = TextosDosCanais::default();
    let pub_ = campos(&reg, sim.world(), e, &mut t);
    for l in lista::todas() {
        for f in l.campos {
            // A caixa e o miolo da âncora só publicam quando a âncora os tem.
            if l.codigo == codigo::ANCORAS && f.field_id >= 4 {
                continue;
            }
            assert!(
                pub_.iter().any(|(k, _)| cl::resolver(*k)
                    .is_some_and(|(m, g, _)| m.codigo == l.codigo && g.field_id == f.field_id)),
                "{} #{} não publicou",
                l.componente,
                f.field_id
            );
        }
    }
}
