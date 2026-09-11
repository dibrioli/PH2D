//! **A UNIDADE do motor de um `Custom` é do EIXO, não do tipo** (W-JointCustom).
//!
//! Nos sete presets o grau de liberdade livre É uma propriedade do tipo: um Pin
//! gira, um Slider desliza, e `JointKind::motor_in_metres` responde sem saber
//! nada da instância. Um Custom **escolhe** o eixo, então metro-ou-radiano é
//! escolhido junto — e a porta do tipo passa a ser um default, não a resposta.
//!
//! ⚠️ **O modo de falha é MUDO e caro:** o alvo de um servo é convertido na
//! fronteira do painel (graus ↔ radianos). Rotulado em graus um número que o
//! solver lê em metros, o artista digita `90` e a peça anda **1,57 m** — nada
//! erra, nada avisa, e o número na tela concorda consigo mesmo.

use ph2d_physics_ecs::{AxisMode, CustomAxis, JointKind, PhysicsJoint};

fn custom(motor_axis: CustomAxis) -> PhysicsJoint {
    let mut j = PhysicsJoint {
        kind: JointKind::Custom,
        ..PhysicsJoint::default()
    };
    j.custom.motor_axis = motor_axis;
    j
}

/// **A porta da INSTÂNCIA segue o eixo; a do TIPO não pode.**
///
/// Mutação que sangra: fazer `PhysicsJoint::motor_in_metres` delegar sempre ao
/// tipo — os dois eixos lineares passam a responder `false` (radianos) sobre um
/// número que o solver lê em metros.
#[test]
fn the_motor_unit_of_a_custom_follows_its_authored_axis() {
    assert!(custom(CustomAxis::X).motor_in_metres());
    assert!(custom(CustomAxis::Y).motor_in_metres());
    assert!(!custom(CustomAxis::Rotation).motor_in_metres());
    // ⚠️ **O CONTROLE, e ele é a metade que dá sentido ao gate:** a porta do
    // TIPO responde a mesma coisa para os três, porque ela não pode saber. É por
    // isso que ela deixou de ser a que a UI pergunta.
    assert!(!JointKind::Custom.motor_in_metres());
}

/// **E os sete presets não mudaram** — a porta da instância delega a eles.
///
/// Sem isto a wave poderia ter trocado a resposta de um Slider ou de uma Rope
/// sem que nada acusasse: os dois são lineares, e um erro que os mantivesse
/// lineares por outro caminho ficaria invisível.
#[test]
fn every_preset_still_answers_through_its_kind() {
    for k in [
        JointKind::Pin,
        JointKind::Spring,
        JointKind::Rope,
        JointKind::Weld,
        JointKind::Slider,
        JointKind::Rod,
        JointKind::Wheel,
        JointKind::Pulley,
    ] {
        let j = PhysicsJoint {
            kind: k,
            ..PhysicsJoint::default()
        };
        assert_eq!(
            j.motor_in_metres(),
            k.motor_in_metres(),
            "{k:?}: a porta da instância tem de delegar ao tipo"
        );
    }
}

/// **Um limiar de TORQUE só é oferecido a um Custom cujo eixo angular é
/// restringido** — a mesma lei que a solda mole trouxe, feita ao EIXO.
///
/// rapier não publica reação nenhuma de um eixo que não restringe, então a caixa
/// de Break com um eixo de rotação LIVRE mostraria um número que nunca dispara.
#[test]
fn a_torque_threshold_is_only_offered_when_the_rotation_axis_is_constrained() {
    let mut free = custom(CustomAxis::X);
    free.custom.axis_mut(CustomAxis::Rotation).mode = AxisMode::Free;
    assert!(!free.breaks_on_torque());

    for m in [AxisMode::Limited, AxisMode::Locked] {
        let mut held = custom(CustomAxis::X);
        held.custom.axis_mut(CustomAxis::Rotation).mode = m;
        assert!(
            held.breaks_on_torque(),
            "{m:?} no eixo angular restringe, logo publica reação"
        );
    }
}

/// **Um Custom nasce SOLTO** — todo eixo livre, o motor no linear X.
///
/// ⚠️ Um default que travasse tudo nasceria indistinguível de uma solda, e o
/// artista escolheria o tipo sem ver diferença nenhuma. É um default de PRODUTO,
/// e por isso tem gate próprio: ele não é mencionado por nenhum dos outros, que
/// é o que torna um teste sobre um default honesto.
#[test]
fn a_fresh_custom_is_the_loosest_joint_there_is() {
    let j = PhysicsJoint {
        kind: JointKind::Custom,
        ..PhysicsJoint::default()
    };
    for a in CustomAxis::ALL {
        assert_eq!(j.custom.axis(a).mode, AxisMode::Free, "{a:?}");
    }
    assert_eq!(j.custom.motor_axis, CustomAxis::X);
}

/// **A configuração viaja no copy/paste de propriedades** (W-JointCopy).
///
/// O `with_properties_of` desestrutura EXAUSTIVAMENTE, então o campo novo não
/// compilava até ser classificado — e a classificação certa é *o que a
/// restrição faz*, que é a metade que viaja.
#[test]
fn the_axis_configuration_travels_with_a_paste() {
    let mut source = custom(CustomAxis::Rotation);
    source.custom.axis_mut(CustomAxis::Y).mode = AxisMode::Locked;
    let target = PhysicsJoint {
        kind: JointKind::Pin,
        body_a: 7,
        body_b: 9,
        ..PhysicsJoint::default()
    };
    let pasted = target.with_properties_of(&source);
    assert_eq!(pasted.custom, source.custom);
    // E a IDENTIDADE fica com o alvo — a linha que o módulo de propriedades
    // desenha, e que esta wave não pode ter movido.
    assert_eq!((pasted.body_a, pasted.body_b), (7, 9));
}
