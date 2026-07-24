//! §11 — **as rows da ZONA**: o que uma ÁREA faz aos corpos que estão dentro dela.
//!
//! Irmão do [`super::inspector_physics_apply`], e o corte é o mesmo que
//! `ph2d-physics-ecs::components::area` faz do lado do modelo: lá mora *o que este CORPO
//! é* (a gravidade dele, a massa dele, os eixos travados), aqui *o que esta ÁREA faz a
//! OUTROS* — um vento, um redemoinho, uma poça, e agora o falloff que os enfraquece na
//! borda. Nasceu do cap de 600 LOC do shell quando o sétimo componente da mesma área
//! chegou (W-AreaFalloff), e a família continua crescendo: a próxima wave de zona
//! aterrissa aqui.
//!
//! **Todas são SENSOR-only**, e é sempre o mesmo motivo: a narrow phase registra
//! sobreposição só quando um dos lados é sensor, e um collider sólido empurra os corpos
//! para FORA em vez de deixá-los entrar. Numa zona sólida qualquer destes números seria
//! autorado e nunca lido. O painel oferece as rows sob exatamente essa condição — e uma
//! recusa que mora no laço de pintura não é uma recusa, então ela é repetida aqui.

use bevy_ecs::world::World;
use ph2d_ecs::Entity;
use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_editor::PhysicsFieldEdit;

use super::inspector_ordering::{queue_remove, queue_set};

/// Aplica um edit de ZONA, ou devolve `false` se este edit não é de zona — e então o irmão
/// segue com ele. O booleano é o mesmo `return` que os braços de lá fazem, atravessando a
/// fronteira de módulo.
pub(crate) fn apply_area_edit(
    world: &World,
    entity: Entity,
    entity_bits: u64,
    edit: PhysicsFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) -> bool {
    use ph2d_physics_ecs::{
        AreaBuoyancy, AreaDrag, AreaEffector, AreaFalloff, AreaFormDrag, AreaTorque, Collider,
    };
    const AREA_EFFECTOR: &str = "ph2d::physics::AreaEffector";
    const AREA_DRAG: &str = "ph2d::physics::AreaDrag";
    const AREA_BUOYANCY: &str = "ph2d::physics::AreaBuoyancy";
    const AREA_FORM_DRAG: &str = "ph2d::physics::AreaFormDrag";
    const AREA_TORQUE: &str = "ph2d::physics::AreaTorque";
    const AREA_FALLOFF: &str = "ph2d::physics::AreaFalloff";

    if let PhysicsFieldEdit::AreaFormDrag(v) = edit {
        // O arrasto de FORMA (W-FormDrag) — quarto componente desta área, mesmo gate
        // SENSOR dos irmãos, mesmo clamp (um coeficiente negativo adicionaria energia).
        if !world.get::<Collider>(entity).is_some_and(|c| c.is_sensor) {
            return true;
        }
        let f = AreaFormDrag(v.max(0.0));
        if f.is_neutral() {
            queue_remove(queue, registry, entity_bits, AREA_FORM_DRAG);
        } else {
            queue_set(queue, registry, entity_bits, AREA_FORM_DRAG, &f);
        }
        return true;
    }

    if let PhysicsFieldEdit::AreaTorque(v) = edit {
        // O torque de área (W-AreaTorque) — o quinto componente desta zona, mesmo gate
        // SENSOR dos irmãos. ⚠️ NÃO clampa em `>= 0`: o sinal é o SENTIDO (negativo =
        // horário), então só o zero exato destaca (`is_neutral` == 0.0). Clampar tiraria
        // metade dos redemoinhos.
        if !world.get::<Collider>(entity).is_some_and(|c| c.is_sensor) {
            return true;
        }
        let t = AreaTorque(v);
        if t.is_neutral() {
            queue_remove(queue, registry, entity_bits, AREA_TORQUE);
        } else {
            queue_set(queue, registry, entity_bits, AREA_TORQUE, &t);
        }
        return true;
    }

    if let PhysicsFieldEdit::AreaFalloff(v) = edit {
        // O falloff (W-AreaFalloff) — o sétimo componente desta zona, mesmo gate SENSOR
        // dos irmãos. Clampado em `0..=1`: é uma FRAÇÃO do empurrão que se perde no
        // caminho, então negativo não é uma coisa e acima de 1 já é tudo (o cap de `t`
        // no kernel faria o resto virar zero de qualquer forma — melhor recusar aqui, na
        // fronteira onde o número é autorado, do que deixar a UI mostrar 3).
        if !world.get::<Collider>(entity).is_some_and(|c| c.is_sensor) {
            return true;
        }
        let f = AreaFalloff(v.clamp(0.0, 1.0));
        if f.is_neutral() {
            queue_remove(queue, registry, entity_bits, AREA_FALLOFF);
        } else {
            queue_set(queue, registry, entity_bits, AREA_FALLOFF, &f);
        }
        return true;
    }

    if let PhysicsFieldEdit::AreaDensity(v) = edit {
        // A densidade do fluido (W-Buoyancy) — o terceiro componente desta área, mesmo
        // gate SENSOR dos irmãos. Uma densidade negativa não é uma coisa; zero é a área
        // sem empuxo, e destaca.
        if !world.get::<Collider>(entity).is_some_and(|c| c.is_sensor) {
            return true;
        }
        let b = AreaBuoyancy(v.max(0.0));
        if b.is_neutral() {
            queue_remove(queue, registry, entity_bits, AREA_BUOYANCY);
        } else {
            queue_set(queue, registry, entity_bits, AREA_BUOYANCY, &b);
        }
        return true;
    }

    if let PhysicsFieldEdit::AreaDrag(v) = edit {
        // The medium half of a force zone (W-AreaDrag). Its OWN component, so a zone
        // that only resists carries no force blob and vice versa — and so that adding
        // it cost no `PROJECT_SCHEMA` bump. Same SENSOR gate as Force: the narrow phase
        // records an overlap only for a sensor, so on a solid collider this would be a
        // number nothing ever reads.
        if !world.get::<Collider>(entity).is_some_and(|c| c.is_sensor) {
            return true;
        }
        // A negative drag would ADD energy — the same clamp the other drag knobs take.
        let d = AreaDrag(v.max(0.0));
        if d.is_neutral() {
            queue_remove(queue, registry, entity_bits, AREA_DRAG);
        } else {
            queue_set(queue, registry, entity_bits, AREA_DRAG, &d);
        }
        return true;
    }

    if let PhysicsFieldEdit::ForceX(_) | PhysicsFieldEdit::ForceY(_) = edit {
        // Force zone (W-Area): a per-collider push carried by the optional
        // `AreaEffector`, read-modify-write so editing one axis keeps the other.
        // ⚠️ Gated on the collider being a SENSOR, not on a body kind — the narrow
        // phase records an overlap only for a sensor, so on a solid collider this
        // would be a number the artist authored and nothing would ever read.
        let Some(col) = world.get::<Collider>(entity).copied() else {
            return true;
        };
        if !col.is_sensor {
            return true;
        }
        let mut a = world
            .get::<AreaEffector>(entity)
            .copied()
            .unwrap_or_default();
        match edit {
            // Signed: a wind blows either way, so no clamp.
            PhysicsFieldEdit::ForceX(v) => a.force[0] = v,
            PhysicsFieldEdit::ForceY(v) => a.force[1] = v,
            _ => unreachable!(),
        }
        // Detach at neutral (zero on both axes) so an area that pushes nothing carries
        // no component — the presence-override idiom.
        if a.is_neutral() {
            queue_remove(queue, registry, entity_bits, AREA_EFFECTOR);
        } else {
            queue_set(queue, registry, entity_bits, AREA_EFFECTOR, &a);
        }
        return true;
    }

    false
}
