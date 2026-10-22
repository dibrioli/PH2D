//! ⭐ **A lei dos valores de cada componente com nome** — a MESMA função que o cartão do Inspector
//! chama no seu `apply_*` e que o script chama depois de `ph2d.set` (doc CodeEditor 08 §3.5): o
//! script não consegue pôr o que o painel recusa.
//!
//! Uma linha por componente com `code` (gate `todo_componente_com_nome_tem_a_sua_lei`); `None` = o
//! cartão aceita tudo o que o tipo aceita. Por família, cada uma no seu bloco.

use bevy_ecs::component::{Component, Mutable};
use ph2d_ecs::{Entity, World};

/// O saneamento de um componente na entidade.
pub(crate) type Lei = Option<fn(&mut World, Entity)>;

fn em<T: Component<Mutability = Mutable>>(world: &mut World, entity: Entity, lei: fn(&mut T)) {
    if let Some(mut c) = world.get_mut::<T>(entity) {
        lei(&mut c);
    }
}

pub(crate) const LEIS: &[(&str, Lei)] = &[
    // — Núcleo e imagem (catálogo `core`, `image`)
    // — Física (catálogo `physics`)
    // — Lógica de jogo, HUD, câmera, som, partículas, script e luz
    ("ph2d::physics::Health", Some(|w, e| em(w, e, crate::vida_inspector::sanitize_health))),
    ("ph2d::ecs::GameCamera", Some(|w, e| em(w, e, crate::camera_inspector::sanitize_camera))),
];

/// Aplica a lei do componente `canonical`, se ele tiver uma.
pub(crate) fn sanitize(canonical: &str, world: &mut World, entity: Entity) {
    if let Some((_, Some(lei))) = LEIS.iter().find(|(n, _)| *n == canonical) {
        lei(world, entity);
    }
}
