//! **As rows do `Custom`** — três eixos, e o eixo em que o motor age.
//!
//! Módulo irmão do `joint.rs` (que estava em 561 do cap de 600), cortado pelo
//! assunto que a §12 já desenha: *o que este TIPO tem a afinar* × *os graus de
//! liberdade que este joint DESCREVE*.
//!
//! ⚠️ **Um eixo tem TRÊS estados** (*Free / Limited / Locked*, o modelo do
//! Unreal), e o par Min/Max só é pintado no do meio — a lei do knob-morto que
//! esta seção segue desde o W3: um batente num eixo travado é um número que o
//! solver não lê, e um controle que parece funcionar é pior que um que falta.

use super::rows::{num_row, seg_row};
use super::*;
use ph2d_editor_core::screens::hero::InspectorJointInfo;

/// Os três eixos, na ordem em que o array os guarda e a tela os lista.
///
/// ⚠️ **A UNIDADE é do EIXO, não do tipo** — e é a diferença que o `Custom`
/// introduziu: nos sete presets o grau de liberdade livre é uma propriedade do
/// tipo, aqui ele é escolhido.
const AXIS_ROWS: [(&str, &str); 3] = [("X", "m"), ("Y", "m"), ("Rotation", "deg")];

/// Free / Limited / Locked.
const AXIS_MODE_LABELS: [&str; 3] = ["Free", "Limited", "Locked"];

/// X / Y / Rotation — para o seletor de eixo do motor.
const AXIS_LABELS: [&str; 3] = ["X", "Y", "Rotation"];

/// Pinta os três eixos e devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_axis_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorJointInfo,
) -> f32 {
    let mut yy = y;
    for (i, (name, unit)) in AXIS_ROWS.into_iter().enumerate() {
        yy = seg_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            name,
            ids::INSP_JOINT_AXIS_GROUP[i],
            &ids::INSP_JOINT_AXIS_MODE[i],
            &AXIS_MODE_LABELS,
            info.axis_mode_tag[i],
        );
        // ⚠️ **O par só existe no modo `Limited`.** Ele é o único em que o
        // solver lê os dois números (no rapier `locked_axes` e `limits` são
        // coisas separadas), e pintá-lo nos outros ofereceria ao artista um
        // knob que não pode fazer nada.
        if info.axis_mode_tag[i] == MODE_LIMITED {
            for (label, id) in [
                (format!("  Min ({unit})"), ids::INSP_JOINT_AXIS_MIN[i]),
                (format!("  Max ({unit})"), ids::INSP_JOINT_AXIS_MAX[i]),
            ] {
                yy = num_row(
                    scene,
                    text_system,
                    theme,
                    hit_index,
                    store,
                    x,
                    w,
                    yy,
                    &label,
                    id,
                );
            }
        }
    }
    yy
}

/// **Em qual eixo o motor age** — a row que num Custom decide a UNIDADE do alvo.
///
/// Pintada só com o motor ARMADO: sem ele é a escolha de um eixo para um motor
/// que não existe.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_motor_axis_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorJointInfo,
) -> f32 {
    seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        "Motor Axis",
        ids::INSP_JOINT_MOTOR_AXIS_GROUP,
        &ids::INSP_JOINT_MOTOR_AXIS,
        &AXIS_LABELS,
        info.motor_axis_tag,
    )
}

/// A tag do modo `Limited` — o único em que o par Min/Max é lido.
///
/// ⚠️ Literal e não uma leitura de `ph2d_physics_ecs::AxisMode`: o painel é
/// loose-coupled com o motor, como o `KIND_LABELS` ao lado. O gate
/// `the_panel_and_the_component_agree_on_the_axis_mode_tags` compara os dois.
pub(crate) const MODE_LIMITED: u8 = 1;

#[cfg(test)]
mod tests {
    use super::super::joint::{AXIS_ROTATION, KIND_CUSTOM, motor_units};
    use ph2d_editor_core::screens::hero::InspectorJointInfo;

    /// ⚠️ **Construída à mão e não por `..Default::default()`**: o
    /// `InspectorJointInfo` não implementa `Default`, e dar-lhe um seria
    /// oferecer um snapshot de joint que ninguém autorou — a shell o constrói
    /// sempre a partir de um componente vivo.
    fn info(kind_tag: u8, motor_axis_tag: u8) -> InspectorJointInfo {
        InspectorJointInfo {
            entity_bits: 1,
            kind_tag,
            body_a_name: String::new(),
            body_b_name: String::new(),
            bound: true,
            world_anchored: false,
            limits_enabled: false,
            limit_min_ui: 0.0,
            limit_max_ui: 0.0,
            motor_enabled: true,
            motor_mode_tag: 0,
            motor_speed_ui: 0.0,
            motor_target_ui: 0.0,
            motor_max_force: 0.0,
            rest_length: 1.0,
            stiffness: 1.0,
            damping: 0.0,
            max_length: 1.0,
            soft: false,
            pick_armed: 0,
            wheel_count: 0,
            break_enabled: false,
            break_force: 0.0,
            break_torque: 0.0,
            breaks_on_torque: false,
            active: true,
            collide_connected: false,
            paste_targets: 0,
            axis_mode_tag: [0, 0, 0],
            axis_min_ui: [0.0; 3],
            axis_max_ui: [0.0; 3],
            motor_axis_tag,
        }
    }

    /// **O rótulo do motor de um Custom segue o EIXO**, não o tipo.
    ///
    /// ⚠️ É a metade VISÍVEL de um defeito mudo: o número é convertido na
    /// fronteira do painel, então um rótulo em graus sobre um alvo que o solver
    /// lê em metros faz o artista digitar 90 e a peça andar 1,57 m.
    ///
    /// ⚠️ **Esta é a SEGUNDA afirmação da mesma lei** — a primeira é
    /// `PhysicsJoint::motor_in_metres`, gateada em
    /// `ph2d-physics-ecs/tests/joint_custom_unit.rs`. O painel é loose-coupled
    /// com o motor (a convenção de toda seção irmã), então ele não pode
    /// PERGUNTAR; o que o mantém honesto é as duas terem gate, e este arquivo
    /// nomear a outra.
    #[test]
    fn a_customs_motor_rows_are_labelled_by_its_axis() {
        assert_eq!(motor_units(&info(KIND_CUSTOM, 0)), ("m/s", "m"));
        assert_eq!(motor_units(&info(KIND_CUSTOM, 1)), ("m/s", "m"));
        assert_eq!(
            motor_units(&info(KIND_CUSTOM, AXIS_ROTATION)),
            ("\u{00b0}/s", "\u{00b0}")
        );
    }

    /// **E os presets não mudaram** — o eixo do motor é ignorado fora de um
    /// Custom, senão um Pin com o tag herdado de um Custom mudaria de unidade.
    #[test]
    fn a_preset_ignores_the_motor_axis_tag() {
        for tag in 0..8u8 {
            let a = motor_units(&info(tag, 0));
            let b = motor_units(&info(tag, AXIS_ROTATION));
            assert_eq!(a, b, "o tipo {tag} não pode ler o eixo do motor");
        }
    }
}
