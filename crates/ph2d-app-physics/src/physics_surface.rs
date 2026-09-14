//! **De que esta SUPERFÍCIE é feita para quem anda sobre ela** — a metade de
//! ESCRITA (`W-Surface`).
//!
//! Módulo irmão de [`crate::physics_apply`] e
//! [`crate::physics_area`], e o corte segue a mesma linha que os dois
//! já traçavam: lá mora *o que este CORPO é*, ao lado *o que esta ÁREA faz a
//! outros*, e aqui *de que esta SUPERFÍCIE é feita*.
//!
//! ⚠️ **E ele entra na lista de escritores do
//! `every_physics_component_is_authorable`** — o gate ENUMERA os arquivos por
//! onde uma edição vira componente, e um corte que move um escritor para fora
//! da lista o deixa VERMELHO nomeando o componente órfão. É a falha alta que a
//! lista existe para produzir, e o W-AreaFalloff já a pagou uma vez.

use ph2d_ecs::Entity;
use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_editor_core::PhysicsFieldEdit;

use ph2d_inspector_ordering::{queue_remove, queue_set};

/// O nome canônico — a MESMA string que o `queue_set` precisa para achar o
/// `type_id`.
const WALK_SURFACE: &str = "ph2d::physics::WalkSurface";

/// Escreve a superfície, ou devolve `false` se a edição não é uma delas.
pub(super) fn apply_surface_edit(
    world: &bevy_ecs::world::World,
    entity: Entity,
    entity_bits: u64,
    edit: PhysicsFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) -> bool {
    use ph2d_physics_ecs::{Collider, WalkSurface};

    if !matches!(
        edit,
        PhysicsFieldEdit::WalkGrip(_) | PhysicsFieldEdit::WalkBelt(_)
    ) {
        return false;
    }
    // **A superfície de caminhada** (W-Surface): read-modify-write do ÚNICO
    // campo que a edição nomeia no componente opcional `WalkSurface` — uma
    // escrita parcial derrubaria o outro.
    //
    // ⚠️ **Gateado no COLLIDER, e não no `RigidBody` como os irmãos acima —
    // a diferença é a razão de a wave existir.** Uma superfície é
    // propriedade da FACE que o pé encontra, e a face pode ser a PEÇA de um
    // corpo composto (W-Compound), que carrega `Collider` e **não** carrega
    // `RigidBody`. Herdar o guard do vizinho tornaria inexprimível
    // exatamente o caso de uso que a wave testa — uma plataforma com uma
    // face de gelo e outra de borracha.
    if world.get::<Collider>(entity).is_none() {
        return true;
    }
    let mut surf = world
        .get::<WalkSurface>(entity)
        .copied()
        .unwrap_or_default();
    match edit {
        // ⚠️ A tração é um multiplicador de ORÇAMENTO: negativa ela pagaria
        // o personagem para se afastar do alvo, e o piso é o gelo perfeito.
        PhysicsFieldEdit::WalkGrip(v) => surf.grip = v.max(0.0),
        // ⚠️ E a correia é COM SINAL, sem piso: o sinal É a direção ao longo
        // da tangente, e clampá-lo em zero deixaria metade das esteiras
        // inexprimível.
        PhysicsFieldEdit::WalkBelt(v) => surf.belt = v,
        _ => unreachable!(),
    }
    // Detach no neutro (`grip 1`, `belt 0`) — o idioma do presence-override:
    // um chão comum não carrega componente, e o arquivo fica livre do no-op.
    if surf.is_neutral() {
        queue_remove(queue, registry, entity_bits, WALK_SURFACE);
    } else {
        queue_set(queue, registry, entity_bits, WALK_SURFACE, &surf);
    }
    true
}

const SIGNAL_TAG_FILTER: &str = "ph2d::physics::SignalTagFilter";

/// ⭐⭐⭐ **Escreve o FILTRO por tag dos sinais de colisão** (TOP-20 #9, W3c), ou devolve `false` se a
/// edição não é essa.
///
/// ⚠️ **Irmão do [`apply_surface_edit`] e não um braço do `match` dele**, pela razão que aquele já
/// escreve: o componente é OPCIONAL, e um braço no `match` do collider faria uma escrita parcial.
///
/// ⭐ **`0` DESANEXA**, pelo idioma do *presence-override* que a superfície acima usa: *«sem filtro»*
/// é a ausência do componente, não um componente com um valor neutro dentro. ⛔ Guardar `Some(0)` no
/// documento poria no ficheiro um no-op que o `signal_passes` teria de aprender a ignorar — e a cena
/// deixaria de ser byte-idêntica à de quem nunca lhe tocou.
///
/// ⚠️ **Gateado no COLLIDER**, como o irmão e pela mesma razão: um filtro é propriedade da FACE que
/// grita, e uma peça de corpo composto tem `Collider` sem `RigidBody`.
pub(super) fn apply_signal_tag_filter_edit(
    world: &bevy_ecs::world::World,
    entity: Entity,
    entity_bits: u64,
    edit: PhysicsFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) -> bool {
    use ph2d_physics_ecs::{Collider, SignalTagFilter};

    let PhysicsFieldEdit::SignalTagFilter(id) = edit else {
        return false;
    };
    if world.get::<Collider>(entity).is_none() {
        return true;
    }
    if id == 0 {
        queue_remove(queue, registry, entity_bits, SIGNAL_TAG_FILTER);
    } else {
        queue_set(
            queue,
            registry,
            entity_bits,
            SIGNAL_TAG_FILTER,
            &SignalTagFilter(id),
        );
    }
    true
}
