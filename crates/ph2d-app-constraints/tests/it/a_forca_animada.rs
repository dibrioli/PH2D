//! ⭐ **A força animada** — a curva escreve a força do item, o Auto Key lê-a sem chave fantasma, e
//! gravar a meio do play grava a força AUTORADA.

use ph2d_app_constraints::canais;
use ph2d_constraints::{Canais, CopyTransform, Kind, Space};
use ph2d_constraints_ecs::Constraints;
use ph2d_ecs::canal_constraint::{FORCA, LIGADA, chave};
use ph2d_ecs::{CanaisEscritos, CanaisLidos, SimWorld, StableId, Transform};
use ph2d_preview_drive::{FieldDrive, PreviewDrive};

fn mundo() -> (SimWorld, bevy_ecs::entity::Entity, u32, u32) {
    let mut sim = SimWorld::new();
    let mut alvo = Transform::IDENTITY;
    alvo.translation.x = 10.0;
    sim.world_mut().spawn((StableId(1), alvo));
    let mut c = Constraints::default();
    let copia = Kind::CopyTransform(CopyTransform {
        canais: Canais::LOCATION,
        space: Space::World,
        ..CopyTransform::default()
    });
    // Dois itens: o segundo é o conduzido (o id não é a posição).
    let primeiro = c.push(copia, Some(StableId(1)));
    let segundo = c.push(copia, Some(StableId(1)));
    let e = sim
        .world_mut()
        .spawn((StableId(2), Transform::IDENTITY, c))
        .id();
    (sim, e, primeiro, segundo)
}

fn escrever(sim: &mut SimWorld, e: u64, k: ph2d_ecs::FieldKey, v: f32) {
    sim.world_mut()
        .get_resource_or_insert_with(CanaisEscritos::default)
        .escrever(e, k, v);
}

#[test]
fn the_curve_drives_the_influence_and_a_save_keeps_the_authored_one() {
    let (mut sim, e, primeiro, segundo) = mundo();
    let b = e.to_bits();
    let mut drive = PreviewDrive::default();
    escrever(&mut sim, b, chave(FORCA, segundo), 0.25);
    escrever(&mut sim, b, chave(LIGADA, primeiro), 0.0);
    canais::aplicar(&mut sim, &mut drive);
    let c = sim.world().get::<Constraints>(e).unwrap().clone();
    assert_eq!(c.item(segundo).unwrap().influence, 0.25);
    assert!(!c.item(primeiro).unwrap().enabled);
    assert_eq!(
        c.item(primeiro).unwrap().influence,
        1.0,
        "a curva de um item não toca no outro"
    );

    // O Auto Key lê o valor da curva: nada a gravar.
    canais::publicar(&mut sim, &[b]);
    let lidos = sim.world().resource::<CanaisLidos>();
    assert_eq!(lidos.valor(b, chave(FORCA, segundo)), Some(0.25));
    assert_eq!(lidos.valor(b, chave(LIGADA, primeiro)), Some(0.0));

    // Gravar a meio do play: o autorado.
    let vivo = drive.fields.substitute(&mut sim);
    let gravado = sim.world().get::<Constraints>(e).unwrap().clone();
    assert_eq!(gravado.item(segundo).unwrap().influence, 1.0);
    assert!(gravado.item(primeiro).unwrap().enabled);
    FieldDrive::restore(&mut sim, &vivo);
    assert_eq!(
        sim.world()
            .get::<Constraints>(e)
            .unwrap()
            .item(segundo)
            .unwrap()
            .influence,
        0.25
    );
}

/// A meia força conduzida pela curva move a meio caminho, e a pilha avalia o que a curva deixou no
/// mesmo quadro.
#[test]
fn the_stack_reads_the_curve_in_the_same_frame() {
    let (mut sim, e, primeiro, segundo) = mundo();
    let b = e.to_bits();
    let mut drive = PreviewDrive::default();
    escrever(&mut sim, b, chave(LIGADA, primeiro), 0.0);
    escrever(&mut sim, b, chave(FORCA, segundo), 0.5);
    ph2d_app_constraints::quadro(&mut sim, &ph2d_vec_scene::VecScene::new(), &mut drive, None);
    assert_eq!(sim.world().get::<Transform>(e).unwrap().translation.x, 5.0);
}

/// ⭐ A curva de um AJUSTE (o Max do Limit Rotation, em graus) apara a pose no mesmo quadro, o Auto
/// Key lê-o, e gravar a meio do play grava o autorado.
#[test]
fn a_curve_drives_a_setting_and_a_save_keeps_the_authored_one() {
    use ph2d_constraints::Limit;
    use ph2d_ecs::canal_constraint::LIMITE_MAX;
    let mut sim = SimWorld::new();
    let mut c = Constraints::default();
    let id = c.push(
        Kind::Limit(Limit {
            rot: Some([-1.0, 1.0]),
            ..Limit::default()
        }),
        None,
    );
    let mut t = Transform::IDENTITY;
    t.rotation = 0.9;
    let e = sim.world_mut().spawn((StableId(3), t, c)).id();
    let b = e.to_bits();
    let mut drive = PreviewDrive::default();
    escrever(&mut sim, b, chave(LIMITE_MAX, id), 30.0);
    ph2d_app_constraints::quadro(&mut sim, &ph2d_vec_scene::VecScene::new(), &mut drive, None);
    let rot = sim.world().get::<Transform>(e).unwrap().rotation;
    assert!(
        (f64::from(rot) - 30f64.to_radians()).abs() < 1e-6,
        "a curva não aparou: {rot}"
    );
    canais::publicar(&mut sim, &[b]);
    let lido = sim
        .world()
        .resource::<CanaisLidos>()
        .valor(b, chave(LIMITE_MAX, id))
        .unwrap();
    assert!((lido - 30.0).abs() < 1e-4, "o Auto Key leu {lido}");
    let vivo = drive.fields.substitute(&mut sim);
    let Kind::Limit(l) = sim
        .world()
        .get::<Constraints>(e)
        .unwrap()
        .item(id)
        .unwrap()
        .kind
    else {
        panic!()
    };
    assert_eq!(l.rot, Some([-1.0, 1.0]), "gravou a curva");
    FieldDrive::restore(&mut sim, &vivo);
}
