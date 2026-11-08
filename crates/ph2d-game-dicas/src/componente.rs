//! ⭐ **O Add Component em 11 assuntos** (plano 25, dono 08/11): grupo, posição, dica e desenho de cada
//! entrada, o porquê de cada peça que vem junto e o ensino da busca. Só DADOS e APRESENTAÇÃO — o catálogo
//! (`ComponentCategory`) não muda; quem monta o modelo é `ph2d_app_components::component_palette`.

use crate::componente_desenhos as d;
use ph2d_editor_core::widget::illustration::Mark;
use ph2d_i18n::TextKey;
use ph2d_tokens::ColorToken;

/// Os 11 assuntos, na ordem da janela (maquete B da Crítica).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Assunto {
    Move,
    Path,
    Rules,
    Screen,
    Camera,
    Background,
    Time,
    Look,
    Sound,
    Bones,
    Organize,
}

impl Assunto {
    pub const ALL: [Self; 11] = [
        Self::Move,
        Self::Path,
        Self::Rules,
        Self::Screen,
        Self::Camera,
        Self::Background,
        Self::Time,
        Self::Look,
        Self::Sound,
        Self::Bones,
        Self::Organize,
    ];

    /// O título, traduzido.
    #[must_use]
    pub fn title(self) -> &'static str {
        TextKey::new(match self {
            Self::Move => "component_add.group.move",
            Self::Path => "component_add.group.path",
            Self::Rules => "component_add.group.rules",
            Self::Screen => "component_add.group.screen",
            Self::Camera => "component_add.group.camera",
            Self::Background => "component_add.group.background",
            Self::Time => "component_add.group.time",
            Self::Look => "component_add.group.look",
            Self::Sound => "component_add.group.sound",
            Self::Bones => "component_add.group.bones",
            Self::Organize => "component_add.group.organize",
        })
        .tr()
    }

    /// A tinta: a da maquete B da Crítica (7 tokens `NodeCat*` para 11 assuntos; mexer-se e o som são
    /// saídas, regras e ecrã utilidades, câmera e fundo enquadram, tempo e ossos transformam).
    #[must_use]
    pub const fn color(self) -> ColorToken {
        match self {
            Self::Move | Self::Path | Self::Sound => ColorToken::NodeCatOutput,
            Self::Rules | Self::Screen => ColorToken::NodeCatUtility,
            Self::Camera | Self::Background => ColorToken::NodeCatFocus,
            Self::Time | Self::Bones => ColorToken::NodeCatTransform,
            Self::Look => ColorToken::NodeCatFx,
            Self::Organize => ColorToken::NodeCatSource,
        }
    }
}

/// O nome do item «Animation» (não é componente do mundo: são os clips do objecto na Timeline).
pub const ANIMATION: &str = "timeline.animation";

/// Uma entrada: o nome canónico (ou [`ANIMATION`]), o assunto, a frase da dica e o desenho. A posição é a
/// ordem em [`ENTRADAS`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Entrada {
    pub name: &'static str,
    pub assunto: Assunto,
    pub tip: TextKey,
    pub picture: &'static [Mark],
}

const fn e(
    name: &'static str,
    assunto: Assunto,
    tip: &'static str,
    picture: &'static [Mark],
) -> Entrada {
    Entrada {
        name,
        assunto,
        tip: TextKey::new(tip),
        picture,
    }
}

use Assunto as A;

/// ⭐ As 63 entradas, agrupadas e na ordem da janela.
pub const ENTRADAS: &[Entrada] = &[
    e(
        "ph2d::physics::RigidBody",
        A::Move,
        "component_add.tip.rigid_body",
        d::RIGID_BODY,
    ),
    e(
        "ph2d::physics::TopDownPlayer",
        A::Move,
        "component_add.tip.top_down_player",
        d::TOP_DOWN_PLAYER,
    ),
    e(
        "ph2d::physics::ProjectileMotion",
        A::Move,
        "component_add.tip.projectile_motion",
        d::PROJECTILE_MOTION,
    ),
    e(
        "ph2d::ecs::PathFollow",
        A::Move,
        "component_add.tip.path_follow",
        d::PATH_FOLLOW,
    ),
    e(
        "ph2d::ecs::MotionInput",
        A::Move,
        "component_add.tip.motion_input",
        d::MOTION_INPUT,
    ),
    e(
        "ph2d::physics::RaySensor",
        A::Move,
        "component_add.tip.ray_sensor",
        d::RAY_SENSOR,
    ),
    e(
        "ph2d::physics::NavAgent",
        A::Path,
        "component_add.tip.nav_agent",
        d::NAV_AGENT,
    ),
    e(
        "ph2d::physics::NavRegion",
        A::Path,
        "component_add.tip.nav_region",
        d::NAV_REGION,
    ),
    e(
        "ph2d::physics::NavCostArea",
        A::Path,
        "component_add.tip.nav_cost_area",
        d::NAV_COST_AREA,
    ),
    e(
        "ph2d::physics::NavLink",
        A::Path,
        "component_add.tip.nav_link",
        d::NAV_LINK,
    ),
    e(
        "ph2d::ecs::SignalActions",
        A::Rules,
        "component_add.tip.signal_actions",
        d::SIGNAL_ACTIONS,
    ),
    e(
        "ph2d::ecs::SignalOnAction",
        A::Rules,
        "component_add.tip.signal_on_action",
        d::SIGNAL_ON_ACTION,
    ),
    e(
        "ph2d::ecs::Counter",
        A::Rules,
        "component_add.tip.counter",
        d::COUNTER,
    ),
    e(
        "ph2d::ecs::CounterWatch",
        A::Rules,
        "component_add.tip.counter_watch",
        d::COUNTER_WATCH,
    ),
    e(
        "ph2d::ecs::Timers",
        A::Rules,
        "component_add.tip.timers",
        d::TIMERS,
    ),
    e(
        "ph2d::ecs::Lifetime",
        A::Rules,
        "component_add.tip.lifetime",
        d::LIFETIME,
    ),
    e(
        "ph2d::ecs::DestroyOutside",
        A::Rules,
        "component_add.tip.destroy_outside",
        d::DESTROY_OUTSIDE,
    ),
    e(
        "ph2d::ecs::Factory",
        A::Rules,
        "component_add.tip.factory",
        d::FACTORY,
    ),
    e(
        "ph2d::ecs::WeaponFire",
        A::Rules,
        "component_add.tip.weapon_fire",
        d::WEAPON_FIRE,
    ),
    e(
        "ph2d::physics::Damage",
        A::Rules,
        "component_add.tip.damage",
        d::DAMAGE,
    ),
    e(
        "ph2d::physics::Health",
        A::Rules,
        "component_add.tip.health",
        d::HEALTH,
    ),
    e(
        "ph2d::ecs::UiCanvas",
        A::Screen,
        "component_add.tip.ui_canvas",
        d::UI_CANVAS,
    ),
    e(
        "ph2d::ecs::UiLabel",
        A::Screen,
        "component_add.tip.ui_label",
        d::UI_LABEL,
    ),
    e(
        "ph2d::ecs::UiButton",
        A::Screen,
        "component_add.tip.ui_button",
        d::UI_BUTTON,
    ),
    e(
        "ph2d::physics::HealthBar",
        A::Screen,
        "component_add.tip.health_bar",
        d::HEALTH_BAR,
    ),
    e(
        "ph2d::ecs::NamedAnchorList",
        A::Screen,
        "component_add.tip.named_anchor_list",
        d::NAMED_ANCHOR_LIST,
    ),
    e(
        "ph2d::ecs::AnchorMount",
        A::Screen,
        "component_add.tip.anchor_mount",
        d::ANCHOR_MOUNT,
    ),
    e(
        "ph2d::ecs::ClipChildren",
        A::Screen,
        "component_add.tip.clip_children",
        d::CLIP_CHILDREN,
    ),
    e(
        "ph2d::ecs::CameraFollow",
        A::Camera,
        "component_add.tip.camera_follow",
        d::CAMERA_FOLLOW,
    ),
    e(
        "ph2d::ecs::CameraLimits",
        A::Camera,
        "component_add.tip.camera_limits",
        d::CAMERA_LIMITS,
    ),
    e(
        "ph2d::ecs::CameraShake",
        A::Camera,
        "component_add.tip.camera_shake",
        d::CAMERA_SHAKE,
    ),
    e(
        "ph2d::ecs::ShakeEmitter",
        A::Camera,
        "component_add.tip.shake_emitter",
        d::SHAKE_EMITTER,
    ),
    e(
        "ph2d::ecs::ScrollFactor",
        A::Background,
        "component_add.tip.scroll_factor",
        d::SCROLL_FACTOR,
    ),
    e(
        "ph2d::ecs::ScrollLimits",
        A::Background,
        "component_add.tip.scroll_limits",
        d::SCROLL_LIMITS,
    ),
    e(
        "ph2d::ecs::ScrollMotion",
        A::Background,
        "component_add.tip.scroll_motion",
        d::SCROLL_MOTION,
    ),
    e(
        "ph2d::ecs::ScrollRepeat",
        A::Background,
        "component_add.tip.scroll_repeat",
        d::SCROLL_REPEAT,
    ),
    e(
        ANIMATION,
        A::Time,
        "component_add.tip.animation",
        d::ANIMATION,
    ),
    e(
        "ph2d::ecs::SpriteAnimations",
        A::Time,
        "component_add.tip.sprite_animations",
        d::SPRITE_ANIMATIONS,
    ),
    e(
        "ph2d::ecs::SpriteAnimator",
        A::Time,
        "component_add.tip.sprite_animator",
        d::SPRITE_ANIMATOR,
    ),
    e(
        "ph2d::ecs::StateMachine",
        A::Time,
        "component_add.tip.state_machine",
        d::STATE_MACHINE,
    ),
    e(
        "ph2d::ecs::Tweens",
        A::Time,
        "component_add.tip.tweens",
        d::TWEENS,
    ),
    e(
        "ph2d::ecs::SequencePlayer",
        A::Time,
        "component_add.tip.sequence_player",
        d::SEQUENCE_PLAYER,
    ),
    e(
        "ph2d::script::LuauScript",
        A::Time,
        "component_add.tip.luau_script",
        d::LUAU_SCRIPT,
    ),
    e(
        "ph2d::ecs::BlendMode",
        A::Look,
        "component_add.tip.blend_mode",
        d::BLEND_MODE,
    ),
    e(
        "ph2d::ecs::Mask2D",
        A::Look,
        "component_add.tip.mask_2d",
        d::MASK_2D,
    ),
    e(
        "ph2d::ecs::MaskInteraction",
        A::Look,
        "component_add.tip.mask_interaction",
        d::MASK_INTERACTION,
    ),
    e(
        "ph2d::ecs::SliceNine",
        A::Look,
        "component_add.tip.slice_nine",
        d::SLICE_NINE,
    ),
    e(
        "ph2d::ecs::TextureFilter",
        A::Look,
        "component_add.tip.texture_filter",
        d::TEXTURE_FILTER,
    ),
    e(
        "ph2d::ecs::TextureRepeat",
        A::Look,
        "component_add.tip.texture_repeat",
        d::TEXTURE_REPEAT,
    ),
    e(
        "ph2d::ecs::UvTransform",
        A::Look,
        "component_add.tip.uv_transform",
        d::UV_TRANSFORM,
    ),
    e(
        "ph2d::ecs::ParticleEmitter",
        A::Look,
        "component_add.tip.particle_emitter",
        d::PARTICLE_EMITTER,
    ),
    e(
        "ph2d::light2d::BloqueiaLuz",
        A::Look,
        "component_add.tip.bloqueia_luz",
        d::BLOQUEIA_LUZ,
    ),
    e(
        "ph2d::light2d::NaoRecebeLuz",
        A::Look,
        "component_add.tip.nao_recebe_luz",
        d::NAO_RECEBE_LUZ,
    ),
    e(
        "ph2d::ecs::AudioSource2D",
        A::Sound,
        "component_add.tip.audio_source_2d",
        d::AUDIO_SOURCE_2D,
    ),
    e(
        "ph2d::ecs::AudioListener2D",
        A::Sound,
        "component_add.tip.audio_listener_2d",
        d::AUDIO_LISTENER_2D,
    ),
    e(
        "ph2d::skeleton::Bone",
        A::Bones,
        "component_add.tip.bone",
        d::BONE,
    ),
    e(
        "ph2d::skeleton::BoneLimit",
        A::Bones,
        "component_add.tip.bone_limit",
        d::BONE_LIMIT,
    ),
    e(
        "ph2d::skeleton::SmartBone",
        A::Bones,
        "component_add.tip.smart_bone",
        d::SMART_BONE,
    ),
    e(
        "ph2d::ecs::Transform",
        A::Organize,
        "component_add.tip.transform",
        d::TRANSFORM,
    ),
    e(
        "ph2d::ecs::Tags",
        A::Organize,
        "component_add.tip.tags",
        d::TAGS,
    ),
    e(
        "ph2d::constraints::Constraints",
        A::Organize,
        "component_add.tip.constraints",
        d::CONSTRAINTS,
    ),
    e(
        "ph2d::ecs::VisibilityLayer",
        A::Organize,
        "component_add.tip.visibility_layer",
        d::VISIBILITY_LAYER,
    ),
    e(
        "ph2d::ecs::OnScreenEnabler",
        A::Organize,
        "component_add.tip.on_screen_enabler",
        d::ON_SCREEN_ENABLER,
    ),
];

/// A entrada de `name`, se a tabela a tem.
#[must_use]
pub fn entrada(name: &str) -> Option<&'static Entrada> {
    ENTRADAS.iter().find(|e| e.name == name)
}

/// ⭐ **O porquê de cada peça que vem junto** — uma linha por aresta `requires` do catálogo (o gate
/// `toda_dependencia_tem_o_seu_porque`). `(quem pede, o que vem, a frase)`.
pub const PORQUE: &[(&str, &str, TextKey)] = &[
    (
        "ph2d::physics::RigidBody",
        "ph2d::physics::Collider",
        TextKey::new("component_add.why.shape"),
    ),
    (
        "ph2d::physics::TopDownPlayer",
        "ph2d::physics::RigidBody",
        TextKey::new("component_add.why.body_moves"),
    ),
    (
        "ph2d::physics::ProjectileMotion",
        "ph2d::physics::RigidBody",
        TextKey::new("component_add.why.body_flies"),
    ),
    (
        "ph2d::physics::Damage",
        "ph2d::physics::RigidBody",
        TextKey::new("component_add.why.body_touches"),
    ),
    (
        "ph2d::physics::Health",
        "ph2d::physics::RigidBody",
        TextKey::new("component_add.why.body_is_hit"),
    ),
    (
        "ph2d::physics::NavAgent",
        "ph2d::physics::TopDownPlayer",
        TextKey::new("component_add.why.agent_walks"),
    ),
    (
        "ph2d::ecs::Factory",
        "ph2d::ecs::Timers",
        TextKey::new("component_add.why.clock"),
    ),
    (
        "ph2d::ecs::PathFollow",
        "ph2d::ecs::Timers",
        TextKey::new("component_add.why.clock"),
    ),
    (
        "ph2d::ecs::SequencePlayer",
        "ph2d::ecs::Timers",
        TextKey::new("component_add.why.clock"),
    ),
    (
        "ph2d::ecs::Tweens",
        "ph2d::ecs::Timers",
        TextKey::new("component_add.why.clock"),
    ),
    (
        "ph2d::ecs::WeaponFire",
        "ph2d::ecs::Counter",
        TextKey::new("component_add.why.ammo"),
    ),
    (
        "ph2d::ecs::ScrollLimits",
        "ph2d::ecs::ScrollFactor",
        TextKey::new("component_add.why.parallax"),
    ),
    (
        "ph2d::ecs::ScrollMotion",
        "ph2d::ecs::ScrollFactor",
        TextKey::new("component_add.why.parallax"),
    ),
    (
        "ph2d::ecs::ScrollRepeat",
        "ph2d::ecs::ScrollFactor",
        TextKey::new("component_add.why.parallax"),
    ),
];

/// O porquê de `needs` trazer `needed`.
#[must_use]
pub fn porque(needs: &str, needed: &str) -> Option<TextKey> {
    PORQUE
        .iter()
        .find(|(a, b, _)| *a == needs && *b == needed)
        .map(|(_, _, k)| *k)
}

/// ⭐ **O que vem JUNTO com `name`**, pela ordem em que a porta os anexa, cada um com quem o pede: o fecho
/// TRANSITIVO de `requires` (Nav Agent traz Top-Down Player, que traz Physics Body, que traz a forma), sem o
/// que o objecto `ja` tem — a porta não anexa duas vezes.
#[must_use]
pub fn trazidos(name: &str, ja: &[&str]) -> Vec<(&'static str, &'static str)> {
    let mut out: Vec<(&'static str, &'static str)> = Vec::new();
    let mut stack: Vec<&'static str> = ph2d_component_desc::desc_for(name)
        .map(|d| vec![d.canonical_name])
        .unwrap_or_default();
    while let Some(n) = stack.pop() {
        let Some(d) = ph2d_component_desc::desc_for(n) else {
            continue;
        };
        for r in d.requires {
            if *r != name && !out.iter().any(|(_, b)| b == r) {
                out.push((d.canonical_name, *r));
                stack.push(r);
            }
        }
    }
    out.retain(|(_, b)| !ja.contains(b));
    out
}

/// O nome que o artista lê de um componente.
#[must_use]
pub fn nome(canonical: &str) -> &'static str {
    ph2d_component_desc::desc_for(canonical).map_or("", |d| ph2d_i18n::tr(d.display_key))
}

/// ⭐ **O recibo** (a maquete A 4 da Crítica): «Physics Body added. Also added: Collision Shape, because a
/// body needs a shape.» — `None` quando nada veio junto.
#[must_use]
pub fn recibo(name: &str, ja: &[&str]) -> Option<String> {
    let vieram = trazidos(name, ja);
    if vieram.is_empty() {
        return None;
    }
    let partes: Vec<String> = vieram
        .iter()
        .map(|(a, b)| {
            let why = porque(a, b).map_or("", TextKey::tr);
            TextKey::new("component_add.receipt.because")
                .tr()
                .replace("{name}", nome(b))
                .replace("{why}", why)
        })
        .collect();
    Some(
        TextKey::new("component_add.receipt")
            .tr()
            .replace("{name}", nome(name))
            .replace("{list}", &partes.join("; ")),
    )
}

/// ⭐ **O que a busca que não acha ensina** (plano 25 §2.5): `(frase, palavras, por onde começar)`, chaves
/// i18n. Só cita peças que existem e só dispara sem resultado (gate
/// `cada_ensino_do_add_component_so_cita_o_que_existe_e_so_aparece_sem_resultado`).
pub const TEACH: &[(TextKey, TextKey, &str)] = &[
    (
        TextKey::new("component_add.teach.platform_player"),
        TextKey::new("component_add.teach.platform_player.words"),
        "ph2d::physics::RigidBody",
    ),
    (
        TextKey::new("component_add.teach.collision_shape"),
        TextKey::new("component_add.teach.collision_shape.words"),
        "ph2d::physics::RigidBody",
    ),
];

#[cfg(test)]
#[path = "componente_tests.rs"]
mod tests;
