//! **O ENVELOPE que o artista permitiu, e só ele** (W-Pulley W0).
//!
//! Irmão de `physics_overlay_joints_tests`, separado dele no cap de 600 LOC — e
//! o corte é por assunto: lá *o que este joint É* (a figura do tipo, a
//! deformação, de quem é cada ponta, a barriga da corda), aqui *o que o artista
//! AUTOROU sobre ele* — o alcance, o comprimento, o motor. São as marcas que a
//! §12 desenha a partir de NÚMEROS, e por isso as que erram quando um tipo novo
//! chega e ninguém pergunta a porta dele.

use super::super::physics_overlay_joint_glyphs::LIMIT_ARC_PX;
use super::joint_tests::{angular_spread, marks, points_of, view};
use super::{JOINT_DIM_RGBA, JOINT_RGBA};
use ph2d_physics_ecs::{JointKind, JointView};

/// **Um limite autorado é um limite DESENHADO** — e um pino livre não inventa
/// paredes.
#[test]
fn a_limited_hinge_draws_its_arc_and_a_free_one_does_not() {
    let mut limited = view(JointKind::Pin);
    limited.limits = Some([-0.7, 0.7]);
    let free = view(JointKind::Pin);

    // ⚠️ O oráculo é ANGULAR, não de extensão: a caixa da banda apagada é
    // dominada pelas linhas de posse (2 m = 200 px), e o arco de 21 px cabe
    // inteiro dentro dela — a 1ª versão deste gate mediu 200,0 px nos dois
    // casos e não podia falhar. Um arco é a única coisa que COBRE ângulo à
    // distância fixa da âncora.
    let band = (LIMIT_ARC_PX - 3.0, LIMIT_ARC_PX + 3.0);
    let with = angular_spread(&marks(&limited), JOINT_DIM_RGBA, limited.anchor_a, band);
    let without = angular_spread(&marks(&free), JOINT_DIM_RGBA, free.anchor_a, band);
    assert!(
        with > 1.0,
        "o alcance autorado cobriu {with:.2} rad a {LIMIT_ARC_PX} px da âncora \
         — o limite segue sendo número cego no §12"
    );
    assert!(
        without < 0.2,
        "um pino LIVRE desenhou {without:.2} rad de arco: paredes que ninguém \
         autorou, que é pior que nenhuma parede"
    );
}

/// **Um motor autorado gira na tela** — e o glifo é o MESMO da zona de torque,
/// porque a pergunta é a mesma (*para que lado?*); a cor é que diz de quem.
#[test]
fn a_motor_is_visible_and_its_direction_is_the_sign() {
    let mut cw = view(JointKind::Pin);
    cw.motor_speed = Some(-2.0);
    let mut ccw = view(JointKind::Pin);
    ccw.motor_speed = Some(2.0);
    let passive = view(JointKind::Pin);

    let n = |v: &JointView| points_of(&marks(v), JOINT_RGBA).len();
    assert!(
        n(&cw) > n(&passive) && n(&ccw) > n(&passive),
        "um pino motorizado desenhou o mesmo que um passivo: o motor é \
         invisível na cena"
    );
    assert_ne!(
        points_of(&marks(&cw), JOINT_RGBA),
        points_of(&marks(&ccw), JOINT_RGBA),
        "motor horário e anti-horário desenham igual — o SINAL é a informação \
         inteira do glifo"
    );
}

/// **Cada TIPO desenha só as anotações que ele usa** — e a pergunta é feita a
/// uma porta em `JointKind`, nunca a `Option::is_some`.
///
/// ## Por que este gate existe
///
/// O anel de comprimento era `if let Some(len) = v.length`, e a POLIA carrega um
/// `length` que **não é um raio**: é a corda inteira, a soma de dois ramos que
/// passam por roldanas em outro lugar. Pintado como raio, virou um círculo
/// gigante em volta da âncora — o que o artista fotografou e reportou.
///
/// ⚠️ **A porta certa já existia** (`length_is_a_radius`, criada exatamente para
/// isto) e estava aplicada a UM dos dois consumidores: o handle arrastável do
/// `point_gizmo`. Esta era a segunda cópia da mesma pergunta, e **duas cópias
/// divergem** — é a irmã exata do bug do trilho de 26/07.
///
/// ## O oráculo é uma DIFERENÇA, não um formato
///
/// Identificar "qual destes paths é o anel" pela geometria seria frágil e
/// circular. Em vez disso o gate **liga e desliga uma entrada** e conta: se
/// oferecer o `length` acrescenta um path, o anel foi desenhado. A mesma forma
/// serve o arco de limite, que responde a outra porta (`limits_in_metres`).
#[test]
fn each_kind_draws_only_the_annotations_it_uses() {
    for kind in JointKind::ALL {
        let bare = JointView {
            limits: None,
            length: None,
            ..view(kind)
        };
        let base = marks(&bare).len();

        let ringed = marks(&JointView {
            length: Some(1.5),
            ..bare
        })
        .len();
        assert_eq!(
            ringed > base,
            kind.length_is_a_radius(),
            "{kind:?}: o anel de comprimento é desenhado quando e só quando o \
             número nomeia um RAIO a partir da âncora — numa polia ele é a corda \
             inteira, e o círculo descreveria uma distância que não existe"
        );

        let arced = marks(&JointView {
            limits: Some([-0.5, 0.5]),
            ..bare
        })
        .len();
        assert_eq!(
            arced > base,
            kind.has_limits() && !kind.limits_in_metres(),
            "{kind:?}: o arco é o envelope de um alcance ANGULAR; um curso em \
             metros é desenhado pelos tiques do trilho"
        );
    }
}
