//! **O que a família «mundo» grava, componente a componente** (`docs/Macros/17_W19_plano.md` §2.5). Cada
//! componente registado no produto tem aqui UMA decisão; o censo da shell
//! (`todo_componente_registado_tem_porta_no_mundo`) reprova um registo sem ela — nunca omissão calada.

use ph2d_editor_core::macro_edits::FamiliaDaMacro;
use ph2d_macro::{Door, Recordability};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Porta {
    /// Os bytes bastam noutro projecto (as referências a objectos remapeiam-se ao aplicar).
    Gravado,
    /// Grava-se, menos no objecto Macro: o aplicar dá-lho (a pose, o nome).
    ForaDaRaiz,
    /// No objecto que esta família faz nascer, é dela (a trilha dela já o tem); nos outros, grava-se.
    DaFamilia(FamiliaDaMacro),
    /// Só a trilha desta família o leva (o documento dela vive fora do ECS); noutro objecto, recusa-se.
    SoDaFamilia(FamiliaDaMacro),
    /// Não se grava e não é recusa: quem nasce recebe-o de novo (a ordem entre irmãos, a própria macro).
    DoAplicar,
    /// Não se grava, e o painel di-lo pelo nome.
    Recusado(&'static str),
}

use FamiliaDaMacro::{Motion, Painter};
use Porta::{DaFamilia, DoAplicar, ForaDaRaiz, Gravado, Recusado, SoDaFamilia};

/// Fica no projecto: um dado dele fora do ECS que o blob só nomeia.
const DO_PROJECTO: &str = "a reference to project data outside the object";
/// A ligação a uma receita: a receita fica no projecto de onde a macro saiu.
const PREFAB: &str = "a link to a recipe of the project";
/// Uma referência a outro objecto que o censo `RefKind::Object` ainda não remapeia: aplicada noutro
/// projecto apontaria para o objecto errado.
const SEM_REMAP: &str = "a reference to another object that is not remapped yet";

/// **A tabela**, ordenada pelo nome canónico — a classificação de 24/10 (os 165 do registo do produto).
pub const PORTAS: &[(&str, Porta)] = &[
    ("ph2d::ecs::AnchorMount", Gravado),
    ("ph2d::ecs::AnchorVisibility", Gravado),
    ("ph2d::ecs::AudioListener2D", Gravado),
    ("ph2d::ecs::AudioSource2D", Recusado(DO_PROJECTO)),
    ("ph2d::ecs::BlendMode", Gravado),
    ("ph2d::ecs::CameraFollow", Gravado),
    ("ph2d::ecs::CameraLimits", Gravado),
    ("ph2d::ecs::CameraShake", Gravado),
    ("ph2d::ecs::ClipChildren", Gravado),
    ("ph2d::ecs::Counter", Gravado),
    ("ph2d::ecs::CounterWatch", Gravado),
    ("ph2d::ecs::DestroyOutside", Gravado),
    ("ph2d::ecs::Factory", Recusado(PREFAB)),
    ("ph2d::ecs::FlipObjectRef", Recusado(DO_PROJECTO)),
    ("ph2d::ecs::GameCamera", Gravado),
    ("ph2d::ecs::GroupedChildren", Gravado),
    ("ph2d::ecs::ImageUseMemory", Gravado),
    ("ph2d::ecs::InstanceOf", Recusado(PREFAB)),
    ("ph2d::ecs::Lifetime", Gravado),
    ("ph2d::ecs::LinkedArt", Recusado(PREFAB)),
    ("ph2d::ecs::Locked", Gravado),
    ("ph2d::ecs::Macro", DoAplicar),
    ("ph2d::ecs::Mask2D", Gravado),
    ("ph2d::ecs::MaskInteraction", Gravado),
    ("ph2d::ecs::MasterRoot", Recusado(PREFAB)),
    ("ph2d::ecs::MotionGenerator", SoDaFamilia(Motion)),
    ("ph2d::ecs::MotionInput", Gravado),
    ("ph2d::ecs::Name", ForaDaRaiz),
    ("ph2d::ecs::NamedAnchorList", Gravado),
    ("ph2d::ecs::ObjectInstance", Recusado(PREFAB)),
    ("ph2d::ecs::OnScreenEnabler", Gravado),
    ("ph2d::ecs::OrderInLayer", Gravado),
    ("ph2d::ecs::PaintedDoc", SoDaFamilia(Painter)),
    ("ph2d::ecs::ParticleEmitter", Gravado),
    ("ph2d::ecs::PathFollow", Recusado(DO_PROJECTO)),
    ("ph2d::ecs::RootOrder", DoAplicar),
    ("ph2d::ecs::ScrollFactor", Gravado),
    ("ph2d::ecs::ScrollLimits", Gravado),
    ("ph2d::ecs::ScrollMotion", Gravado),
    ("ph2d::ecs::ScrollRepeat", Gravado),
    ("ph2d::ecs::SequencePlayer", Recusado(DO_PROJECTO)),
    ("ph2d::ecs::ShakeEmitter", Gravado),
    ("ph2d::ecs::ShowBehindParent", Gravado),
    ("ph2d::ecs::SiblingOrder", DoAplicar),
    ("ph2d::ecs::SignalActions", Gravado),
    ("ph2d::ecs::SignalOnAction", Gravado),
    ("ph2d::ecs::SliceNine", Gravado),
    ("ph2d::ecs::SortingGroup", Gravado),
    ("ph2d::ecs::SortingLayer", Gravado),
    ("ph2d::ecs::SpriteAnimCursor", Gravado),
    ("ph2d::ecs::SpriteAnimFrames", Gravado),
    ("ph2d::ecs::SpriteAnimations", Gravado),
    ("ph2d::ecs::SpriteAnimator", Gravado),
    ("ph2d::ecs::SpriteCornerTint", Gravado),
    ("ph2d::ecs::SpriteEmissive", Gravado),
    ("ph2d::ecs::SpriteGrid", Gravado),
    ("ph2d::ecs::SpritePixels", DaFamilia(Painter)),
    ("ph2d::ecs::SpriteRegion", Gravado),
    ("ph2d::ecs::SpriteSheetFrame", Gravado),
    ("ph2d::ecs::SpriteSheetRef", Recusado(DO_PROJECTO)),
    ("ph2d::ecs::StateMachine", Gravado),
    ("ph2d::ecs::Tags", Recusado(DO_PROJECTO)),
    ("ph2d::ecs::TextureFilter", Gravado),
    ("ph2d::ecs::TextureRepeat", Gravado),
    ("ph2d::ecs::TiledDraw", Gravado),
    ("ph2d::ecs::Timers", Gravado),
    ("ph2d::ecs::TopLevel", Gravado),
    ("ph2d::ecs::Transform", ForaDaRaiz),
    ("ph2d::ecs::Tweens", Gravado),
    ("ph2d::ecs::UiButton", Gravado),
    ("ph2d::ecs::UiCanvas", Gravado),
    ("ph2d::ecs::UiLabel", Gravado),
    ("ph2d::ecs::UvTransform", Gravado),
    ("ph2d::ecs::VecAnchors", Gravado),
    ("ph2d::ecs::VecBindings", Gravado),
    ("ph2d::ecs::VecBlend", Recusado(DO_PROJECTO)),
    ("ph2d::ecs::VecBoolGroup", Gravado),
    ("ph2d::ecs::VecBoolOp", Gravado),
    ("ph2d::ecs::VecBucketFill", Recusado(DO_PROJECTO)),
    ("ph2d::ecs::VecClipContent", Gravado),
    ("ph2d::ecs::VecConnector", Recusado(DO_PROJECTO)),
    ("ph2d::ecs::VecContour", Gravado),
    ("ph2d::ecs::VecCutPath", Gravado),
    ("ph2d::ecs::VecEnvelope", Recusado(DO_PROJECTO)),
    ("ph2d::ecs::VecFilter", Gravado),
    ("ph2d::ecs::VecFrame", Gravado),
    ("ph2d::ecs::VecLabel", Recusado(DO_PROJECTO)),
    ("ph2d::ecs::VecLayout", Gravado),
    ("ph2d::ecs::VecLayoutAbsolute", Gravado),
    ("ph2d::ecs::VecLayoutItem", Gravado),
    ("ph2d::ecs::VecLayoutSize", Gravado),
    ("ph2d::ecs::VecMorph", Recusado(DO_PROJECTO)),
    ("ph2d::ecs::VecMorphMachine", Recusado(DO_PROJECTO)),
    ("ph2d::ecs::VecOffset", Gravado),
    ("ph2d::ecs::VecPathRef", Recusado(DO_PROJECTO)),
    ("ph2d::ecs::VecPatternPath", Recusado(DO_PROJECTO)),
    ("ph2d::ecs::VecPatternRotation", Gravado),
    ("ph2d::ecs::VecResizeBox", Gravado),
    ("ph2d::ecs::VecShape", Gravado),
    ("ph2d::ecs::VecStrokeProfile", Gravado),
    ("ph2d::ecs::VecSymmetry", Gravado),
    ("ph2d::ecs::VecTextPath", Recusado(DO_PROJECTO)),
    ("ph2d::ecs::VecWidget", Gravado),
    ("ph2d::ecs::VecWidgetBind", Recusado(DO_PROJECTO)),
    ("ph2d::ecs::VecWidgetIcon", Gravado),
    ("ph2d::ecs::VecWidgetValue", Gravado),
    ("ph2d::ecs::Visibility", Gravado),
    ("ph2d::ecs::VisibilityLayer", Gravado),
    ("ph2d::ecs::WeaponFire", Gravado),
    ("ph2d::ecs::YSort", Gravado),
    ("ph2d::ecs::ZAsRelative", Gravado),
    ("ph2d::ecs::ZIndexOverride", Gravado),
    ("ph2d::light2d::BloqueiaLuz", Gravado),
    ("ph2d::light2d::Luz", Gravado),
    ("ph2d::light2d::NaoRecebeLuz", Gravado),
    ("ph2d::physics::AnimRules", Gravado),
    ("ph2d::physics::AreaBuoyancy", Gravado),
    ("ph2d::physics::AreaDrag", Gravado),
    ("ph2d::physics::AreaEffector", Gravado),
    ("ph2d::physics::AreaFalloff", Gravado),
    ("ph2d::physics::AreaForceWorldAxes", Gravado),
    ("ph2d::physics::AreaFormDrag", Gravado),
    ("ph2d::physics::AreaTorque", Gravado),
    ("ph2d::physics::Ccd", Gravado),
    ("ph2d::physics::CharacterAnimations", Gravado),
    ("ph2d::physics::Collider", Gravado),
    ("ph2d::physics::Damage", Gravado),
    ("ph2d::physics::DampingOverride", Gravado),
    ("ph2d::physics::Dominance", Gravado),
    ("ph2d::physics::GravityScale", Gravado),
    ("ph2d::physics::Health", Gravado),
    ("ph2d::physics::HealthBar", Gravado),
    ("ph2d::physics::InitialVelocity", Gravado),
    ("ph2d::physics::JointWorldAnchor", Gravado),
    ("ph2d::physics::LockPositionX", Gravado),
    ("ph2d::physics::LockPositionY", Gravado),
    ("ph2d::physics::LockRotation", Gravado),
    ("ph2d::physics::MassOverride", Gravado),
    ("ph2d::physics::MaterialCombine", Gravado),
    ("ph2d::physics::NavAgent", Recusado(DO_PROJECTO)),
    ("ph2d::physics::NavCostArea", Gravado),
    ("ph2d::physics::NavLink", Recusado(SEM_REMAP)),
    ("ph2d::physics::NavRegion", Gravado),
    ("ph2d::physics::NoWallCling", Gravado),
    ("ph2d::physics::OneWayPlatform", Gravado),
    ("ph2d::physics::PhysicsJoint", Gravado),
    ("ph2d::physics::PlatformPlayer", Gravado),
    ("ph2d::physics::PlayerMode", Gravado),
    ("ph2d::physics::PlayerSignals", Gravado),
    ("ph2d::physics::ProjectileMotion", Recusado(SEM_REMAP)),
    ("ph2d::physics::PulleyWheel", Gravado),
    ("ph2d::physics::RaySensor", Gravado),
    ("ph2d::physics::RaySignals", Gravado),
    ("ph2d::physics::RigidBody", Gravado),
    ("ph2d::physics::RopeStops", Gravado),
    ("ph2d::physics::SignalOnHit", Gravado),
    ("ph2d::physics::SignalOnLeave", Gravado),
    ("ph2d::physics::SignalTagFilter", Recusado(DO_PROJECTO)),
    ("ph2d::physics::TopDownPlayer", Gravado),
    ("ph2d::physics::WalkSurface", Gravado),
    ("ph2d::physics::WestonAxle", Gravado),
    ("ph2d::render::Sprite", DaFamilia(Painter)),
    ("ph2d::script::LuauScript", Recusado(DO_PROJECTO)),
    ("ph2d::skeleton::Bone", Recusado(SEM_REMAP)),
    ("ph2d::skeleton::BoneLimit", Gravado),
    ("ph2d::skeleton::BoneRest", Gravado),
    ("ph2d::skeleton::IkGoal", Recusado(SEM_REMAP)),
    ("ph2d::skeleton::IkTarget", Gravado),
    ("ph2d::skeleton::Skeleton", Gravado),
    ("ph2d::skeleton::Skin", Recusado(SEM_REMAP)),
    ("ph2d::skeleton::SmartBone", Gravado),
];

/// A decisão do componente `nome`; `None` = sem decisão (o censo reprova).
#[must_use]
pub fn porta(nome: &str) -> Option<Porta> {
    PORTAS.iter().find(|(n, _)| *n == nome).map(|(_, p)| *p)
}

/// As portas na língua do núcleo (`ph2d_macro::Door`): `mundo.<nome>`.
#[must_use]
pub fn como_portas_do_nucleo() -> Vec<Door> {
    PORTAS
        .iter()
        .map(|(n, p)| Door {
            name: n,
            recordability: match p {
                Recusado(why) => Recordability::Refused { why },
                _ => Recordability::Recorded,
            },
        })
        .collect()
}
