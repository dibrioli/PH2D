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
    ("ph2d::ecs::AnchorMount", None),
    ("ph2d::ecs::AnchorVisibility", None),
    ("ph2d::ecs::BlendMode", None),
    ("ph2d::ecs::ClipChildren", None),
    (
        "ph2d::ecs::MaskInteraction",
        Some(|w, e| em(w, e, crate::sprite_inspector::sanitize_mask_interaction)),
    ),
    ("ph2d::ecs::OnScreenEnabler", None),
    ("ph2d::ecs::OrderInLayer", None),
    (
        "ph2d::ecs::SliceNine",
        Some(|w, e| em(w, e, crate::sprite_inspector::sanitize_slice_nine)),
    ),
    ("ph2d::ecs::SortingGroup", None),
    ("ph2d::ecs::SortingLayer", None),
    (
        "ph2d::ecs::SpriteAnimator",
        Some(|w, e| em(w, e, crate::sprite_inspector::sanitize_sprite_animator)),
    ),
    ("ph2d::ecs::SpriteCornerTint", None),
    (
        "ph2d::ecs::SpriteEmissive",
        Some(|w, e| em(w, e, crate::sprite_inspector::sanitize_sprite_emissive)),
    ),
    (
        "ph2d::ecs::SpriteGrid",
        Some(|w, e| em(w, e, crate::sprite_inspector::sanitize_sprite_grid)),
    ),
    (
        "ph2d::ecs::SpriteRegion",
        Some(|w, e| em(w, e, crate::sprite_inspector::sanitize_sprite_region)),
    ),
    ("ph2d::ecs::Tags", None),
    ("ph2d::ecs::TextureFilter", None),
    ("ph2d::ecs::TextureRepeat", None),
    ("ph2d::ecs::UvTransform", None),
    ("ph2d::ecs::Visibility", None),
    ("ph2d::ecs::VisibilityLayer", None),
    ("ph2d::ecs::YSort", None),
    ("ph2d::ecs::ZAsRelative", None),
    (
        "ph2d::ecs::ZIndexOverride",
        Some(|w, e| em(w, e, crate::sprite_inspector::sanitize_z_index)),
    ),
    (
        "ph2d::render::Sprite",
        Some(|w, e| em(w, e, crate::sprite_inspector::sanitize_sprite)),
    ),
    // — Física (catálogo `physics`)
    (
        "ph2d::physics::NavAgent",
        Some(|w, e| em(w, e, crate::nav_inspector::sanitize_nav_agent)),
    ),
    (
        "ph2d::physics::NavCostArea",
        Some(|w, e| em(w, e, crate::nav_inspector::sanitize_nav_cost_area)),
    ),
    (
        "ph2d::physics::NavLink",
        Some(|w, e| em(w, e, crate::nav_inspector::sanitize_nav_link)),
    ),
    (
        "ph2d::physics::NavRegion",
        Some(|w, e| em(w, e, crate::nav_inspector::sanitize_nav_region)),
    ),
    (
        "ph2d::physics::RaySensor",
        Some(|w, e| em(w, e, crate::ray_inspector::sanitize_ray_sensor)),
    ),
    ("ph2d::physics::RaySignals", None),
    (
        "ph2d::physics::ProjectileMotion",
        Some(|w, e| em(w, e, crate::projectile_inspector::sanitize_projectile)),
    ),
    (
        "ph2d::physics::TopDownPlayer",
        Some(|w, e| em(w, e, ph2d_physics_ecs::TopDownPlayer::sanitize)),
    ),
    (
        "ph2d::physics::PhysicsJoint",
        Some(|w, e| em(w, e, ph2d_physics_ecs::PhysicsJoint::sanitize)),
    ),
    (
        "ph2d::physics::PulleyWheel",
        Some(|w, e| em(w, e, ph2d_physics_ecs::PulleyWheel::sanitize)),
    ),
    (
        "ph2d::physics::PlatformPlayer",
        Some(|w, e| em(w, e, ph2d_physics_ecs::PlatformPlayer::sanitize)),
    ),
    ("ph2d::physics::PlayerMode", None),
    // As linhas da §11 (o cartão do corpo): a lei mora com o tipo (`components/body_law.rs`).
    ("ph2d::physics::RigidBody", None),
    (
        "ph2d::physics::Collider",
        Some(|w, e| em(w, e, ph2d_physics_ecs::Collider::sanitize)),
    ),
    ("ph2d::physics::GravityScale", None),
    ("ph2d::physics::InitialVelocity", None),
    ("ph2d::physics::Dominance", None),
    (
        "ph2d::physics::MassOverride",
        Some(|w, e| em(w, e, ph2d_physics_ecs::MassOverride::sanitize)),
    ),
    ("ph2d::physics::MaterialCombine", None),
    (
        "ph2d::physics::DampingOverride",
        Some(|w, e| em(w, e, ph2d_physics_ecs::DampingOverride::sanitize)),
    ),
    (
        "ph2d::physics::WalkSurface",
        Some(|w, e| em(w, e, ph2d_physics_ecs::WalkSurface::sanitize)),
    ),
    ("ph2d::physics::SignalOnHit", None),
    ("ph2d::physics::SignalOnLeave", None),
    ("ph2d::physics::SignalTagFilter", None),
    ("ph2d::physics::AreaEffector", None),
    (
        "ph2d::physics::AreaDrag",
        Some(|w, e| em(w, e, ph2d_physics_ecs::AreaDrag::sanitize)),
    ),
    (
        "ph2d::physics::AreaBuoyancy",
        Some(|w, e| em(w, e, ph2d_physics_ecs::AreaBuoyancy::sanitize)),
    ),
    (
        "ph2d::physics::AreaFormDrag",
        Some(|w, e| em(w, e, ph2d_physics_ecs::AreaFormDrag::sanitize)),
    ),
    ("ph2d::physics::AreaTorque", None),
    (
        "ph2d::physics::AreaFalloff",
        Some(|w, e| em(w, e, ph2d_physics_ecs::AreaFalloff::sanitize)),
    ),
    // — Lógica de jogo, HUD, câmera, som, partículas, script e luz
    ("ph2d::ecs::DestroyOutside", None),
    ("ph2d::ecs::Factory", None),
    ("ph2d::ecs::Lifetime", None),
    ("ph2d::ecs::Macro", None),
    ("ph2d::ecs::MotionInput", None),
    (
        "ph2d::ecs::PathFollow",
        Some(|w, e| em(w, e, crate::path_follow_inspector::sanitize_path_follow)),
    ),
    ("ph2d::ecs::SequencePlayer", None),
    ("ph2d::ecs::WeaponFire", None),
    (
        "ph2d::physics::Damage",
        Some(|w, e| em(w, e, crate::vida_inspector::sanitize_damage)),
    ),
    (
        "ph2d::physics::Health",
        Some(|w, e| em(w, e, crate::vida_inspector::sanitize_health)),
    ),
    (
        "ph2d::physics::HealthBar",
        Some(|w, e| em(w, e, crate::vida_inspector::sanitize_health_bar)),
    ),
    ("ph2d::ecs::Counter", None),
    ("ph2d::ecs::UiButton", None),
    (
        "ph2d::ecs::UiCanvas",
        Some(|w, e| em(w, e, crate::hud_inspector::sanitize_hud_canvas)),
    ),
    ("ph2d::ecs::UiLabel", None),
    (
        "ph2d::ecs::CameraFollow",
        Some(|w, e| em(w, e, crate::camera_inspector::sanitize_camera_follow)),
    ),
    ("ph2d::ecs::CameraLimits", None),
    (
        "ph2d::ecs::CameraShake",
        Some(|w, e| em(w, e, crate::shake_inspector::sanitize_camera_shake)),
    ),
    (
        "ph2d::ecs::GameCamera",
        Some(|w, e| em(w, e, crate::camera_inspector::sanitize_camera)),
    ),
    ("ph2d::ecs::ScrollFactor", None),
    ("ph2d::ecs::ScrollLimits", None),
    ("ph2d::ecs::ScrollMotion", None),
    (
        "ph2d::ecs::ScrollRepeat",
        Some(|w, e| em(w, e, crate::parallax_inspector::sanitize_parallax_repeat)),
    ),
    (
        "ph2d::ecs::AudioSource2D",
        Some(|w, e| em(w, e, crate::audio_inspector::sanitize_audio_source)),
    ),
    // A lei mora com o tipo; o cartão (`ph2d_app_light2d::inspector`) chama a mesma.
    (
        "ph2d::light2d::BloqueiaLuz",
        Some(|w, e| em(w, e, ph2d_light2d_ecs::BloqueiaLuz::sanitize)),
    ),
    (
        "ph2d::light2d::Luz",
        Some(|w, e| em(w, e, ph2d_light2d_ecs::Luz::sanitize)),
    ),
    ("ph2d::ecs::ParticleEmitter", None),
];

/// Aplica a lei do componente `canonical`, se ele tiver uma.
pub(crate) fn sanitize(canonical: &str, world: &mut World, entity: Entity) {
    if let Some((_, Some(lei))) = LEIS.iter().find(|(n, _)| *n == canonical) {
        lei(world, entity);
    }
}
