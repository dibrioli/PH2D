use super::*;
use ph2d_ecs::canal::familia::OSSO;
use ph2d_ecs::canal_osso as o;
use ph2d_ecs::{CanaisEscritos, ChildOf, FieldKey, Transform};
use ph2d_preview_drive::PreviewDrive;

/// Pai de comprimento `10`; um filho-osso na PONTA e um objecto a MEIO do eixo (com `y`).
fn cena() -> (SimWorld, Entity, Entity, Entity) {
    let mut sim = SimWorld::new();
    let pai = (sim.world_mut())
        .spawn((
            Transform::IDENTITY,
            Bone {
                length: 10.0,
                ..Bone::default()
            },
        ))
        .id();
    let mut na_ponta = Transform::IDENTITY;
    na_ponta.translation.x = 10.0;
    let ponta = (sim.world_mut())
        .spawn((na_ponta, Bone::default(), ChildOf(pai)))
        .id();
    let mut a_meio = Transform::IDENTITY;
    a_meio.translation.x = 5.0;
    a_meio.translation.y = 2.0;
    let meio = sim.world_mut().spawn((a_meio, ChildOf(pai))).id();
    (sim, pai, ponta, meio)
}

fn pose(sim: &SimWorld, e: Entity) -> Transform {
    *sim.world().get::<Transform>(e).expect("transform")
}

fn assert_empurrado(sim: &SimWorld, ponta: Entity, meio: Entity, l: f32) {
    let (p, m) = (pose(sim, ponta), pose(sim, meio));
    assert!(
        (p.translation.x - l).abs() < 1e-5,
        "a ponta: {}",
        p.translation.x
    );
    assert!(
        (m.translation.x - l / 2.0).abs() < 1e-5,
        "o meio: {}",
        m.translation.x
    );
    assert!((m.translation.y - 2.0).abs() < 1e-6, "o y não é do eixo");
}

/// ⭐ **A MÃO cresce o osso e os filhos andam com a ponta**; ⛔ CONTROLO: a força não mexe neles.
#[test]
fn a_mao_no_comprimento_empurra_os_filhos() {
    let (mut sim, pai, ponta, meio) = cena();
    assert!(aplica(&mut sim, pai, BoneKnob::Strength, 3.0));
    assert_empurrado(&sim, ponta, meio, 10.0);
    assert!(aplica(&mut sim, pai, BoneKnob::Length, 16.0));
    assert_empurrado(&sim, ponta, meio, 16.0);
    assert!(aplica(&mut sim, pai, BoneKnob::Length, 4.0));
    assert_empurrado(&sim, ponta, meio, 4.0);
}

/// ⭐ **A CURVA empurra os filhos como pré-visualização**: o `substitute` lê a pose AUTORADA dos
/// filhos (e o comprimento autorado), o `restore` devolve a empurrada.
#[test]
fn a_curva_empurra_os_filhos_e_o_undo_ve_o_autorado() {
    let (mut sim, pai, ponta, meio) = cena();
    let mut drive = PreviewDrive::default();
    (sim.world_mut())
        .get_resource_or_insert_with(CanaisEscritos::default)
        .escrever(
            pai.to_bits(),
            FieldKey::new(OSSO, o::COMPRIMENTO, 0, 0),
            20.0,
        );
    assert!(crate::canais_osso::aplicar(&mut sim, &mut drive));
    assert_empurrado(&sim, ponta, meio, 20.0);
    assert!(drive.drives(ponta.to_bits()) && drive.drives(meio.to_bits()));
    let vivo = drive.substitute(&mut sim);
    assert_empurrado(&sim, ponta, meio, 10.0);
    let l = sim.world().get::<Bone>(pai).expect("osso").length;
    assert!((l - 10.0).abs() < 1e-9, "o comprimento autorado: {l}");
    PreviewDrive::restore(&mut sim, &vivo);
    assert_empurrado(&sim, ponta, meio, 20.0);
}

/// ⭐ Smoke do dono 30/10 (`PH2D_VEC_BONE_SMOKE=8`): arrastar Length até ao 0 e voltar devolve os
/// filhos — em 0 o osso não tem eixo de onde medir e o empurrão de volta recusava-se (`l0 <= 0`),
/// os filhos ficavam colapsados na raiz para sempre.
#[test]
fn arrastar_o_comprimento_ate_zero_e_voltar_devolve_os_filhos() {
    let (mut sim, pai, ponta, meio) = cena();
    let (p0, m0) = (pose(&sim, ponta), pose(&sim, meio));
    for v in [6.0, 0.0, -3.0, 0.0, 10.0] {
        assert!(aplica(&mut sim, pai, BoneKnob::Length, v));
    }
    let (p, m) = (pose(&sim, ponta), pose(&sim, meio));
    assert!(
        (p.translation.x - p0.translation.x).abs() < 1e-4,
        "a ponta: {}",
        p.translation.x
    );
    assert!(
        (m.translation.x - m0.translation.x).abs() < 1e-4,
        "o meio: {}",
        m.translation.x
    );
    assert_eq!(m.translation.y, m0.translation.y);
}
