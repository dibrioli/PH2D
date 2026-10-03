//! **O modo do objecto activo, com o tipo perguntado ao MARCADOR** (spec/06 F2) — a porta que a
//! shell chama em todo quadro. O quadro em si é `ph2d_editor_core::screens::hero::mode_drive`; aqui
//! só se responde «que tipo e que nome tem esta entidade», que é desta crate
//! ([`crate::component_attach::kind_of`]).

use ph2d_editor_core::ToolRegistry;
use ph2d_editor_core::object_mode::ModeRequest;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::screens::hero::mode_drive::{self, ModeFamily};
use ph2d_editor_core::toast::ToastQueue;

/// Corre o quadro do modo sobre `world`. Devolve se o modo mudou.
pub fn drive(
    world: &ph2d_ecs::World,
    families: &[ModeFamily],
    tools: &mut ToolRegistry,
    hero: &mut HeroScreen,
    toasts: &mut ToastQueue,
    request: Option<ModeRequest>,
) -> bool {
    let entity = ph2d_ecs::Entity::from_bits;
    let kind_of = |b| crate::component_attach::kind_of(world, entity(b));
    let name_of = |b| {
        world
            .get::<ph2d_ecs::Name>(entity(b))
            .map_or_else(String::new, |n| n.0.clone())
    };
    mode_drive::drive(families, &kind_of, &name_of, tools, hero, toasts, request)
}
