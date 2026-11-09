//! `ComponentRegistry::derivado_quando` (P14 §4 A): `C` não se grava na entidade com a marca `M`,
//! nem no ficheiro nem no diff do undo; noutra entidade grava-se como sempre.

use bevy_ecs::component::Component;
use ph2d_ecs::scene::incremental::{CaptureCache, capture_incremental};
use ph2d_ecs::scene::registry::{ComponentRegistry, register_ecs_components};
use ph2d_ecs::scene::save::WorldSnapshot;
use ph2d_ecs::scene::world_to_snapshot;
use ph2d_ecs::{Name, SimWorld, Transform, TransformPropagationState, WorklistBuf};
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct Clips(u32);

#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct Marca;

fn reg() -> ComponentRegistry {
    let mut r = ComponentRegistry::new();
    register_ecs_components(&mut r);
    r.register_default::<Clips>("p14::test::Clips");
    r.derivado_quando::<Clips, Marca>();
    r
}

fn clips_no(snap: &WorldSnapshot, r: &ComponentRegistry, nome: &str) -> Option<u32> {
    let id = r
        .get_by_name("p14::test::Clips")
        .expect("registado")
        .type_id;
    let name_id = r.get_by_name("ph2d::ecs::Name").expect("Name").type_id;
    let row = snap.entities.iter().find(|row| {
        row.components.iter().any(|c| {
            c.type_id == name_id && postcard::from_bytes::<Name>(&c.data).is_ok_and(|n| n.0 == nome)
        })
    })?;
    row.components
        .iter()
        .find(|c| c.type_id == id)
        .map(|c| postcard::from_bytes::<Clips>(&c.data).expect("decode").0)
}

#[test]
fn o_derivado_nao_se_grava_so_na_entidade_com_a_marca() {
    let r = reg();
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    w.spawn((Transform::IDENTITY, Name::new("Comum"), Clips(7)));
    let pixel = w
        .spawn((Transform::IDENTITY, Name::new("Pixel"), Clips(9), Marca))
        .id();

    // O ficheiro (e a foto do Play): `world_to_snapshot`.
    let mut prop = TransformPropagationState::new(sim.world_mut());
    let mut wl = WorklistBuf::default();
    let mut snap = WorldSnapshot::default();
    world_to_snapshot(sim.world_mut(), &mut prop, &mut wl, &r, &mut snap).expect("captura");
    assert_eq!(clips_no(&snap, &r, "Comum"), Some(7), "o comum grava-se");
    assert_eq!(
        clips_no(&snap, &r, "Pixel"),
        None,
        "o derivado entrou no ficheiro da entidade com a marca"
    );

    // O diff do undo: `capture_incremental`.
    let mut cache = CaptureCache::new();
    let mut out = WorldSnapshot::default();
    capture_incremental(sim.world_mut(), &mut cache, &r, &mut out).expect("1.ª");
    sim.world_mut().get_mut::<Clips>(pixel).expect("tem").0 = 10;
    let rep = capture_incremental(sim.world_mut(), &mut cache, &r, &mut out).expect("2.ª");
    assert_eq!(
        rep.reserialized, 0,
        "mudar o derivado marcou diferença na linha — seria um passo de undo que ninguém fez"
    );
    assert!(!rep.changed());

    // Controlo: o mesmo gesto na entidade SEM a marca é diferença.
    let comum = {
        let w = sim.world_mut();
        let mut q = w.query::<(bevy_ecs::entity::Entity, &Name)>();
        q.iter(w).find(|(_, n)| n.0 == "Comum").expect("comum").0
    };
    sim.world_mut().get_mut::<Clips>(comum).expect("tem").0 = 8;
    let rep = capture_incremental(sim.world_mut(), &mut cache, &r, &mut out).expect("3.ª");
    assert_eq!(rep.reserialized, 1, "o controlo: o comum tem de marcar");
}

/// Inspector, cópias, instâncias e Simular leem o `serialize`: eles têm de VER o derivado (sem isto o
/// Pixel Sprite aparecia sem clips, a paleta oferecia-os de novo e a cópia nascia sem eles).
#[test]
fn os_outros_leitores_veem_o_derivado() {
    let r = reg();
    let mut sim = SimWorld::new();
    let pixel = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Pixel"), Clips(9), Marca))
        .id();
    let entry = r.get_by_name("p14::test::Clips").expect("registado");
    let visto = (entry.serialize)(sim.world(), pixel).expect("lê");
    assert_eq!(
        visto.map(|b| postcard::from_bytes::<Clips>(&b).expect("decode").0),
        Some(9),
        "o serialize escondeu o derivado dos leitores que não gravam"
    );
    assert_eq!((entry.grava)(sim.world(), pixel).expect("lê"), None);
}

#[test]
#[should_panic(expected = "is not registered")]
fn derivado_de_um_tipo_nao_registado_e_panico_claro() {
    let mut r = ComponentRegistry::new();
    r.derivado_quando::<Clips, Marca>();
}
