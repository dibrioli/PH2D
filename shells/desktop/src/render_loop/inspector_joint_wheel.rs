//! **A metade RODA da §12** (W-Pulley W1) — o que a seção da corda faz com as
//! roldanas dela.
//!
//! Irmão de [`super::inspector_joint`], e o corte é o assunto: lá mora *o que um
//! JOINT é* (os campos, o `clamped`, o gesto de criar); aqui, *o que a CORDA faz
//! com a lista de rodas dela*. Nasceu do cap de 600 LOC quando a roldana virou
//! entidade, e a próxima wave da polia (motor por roda, ruptura no centro) chega
//! aqui.

use super::inspector_ordering::queue_set;
use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_editor::{InspectorWheelInfo, WheelFieldEdit};
use ph2d_physics_ecs::PhysicsBridge;

/// O nome de tipo do componente, como o registry o conhece.
const WHEEL: &str = "ph2d::physics::PulleyWheel";

/// Quantas roldanas apontam para esta corda, no mundo AUTORADO.
pub(crate) fn rope_wheel_count(sim: &mut SimWorld, joint: Entity) -> u32 {
    let Some(rope) = sim
        .world()
        .get::<Name>(joint)
        .map(|n| stable_name_id(n.as_str()))
    else {
        return 0;
    };
    let mut q = sim.world_mut().query::<&ph2d_physics_ecs::PulleyWheel>();
    u32::try_from(q.iter(sim.world()).filter(|w| w.rope == rope).count()).unwrap_or(u32::MAX)
}

/// **Acrescentar uma roldana a uma corda** — o pedido (4) do artista, *"escolher
/// o número de roldanas, em tempo real"*.
///
/// A roda nova entra ao FIM da rota, e **sobre a corda**: no meio do último
/// trecho que a rota desenha hoje. Duas razões, e as duas são sobre não
/// surpreender — ali o comprimento quase não muda (a corda já passava por aquele
/// ponto, e o que ela ganha é a diferença entre o arco e a corda do enlace, que é
/// pequena), e o artista vê a roda aparecer EM CIMA da corda que ele está
/// olhando, em vez de num canto que ele teria de ir procurar.
///
/// ⚠️ **Sem roldana nenhuma o "último trecho" é a corda inteira**, então a
/// primeira nasce no meio dela — que é onde uma roldana faria sentido.
///
/// O raio herda o da última roldana (ou o default), porque uma corda com rodas de
/// tamanhos aleatórios não é o que ninguém pede; e o `order` é o seguinte, então
/// a rota simplesmente cresce por onde ela já ia.
pub(crate) fn add_pulley_wheel(sim: &mut SimWorld, physics: &PhysicsBridge, joint_bits: u64) {
    let joint = Entity::from_bits(joint_bits);
    let Some(name) = sim
        .world()
        .get::<Name>(joint)
        .map(|n| n.as_str().to_string())
    else {
        return;
    };
    let rope = stable_name_id(&name);
    let Some(v) = physics.joint_views().find(|v| v.entity == joint) else {
        return;
    };
    // A geometria VIVA, pela mesma porta que o desenho usa — uma segunda
    // derivação poria a roda onde a corda não está.
    let wheels: Vec<_> = physics.rope_wheels(joint).map(|(_, w)| w).collect();
    let last = wheels.last().map_or(v.anchor_a, |w| w.centre);
    let centre = [
        0.5 * (last[0] + v.anchor_b[0]),
        0.5 * (last[1] + v.anchor_b[1]),
    ];
    let radius = wheels
        .last()
        .map_or(ph2d_physics_ecs::PulleyWheel::DEFAULT_RADIUS, |w| w.radius);
    let order = u16::try_from(wheels.len()).unwrap_or(u16::MAX);
    let label = crate::name_unique::unique_name(sim, &format!("{name} Wheel {}", order + 1));
    sim.world_mut().spawn((
        Name::new(label),
        ph2d_physics_ecs::PulleyWheel {
            rope,
            order,
            radius,
            wrap: ph2d_physics_ecs::WrapSide::Auto,
            motor_speed: 0.0,
            break_enabled: false,
            break_force: ph2d_physics_ecs::PulleyWheel::DEFAULT_BREAK_FORCE,
        },
        Transform::from_translation(ph2d_core::Vec2::new(centre[0], centre[1])),
    ));
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
}

/// **O snapshot da §13.** `None` para qualquer coisa que não seja uma roldana —
/// como a §12 e pelo mesmo motivo, esta seção não tem face vazia.
pub(crate) fn build_wheel_info(sim: &mut SimWorld, entity_bits: u64) -> Option<InspectorWheelInfo> {
    let entity = Entity::from_bits(entity_bits);
    let wheel = *sim.world().get::<ph2d_physics_ecs::PulleyWheel>(entity)?;
    // A corda a que ela pertence, resolvida do HASH para o nome. ⚠️ Exige um
    // `PhysicsJoint`, não só um nome que bate: uma roldana só entra numa rota se
    // o nome for o de uma CORDA, e um sprite homônimo não a põe em lugar nenhum.
    let mut q = sim
        .world_mut()
        .query::<(&Name, &ph2d_physics_ecs::PhysicsJoint)>();
    let world = sim.world();
    let rope_name = q
        .iter(world)
        .find(|(n, _)| stable_name_id(n.as_str()) == wheel.rope)
        .map(|(n, _)| n.as_str().to_string())
        .unwrap_or_default();
    Some(InspectorWheelInfo {
        entity_bits,
        bound: !rope_name.is_empty(),
        rope_name,
        radius: wheel.radius,
        // O componente conta de zero e a pessoa conta de um. A conversão mora
        // aqui e no `wheel_with_edit`, uma vez de cada lado.
        order_ui: u32::from(wheel.order) + 1,
        wrap_tag: wheel.wrap.tag(),
        // Radianos no componente, GRAUS na row — a fronteira do motor do Pin,
        // e a conversão mora aqui e no `wheel_with_edit`, uma vez de cada lado.
        motor_deg_per_s: wheel.motor_speed.to_degrees(),
        break_enabled: wheel.break_enabled,
        break_force: wheel.break_force,
    })
}

/// Aplica um [`WheelFieldEdit`], pelo mesmo funil do irmão `apply_joint_edit`:
/// lê a roldana viva e a escreve de volta mudada, porque uma escrita parcial
/// derrubaria os campos que não estão sendo editados.
pub(crate) fn apply_wheel_edit(
    sim: &SimWorld,
    entity_bits: u64,
    edit: WheelFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) {
    let entity = Entity::from_bits(entity_bits);
    let Some(&current) = sim.world().get::<ph2d_physics_ecs::PulleyWheel>(entity) else {
        return;
    };
    let Some(next) = wheel_with_edit(current, edit) else {
        return;
    };
    if next != current {
        queue_set(queue, registry, entity_bits, WHEEL, &next);
    }
}

/// **Uma edição aplicada a uma roldana** — a metade pura, e o funil único.
///
/// `None` quando a edição não é uma escrita de componente: um tag de `Wrap` que
/// não nomeia variante nenhum é **recusado**, nunca dobrado em `Auto`. Dobrar o
/// desconhecido no primeiro variant é o defeito que o `BodyKind` do W4 pagou —
/// com dois variants é redundante, com o terceiro vira um chip que seleciona
/// outra coisa.
#[must_use]
pub(crate) fn wheel_with_edit(
    current: ph2d_physics_ecs::PulleyWheel,
    edit: WheelFieldEdit,
) -> Option<ph2d_physics_ecs::PulleyWheel> {
    let mut next = current;
    match edit {
        WheelFieldEdit::Radius(v) => next.radius = v,
        // 1-based na row, 0-based no componente. `saturating_sub` e não `- 1`:
        // a fronteira do painel já põe o piso em 1, e um zero que escapasse por
        // outra rota viraria `u16::MAX` num wrap silencioso.
        WheelFieldEdit::Order(v) => {
            next.order = u16::try_from(v.saturating_sub(1)).unwrap_or(u16::MAX);
        }
        WheelFieldEdit::Wrap(tag) => next.wrap = ph2d_physics_ecs::WrapSide::from_tag(tag)?,
        // Graus na row, radianos no componente.
        WheelFieldEdit::MotorDegPerS(v) => next.motor_speed = v.to_radians(),
        WheelFieldEdit::BreakEnabled(on) => next.break_enabled = on,
        WheelFieldEdit::BreakForce(v) => next.break_force = v,
    }
    // A MESMA porta de carga que o load usa: raio negativo inverteria a
    // tangente, `NaN` envenenaria a pose e o hash C9.
    Some(next.clamped())
}
