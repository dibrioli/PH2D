//! ⭐⭐ **O TIPO de cada objecto na Hierarquia — um ícone ÚNICO e uma cor PRÓPRIA** (F42, dono 01/11:
//! *«cada tipo de objeto na hierarchy tenha um ícone único, bem representativo de sua classe e com
//! cores diferentes dos outros tipos»*).
//!
//! O tipo deriva-se por PRESENÇA de um marcador, como o [`ObjectKind`]: os dele, mais os que um
//! COMPONENTE define (o jogador, e num vazio a câmera, a lâmpada, a macro…). A imagem pintada é uma
//! imagem: pintar é a ferramenta dela, não outro tipo (dono 06/11). A precedência é a ordem
//! de [`ObjectType::ALL`]. ⚠️ O ícone é exclusivo de cada tipo, e é isso que deixa o pintor da linha
//! achar a cor pelo ícone ([`ObjectType::of_icon`]) sem um campo a mais em cada linha.
//!
//! Plano e paleta medida: `docs/UI_New_and_Simple/F42_icone_e_cor_por_tipo.md`.

use ph2d_component_desc::ObjectKind;
use ph2d_tokens::ColorToken;

use crate::icons::IconId;

/// **Que tipo de objecto a linha mostra.** Os sete primeiros são os do [`ObjectKind`]; o
/// [`ObjectType::Player`] ganha a qualquer um; os restantes só valem num vazio.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ObjectType {
    Empty,
    Image,
    Vector,
    Flip,
    Skeleton,
    Motion,
    PixelSprite,
    Player,
    Camera,
    Light,
    Macro,
    Particles,
    Sound,
    Hud,
    Joint,
    Body,
    Collider,
    /// Uma montagem de juntas («Car 1», Joint Editor).
    Assembly,
}

impl ObjectType {
    /// Todos, na ordem de PRECEDÊNCIA entre os definidos por componente.
    pub const ALL: [Self; 18] = [
        Self::Empty,
        Self::Image,
        Self::Vector,
        Self::Flip,
        Self::Skeleton,
        Self::Motion,
        Self::PixelSprite,
        Self::Player,
        Self::Camera,
        Self::Light,
        Self::Macro,
        Self::Particles,
        Self::Sound,
        Self::Hud,
        Self::Joint,
        Self::Body,
        Self::Collider,
        Self::Assembly,
    ];

    /// O glifo — um por tipo, nenhum repetido.
    #[must_use]
    pub const fn icon(self) -> IconId {
        match self {
            Self::Empty => IconId::Cube,
            Self::Image => IconId::Sprite,
            Self::Vector => IconId::Vector,
            Self::Flip => IconId::Flip,
            Self::Skeleton => IconId::Bone,
            Self::Motion => IconId::MotionNodes,
            Self::PixelSprite => IconId::PixelSprite,
            Self::Player => IconId::Player,
            Self::Camera => IconId::Camera,
            Self::Light => IconId::Light,
            Self::Macro => IconId::Macro,
            Self::Particles => IconId::Particle,
            Self::Sound => IconId::Audio,
            Self::Hud => IconId::Hud,
            Self::Joint => IconId::Joint,
            Self::Body => IconId::Body,
            Self::Collider => IconId::Collider,
            Self::Assembly => IconId::Link,
        }
    }

    /// A cor do tipo — um token de dado `obj-*` do tema.
    #[must_use]
    pub const fn color(self) -> ColorToken {
        match self {
            Self::Empty => ColorToken::ObjEmpty,
            Self::Image => ColorToken::ObjImage,
            Self::Vector => ColorToken::ObjVector,
            Self::Flip => ColorToken::ObjFlip,
            Self::Skeleton => ColorToken::ObjSkeleton,
            Self::Motion => ColorToken::ObjMotion,
            Self::PixelSprite => ColorToken::ObjPixelSprite,
            Self::Player => ColorToken::ObjPlayer,
            Self::Camera => ColorToken::ObjCamera,
            Self::Light => ColorToken::ObjLight,
            Self::Macro => ColorToken::ObjMacro,
            Self::Particles => ColorToken::ObjParticles,
            Self::Sound => ColorToken::ObjSound,
            Self::Hud => ColorToken::ObjHud,
            Self::Joint => ColorToken::ObjJoint,
            Self::Body => ColorToken::ObjBody,
            Self::Collider => ColorToken::ObjCollider,
            Self::Assembly => ColorToken::ObjAssembly,
        }
    }

    /// O nome canónico do componente que DEFINE o tipo — `None` nos que vêm do [`ObjectKind`].
    #[must_use]
    pub const fn marker(self) -> Option<&'static str> {
        match self {
            Self::Player => Some("ph2d::physics::PlatformPlayer"),
            Self::Camera => Some("ph2d::ecs::GameCamera"),
            Self::Light => Some("ph2d::light2d::Luz"),
            Self::Macro => Some("ph2d::ecs::Macro"),
            Self::Particles => Some("ph2d::ecs::ParticleEmitter"),
            Self::Sound => Some("ph2d::ecs::AudioSource2D"),
            Self::Hud => Some("ph2d::ecs::UiCanvas"),
            Self::Joint => Some("ph2d::physics::PhysicsJoint"),
            Self::Body => Some("ph2d::physics::RigidBody"),
            Self::Collider => Some("ph2d::physics::Collider"),
            Self::Assembly => Some("ph2d::physics::JointAssembly"),
            Self::Empty
            | Self::Image
            | Self::Vector
            | Self::Flip
            | Self::Skeleton
            | Self::Motion
            | Self::PixelSprite => None,
        }
    }

    /// **O tipo de um objecto**: o jogador ganha a todos; depois o [`ObjectKind`]; num vazio, o 1.º
    /// componente definidor que ele tem. `has(nome)` responde pela presença do componente.
    #[must_use]
    pub fn of(kind: ObjectKind, has: impl Fn(&'static str) -> bool) -> Self {
        let by_marker = |t: &Self| t.marker().is_some_and(&has);
        if by_marker(&Self::Player) {
            return Self::Player;
        }
        match kind {
            ObjectKind::Image => Self::Image,
            ObjectKind::Vector => Self::Vector,
            ObjectKind::Flip => Self::Flip,
            ObjectKind::Painted => Self::Image,
            ObjectKind::Skeleton => Self::Skeleton,
            ObjectKind::Motion => Self::Motion,
            ObjectKind::PixelSprite => Self::PixelSprite,
            ObjectKind::Camera => Self::Camera,
            ObjectKind::Empty => Self::ALL.into_iter().find(by_marker).unwrap_or(Self::Empty),
        }
    }

    /// De volta do ícone ao tipo — vale porque nenhum tipo partilha o glifo.
    #[must_use]
    pub fn of_icon(icon: IconId) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.icon() == icon)
    }
}

#[cfg(test)]
#[path = "hier_types_tests.rs"]
mod tests;
