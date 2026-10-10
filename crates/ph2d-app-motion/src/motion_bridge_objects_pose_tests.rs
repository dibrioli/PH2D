//! Os gates da **POSE de um objeto** (doc 89 folha 14) — irmão do
//! `motion_bridge_objects_tests` pelo teto de LOC do HR-18, e cortado por ASSUNTO: eles
//! respondem *"o que o grafo recebe quando nomeia X"*, e não *"como a membrana anda"*.

use super::*;

/// **A POSE DO OBJETO CARREGA A ROTAÇÃO E A ESCALA — não a identidade.**
///
/// ⚠️ **Este gate nasceu de uma MUTAÇÃO SOBREVIVENTE** (doc 89 folha 14): trocar
/// `t.rotation` por `0.0` no publicador não punha nenhum teste vermelho. O `Transform`
/// estava na query desde sempre e só a translação saía — a rotação e a escala eram
/// **descartadas**, e o `source.object` não tinha como herdar a orientação do objeto que
/// nomeia.
///
/// ⚠️ **E os TRÊS canais têm de continuar separados** (ver `external::position_of`): a
/// aparência mora na origem sem pose, a posição tem o canal dela, a pose tem o seu. Os
/// nomes são afirmados aqui porque uma "simplificação" que os fundisse passaria — é
/// assim que o `motion.look_at` shipou partido.
#[test]
fn the_objects_pose_carries_its_rotation_and_scale() {
    use ph2d_nodegraph::attr::Column;

    let t = ph2d_ecs::Transform {
        translation: ph2d_core::Vec2::new(4.0, -1.0),
        rotation: 0.75,
        scale: ph2d_core::Vec2::new(2.0, 0.5),
        ..ph2d_ecs::Transform::IDENTITY
    };
    let pose = pose_stream(&t);
    match pose.get("rotation") {
        Some(Column::Scalar(v)) => assert_eq!(v[0], 0.75, "a rotação, não zero"),
        _ => panic!("falta a rotação"),
    }
    match pose.get("size") {
        Some(Column::Vec2(v)) => assert_eq!(v[0], [2.0, 0.5], "a escala, não a identidade"),
        _ => panic!("falta a escala"),
    }
    assert!(
        pose.get("P").is_none(),
        "a POSE não carrega posição — essa tem canal próprio, e juntá-las é como o \
         `motion.look_at` shipou partido"
    );
    // Os três nomes de canal são distintos, e continuam a sê-lo.
    let (a, b, c) = (
        ph2d_nodegraph::external::appearance_of("Rock", 0.0),
        ph2d_nodegraph::external::position_of("Rock"),
        ph2d_nodegraph::external::pose_of("Rock"),
    );
    assert_ne!(a, b);
    assert_ne!(b, c);
    assert_ne!(a, c);
}

/// ⭐ **O canal diz onde a coisa está no MUNDO** (onda 4, item 6): um filho de um pai deslocado e
/// rodado publica a pose composta, não a local — é dela que a conversão para o espaço do objecto
/// Motion parte. (Mutação: `pose_de_mundo` devolver o `Transform` local ⇒ RED.)
#[test]
fn a_pose_publicada_de_um_filho_e_a_de_mundo() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let mut pai = Transform::from_translation(ph2d_core::Vec2::new(10.0, 0.0));
    pai.rotation = std::f32::consts::FRAC_PI_2;
    let p = w.spawn(pai).id();
    let f = w
        .spawn((
            Transform::from_translation(ph2d_core::Vec2::new(2.0, 0.0)),
            ChildOf(p),
        ))
        .id();
    let t = pose_de_mundo(sim.world(), f).expect("tem pose");
    assert!((t.translation.x - 10.0).abs() < 1e-4 && (t.translation.y - 2.0).abs() < 1e-4);
    assert!((t.rotation - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
}
