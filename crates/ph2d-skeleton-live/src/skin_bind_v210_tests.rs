use ph2d_ecs::{Entity, SimWorld};
use ph2d_skeleton_ecs::{Bone, CorreccaoDePeso, SKIN_TYPE_NAME, SkinBind, SkinLaw};
use ph2d_vec_scene::{VecPathId, VecScene};
use serde::Serialize;

use crate::barra_da_cena_tests_support::barra_da_cena_com;

/// O tendão como a v210 o gravava, campo a campo — escrito aqui, nem do tipo vivo nem do congelado.
#[derive(Serialize)]
struct TendaoGravado {
    bone: u64,
    rest: [f64; 6],
}

#[derive(Serialize)]
struct PeleGravada<'a> {
    source: &'a [u8],
    tendons: Vec<TendaoGravado>,
    law: SkinLaw,
    correcoes: &'a [CorreccaoDePeso],
}

fn bytes_v210(s: &SkinBind) -> Vec<u8> {
    postcard::to_allocvec(&PeleGravada {
        source: &s.source,
        tendons: (s.tendons.iter())
            .map(|t| TendaoGravado {
                bone: t.bone.0,
                rest: t.rest,
            })
            .collect(),
        law: s.law,
        correcoes: &s.correcoes,
    })
    .expect("codifica")
}

fn verts(sim: &SimWorld, scene: &mut VecScene, id: VecPathId) -> Vec<[f64; 2]> {
    crate::skin_live::recook(sim, scene);
    let p = scene.paths().iter().find(|p| p.id == id).expect("path");
    p.verts.iter().map(|v| v.anchor).collect()
}

fn pior(a: &[[f64; 2]], b: &[[f64; 2]]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(p, q)| (p[0] - q[0]).abs().max((p[1] - q[1]).abs()))
        .fold(0.0, f64::max)
}

/// ⭐ **Uma pele gravada na v210 abre com `comprimento = 0` e NÃO estica** quando o osso cresce — o
/// desenho é o de antes de crescer. ⛔ CONTROLO: a pele de hoje (com o comprimento do bind) estica.
#[test]
fn a_pele_gravada_sem_comprimento_abre_e_nao_estica() {
    ph2d_skeleton_ecs::register_skeleton_components(
        &mut ph2d_ecs::scene::ComponentRegistry::default(),
    );
    let (mut sim, mut scene, map, id, ossos) = barra_da_cena_com(false);
    let e = Entity::from_bits(map[&id]);
    let viva = sim.world().get::<SkinBind>(e).expect("pele").clone();
    assert!(
        viva.tendons.iter().all(|t| t.comprimento > 0.0),
        "o bind grava o comprimento"
    );
    let antiga = bytes_v210(&viva);
    // ⛔ CONTROLO: o tipo vivo NÃO lê a forma antiga (sem o degrau, o ficheiro não abria).
    assert!(postcard::from_bytes::<SkinBind>(&antiga).is_err());
    let migrada: SkinBind = postcard::from_bytes(
        &ph2d_ecs::item_id::migra_blob(SKIN_TYPE_NAME, &antiga).expect("o degrau lê a v210"),
    )
    .expect("forma viva");
    assert!(migrada.tendons.iter().all(|t| t.comprimento == 0.0));
    assert_eq!(migrada.tendons.len(), viva.tendons.len());
    assert!(
        (migrada.tendons.iter().zip(&viva.tendons))
            .all(|(a, b)| a.bone == b.bone && a.rest == b.rest)
    );
    // ⛔ E a forma de hoje não se relê como antiga.
    let hoje = postcard::to_allocvec(&viva).expect("codifica");
    assert!(ph2d_ecs::item_id::migra_blob(SKIN_TYPE_NAME, &hoje).is_none());

    let ponta = *ossos.last().expect("osso");
    let repouso = verts(&sim, &mut scene, id);
    sim.world_mut().get_mut::<Bone>(ponta).expect("osso").length *= 1.5;
    let esticada = verts(&sim, &mut scene, id);
    assert!(
        pior(&repouso, &esticada) > 0.1,
        "controlo: a pele de hoje estica"
    );
    sim.world_mut().entity_mut(e).insert(migrada);
    let antiga_crescida = verts(&sim, &mut scene, id);
    assert!(
        pior(&repouso, &antiga_crescida) < 1e-9,
        "a pele da v210 esticou: {}",
        pior(&repouso, &antiga_crescida)
    );
}
