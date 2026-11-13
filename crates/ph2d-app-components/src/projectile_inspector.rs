//! **A secção PROJECTILE MOTION: o instantâneo, o dreno e o conta-gotas do Target** (TOP-20 #14, GEF 06).
//!
//! O ALVO é um NOME nos dois sentidos: o componente guarda o `stable_name_id` (os bits não sobrevivem a um
//! `Ctrl+Z`), o painel mostra e recebe texto, e «o nome que ninguém tem» é uma resposta própria, não um vazio.

use bevy_ecs::prelude::Resource;
use ph2d_ecs::{Entity, Name, World, stable_name_id};
use ph2d_editor_core::action_bus::{ActionBus, ComponentEdit, EditorAction};
use ph2d_editor_core::projectile_edits::{
    BOUNCES_AO_LIGAR, InspectorProjectileInfo, PULL_AO_LIGAR, ProjectileFieldEdit,
};
use ph2d_physics_ecs::{BodyKind, ProjectileMotion, RigidBody};

/// ⭐ GEF 06 — a bala cujo Target o próximo clique no palco escolhe (o conta-gotas). Estado de sessão.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AlvoArmado(pub Option<u64>);

/// O conta-gotas do Target está armado?
#[must_use]
pub fn armado(world: &World) -> bool {
    world
        .get_resource::<AlvoArmado>()
        .is_some_and(|a| a.0.is_some())
}

/// Desarma o conta-gotas (o `Esc`) e devolve a bala que estava armada.
pub fn desarma(world: &mut World) -> Option<u64> {
    world.get_resource_mut::<AlvoArmado>()?.0.take()
}

/// ⭐ **O clique do conta-gotas no palco**: o de cima em `debaixo` com nome, que não é a bala, vira o Target pela
/// porta do Inspector (um passo de desfazer); escolha ou vazio, desarma. `false` = não estava armado.
pub fn escolher_alvo(world: &mut World, bus: &mut ActionBus, debaixo: &[u64]) -> bool {
    let Some(bala) = desarma(world) else {
        return false;
    };
    let nome = debaixo.iter().filter(|&&b| b != bala).find_map(|&b| {
        world
            .get_entity(Entity::from_bits(b))
            .ok()
            .and_then(|e| e.get::<Name>().map(|n| n.as_str().trim().to_string()))
            .filter(|n| !n.is_empty())
    });
    if let Some(nome) = nome {
        bus.push(EditorAction::InspectorComponentEdit {
            entity_bits: bala,
            edit: ComponentEdit::Projectile(ProjectileFieldEdit::HomingTarget(nome)),
        });
    }
    true
}

/// **O NOME de quem tem este `stable_name_id`** e se ele falta — numa varredura só.
pub(crate) fn nome_do_alvo(world: &World, id: u64) -> (String, bool) {
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
    // O hash não devolve o texto: o painel só pode dizer que o objecto não está lá.
    (String::new(), true)
}

/// **O instantâneo.** `None` para quem não tem o componente (ADR-0166).
pub fn build_projectile_info(
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
        // `== Kinematic`, não `!= Dynamic`: um corpo estático também não serve.
        body_is_kinematic: corpo.is_some_and(|b| b.kind == BodyKind::Kinematic),
        clock_playing,
        flight_over,
        target_pick_armed: world
            .get_resource::<AlvoArmado>()
            .is_some_and(|a| a.0 == Some(bits)),
        selected_count,
    })
}

/// **O dreno.** `true` = tocou no mundo.
pub fn apply_projectile_edit(world: &mut World, bits: u64, edit: &ProjectileFieldEdit) -> bool {
    let e = Entity::from_bits(bits);
    if world.get_entity(e).is_err() {
        return false;
    }
    if let ProjectileFieldEdit::PickTarget(arma) = edit {
        // ⭐ O conta-gotas é da SESSÃO: armar não muda o documento (nenhum passo de desfazer).
        world.insert_resource(AlvoArmado(arma.then_some(bits)));
        return false;
    }
    // O alvo resolve-se ANTES do empréstimo mutável: ele varre o mundo.
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
        ProjectileFieldEdit::InitialSpeed(v) => law.initial_speed = *v,
        // A aceleração pode ser NEGATIVA: a bala trava no ar.
        ProjectileFieldEdit::Acceleration(v) => law.acceleration = *v,
        ProjectileFieldEdit::MaxSpeed(v) => law.max_speed = *v,
        ProjectileFieldEdit::Gravity(v) => law.gravity = *v,
        // A cerca do painel vale também aqui (script, ficheiro): acima de `1` a bala ganharia energia.
        ProjectileFieldEdit::Bounciness(v) => law.bounciness = *v,
        ProjectileFieldEdit::MaxBounces(n) => {
            law.max_bounces = u8::try_from((*n).min(32)).unwrap_or(0);
        }
        ProjectileFieldEdit::Range(v) => law.range = *v,
        ProjectileFieldEdit::FaceVelocity(b) => law.face_velocity = *b,
        ProjectileFieldEdit::HomingAccel(v) => law.homing_accel = *v,
        ProjectileFieldEdit::HomingTarget(_) => {
            alvo_novo = alvo.unwrap_or(0);
        }
        ProjectileFieldEdit::Bounce(on) => law.max_bounces = BOUNCES_AO_LIGAR * u8::from(*on),
        ProjectileFieldEdit::Homing(on) => {
            law.homing_accel = PULL_AO_LIGAR * f32::from(u8::from(*on))
        }
        ProjectileFieldEdit::PickTarget(_) => return false,
    }
    *c = ProjectileMotion::from_law(law, alvo_novo);
    sanitize_projectile(&mut c);
    true
}

/// ⭐ **A lei dos números do PROJÉCTIL** (painel e script, doc CodeEditor 08 §3.5): tudo `≥ 0` menos a
/// aceleração, quique em `0..=1`, no máximo 32 ressaltos.
pub(crate) fn sanitize_projectile(c: &mut ProjectileMotion) {
    for v in [
        &mut c.initial_speed,
        &mut c.max_speed,
        &mut c.gravity,
        &mut c.range,
        &mut c.homing_accel,
    ] {
        *v = v.max(0.0);
    }
    c.bounciness = c.bounciness.clamp(0.0, 1.0);
    c.max_bounces = c.max_bounces.min(32);
}

#[cfg(test)]
#[path = "projectile_inspector_tests.rs"]
mod tests;
