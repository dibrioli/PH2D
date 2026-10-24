use crate::{BlobRef, PreviewDrive};
use ph2d_ecs::scene::{
    ComponentRegistry, WorldSnapshot, register_ecs_components, world_to_snapshot,
};
use ph2d_ecs::{SimWorld, Transform, TransformPropagationState, Visibility, WorklistBuf};

fn captura(sim: &mut SimWorld, reg: &ComponentRegistry) -> WorldSnapshot {
    let mut prop = TransformPropagationState::new(sim.world_mut());
    let mut out = WorldSnapshot::new();
    world_to_snapshot(
        sim.world_mut(),
        &mut prop,
        &mut WorklistBuf::default(),
        reg,
        &mut out,
    )
    .expect("captura");
    out
}

/// A reprodução põe um componente que o documento não tem e muda outro: a captura vê o documento, o
/// `restore` devolve o vivo, e largar devolve o autorado.
#[test]
fn um_blob_conduzido_nao_entra_na_captura() {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn(Transform::default()).id();
    let doc = captura(&mut sim, &reg);

    let vis = BlobRef::of(reg.get_by_name("ph2d::ecs::Visibility").expect("registado"));
    let tr = BlobRef::of(reg.get_by_name("ph2d::ecs::Transform").expect("registado"));
    let escondido = postcard::to_allocvec(&Visibility::hidden()).expect("postcard");
    let mut longe = Transform::default();
    longe.translation.x = 9.0;
    let longe_b = postcard::to_allocvec(&longe).expect("postcard");

    let mut drive = PreviewDrive::default();
    drive.blobs.ronda();
    assert!(drive.blobs.conduz(&mut sim, e, vis, Some(&escondido)));
    assert!(drive.blobs.conduz(&mut sim, e, tr, Some(&longe_b)));
    assert_eq!(drive.blobs.len(), 2);

    let vivo = drive.substitute(&mut sim);
    assert_eq!(
        captura(&mut sim, &reg),
        doc,
        "a captura a meio da reprodução é o documento"
    );
    PreviewDrive::restore(&mut sim, &vivo);
    assert_eq!(
        sim.world().get::<Visibility>(e),
        Some(&Visibility::hidden())
    );
    assert_eq!(
        sim.world().get::<Transform>(e).map(|t| t.translation.x),
        Some(9.0)
    );

    // Outro quadro, nada muda: a condução mantém-se (a `settle` não a promove).
    drive.settle();
    drive.blobs.ronda();
    assert!(drive.blobs.conduz(&mut sim, e, vis, Some(&escondido)));
    assert!(drive.blobs.conduz(&mut sim, e, tr, Some(&longe_b)));
    drive.settle();
    assert_eq!(drive.blobs.len(), 2);

    // A macro parou: o que ela mostrou volta ao documento.
    drive.blobs.ronda();
    assert_eq!(drive.blobs.devolve_os_largados(&mut sim), 2);
    assert_eq!(captura(&mut sim, &reg), doc);
    assert!(drive.blobs.is_empty());
}
