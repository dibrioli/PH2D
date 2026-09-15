//! **A secção PROJECTILE MOTION: o instantâneo e o dreno** (TOP-20 #14, W3).
//!
//! ⚠️ **As duas metades vivem juntas de propósito** — o mesmo molde do
//! [`super::inspector_topdown`]: quem lê o mundo para o painel e quem escreve a edição de volta
//! fazem a MESMA tradução, e separá-los seria a porta pela qual as duas divergem.
//!
//! # ⭐⭐ O que esta secção faz que a do irmão não faz: o ALVO é um NOME
//!
//! O componente guarda o `stable_name_id`; o painel mostra e recebe **texto**. A tradução é aqui, e
//! nos dois sentidos — ⛔ os bits de uma entidade não sobrevivem a um `Ctrl+Z`, e um alvo guardado
//! em bits apontaria para outra coisa (ou para nada) no primeiro undo.
//!
//! ⚠️ **E «o nome que ninguém tem» é uma resposta própria**, não um vazio: um alvo apagado tem de
//! se ler como *«este nome não existe»*, e não como *«sem alvo»*.

use ph2d_ecs::{Entity, Name, World, stable_name_id};
use ph2d_editor_core::projectile_edits::{InspectorProjectileInfo, ProjectileFieldEdit};
use ph2d_physics_ecs::{BodyKind, ProjectileMotion, RigidBody};

/// **O NOME de quem tem este `stable_name_id`** — e se ele existe.
///
/// ⚠️ **Devolve as DUAS coisas de uma vez.** Perguntar «qual é o nome?» e «ele existe?» em duas
/// varreduras seria duas respostas à mesma pergunta, e a segunda envelhece.
fn nome_do_alvo(world: &World, id: u64) -> (String, bool) {
    if id == 0 {
        return (String::new(), false);
    }
    let mut q = match world.try_query::<&Name>() {
        Some(q) => q,
        None => return (String::new(), true),
    };
    for n in q.iter(world) {
        if stable_name_id(n.as_str()) == id {
            return (n.as_str().to_string(), false);
        }
    }
    // ⚠️ **O nome perdeu-se com o objecto.** O componente guarda um hash, não o texto — então o
    // painel não tem como mostrar o nome que o artista escreveu. O que ele pode dizer é que o
    // objecto não está lá, e é isso que a segunda metade carrega.
    (String::new(), true)
}

/// **O instantâneo.** `None` para quem não tem o componente (ADR-0166).
pub(crate) fn build_projectile_info(
    world: &World,
    bits: u64,
    selected_count: usize,
    clock_playing: bool,
    flight_over: bool,
) -> Option<InspectorProjectileInfo> {
    let e = Entity::from_bits(bits);
    let c = world.get::<ProjectileMotion>(e)?;
    let law = c.law();
    let corpo = world.get::<RigidBody>(e);
    let (alvo, em_falta) = nome_do_alvo(world, c.homing_target);
    Some(InspectorProjectileInfo {
        entity_bits: bits,
        initial_speed: law.initial_speed,
        acceleration: law.acceleration,
        max_speed: law.max_speed,
        gravity: law.gravity,
        bounciness: law.bounciness,
        max_bounces: u32::from(law.max_bounces),
        range: law.range,
        face_velocity: law.face_velocity,
        homing_accel: law.homing_accel,
        homing_target: alvo,
        homing_target_missing: em_falta,
        has_body: corpo.is_some(),
        // ⚠️ **A pergunta é `== Kinematic` e não `!= Dynamic`**: um corpo estático também não serve,
        // e a forma negativa deixaria-o passar como se estivesse bem.
        body_is_kinematic: corpo.is_some_and(|b| b.kind == BodyKind::Kinematic),
        clock_playing,
        flight_over,
        selected_count,
    })
}

/// **O dreno.** `true` = tocou no mundo.
pub(crate) fn apply_projectile_edit(
    world: &mut World,
    bits: u64,
    edit: &ProjectileFieldEdit,
) -> bool {
    let e = Entity::from_bits(bits);
    if world.get_entity(e).is_err() {
        return false;
    }
    // ⚠️ **O alvo resolve-se ANTES do empréstimo mutável** — ele precisa de varrer o mundo, e
    // fazê-lo com o componente já emprestado não compila.
    let alvo = match edit {
        ProjectileFieldEdit::HomingTarget(nome) => {
            let t = nome.trim();
            // ⚠️ **Vazio é ZERO**, que é a convenção de «ninguém» desta casa — e não o hash de "".
            Some(if t.is_empty() { 0 } else { stable_name_id(t) })
        }
        _ => None,
    };
    let Some(mut c) = world.get_mut::<ProjectileMotion>(e) else {
        return false;
    };
    let mut law = c.law();
    let mut alvo_novo = c.homing_target;
    match edit {
        ProjectileFieldEdit::InitialSpeed(v) => law.initial_speed = v.max(0.0),
        // ⚠️ **A aceleração pode ser NEGATIVA** — é o que faz uma bala travar no ar, e prendê-la em
        // `0` apagaria metade do campo.
        ProjectileFieldEdit::Acceleration(v) => law.acceleration = *v,
        ProjectileFieldEdit::MaxSpeed(v) => law.max_speed = v.max(0.0),
        ProjectileFieldEdit::Gravity(v) => law.gravity = v.max(0.0),
        // ⚠️ **A cerca é a do PAINEL escrita também AQUI**: o campo é alcançável por outra rota (um
        // script, um ficheiro), e acima de `1` a bala **ganharia energia** a cada parede.
        ProjectileFieldEdit::Bounciness(v) => law.bounciness = v.clamp(0.0, 1.0),
        ProjectileFieldEdit::MaxBounces(n) => {
            law.max_bounces = u8::try_from((*n).min(32)).unwrap_or(0);
        }
        ProjectileFieldEdit::Range(v) => law.range = v.max(0.0),
        ProjectileFieldEdit::FaceVelocity(b) => law.face_velocity = *b,
        ProjectileFieldEdit::HomingAccel(v) => law.homing_accel = v.max(0.0),
        ProjectileFieldEdit::HomingTarget(_) => {
            alvo_novo = alvo.unwrap_or(0);
        }
    }
    *c = ProjectileMotion::from_law(law, alvo_novo);
    true
}

#[cfg(test)]
#[path = "inspector_projectile_tests.rs"]
mod tests;
