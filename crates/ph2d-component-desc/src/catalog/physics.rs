//! **A família da física** — os 32 componentes de `ph2d-physics-ecs`.
//!
//! ⚠️ **São CONFIG, nunca estado vivo do solver** ([ADR-0131](../../../../docs/architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md)):
//! *"o undo ordena por bytes"*. Isso é o que os torna anexáveis com segurança — anexar um
//! `RigidBody` é declarar uma intenção, não injetar um corpo no meio de um passo do solver.
//!
//! ⚠️ **`applies_to` é `ANY` de propósito, e é a medição que o manda:** a ponte da física vê
//! as entidades por *query* (`BodyQuery = (Entity, &RigidBody, &Collider, &Transform)`), e
//! nada nela pergunta que tipo de objeto é — **uma peça materializada com `RigidBody`
//! funciona sem tocar na ponte** (doc 04 §1.2, item 4). Um caminho vetorial, um objeto Flip
//! e um objeto vazio podem todos ser corpos. Estreitar isto para `IMAGE` seria escrever na
//! tabela uma limitação que o código não tem.
//!
//! # ⚠️ As duas referências por NOME que a F1 migra
//!
//! `PhysicsJoint.body_a/b` e `PulleyWheel.rope/.body` são `stable_name_id` — hash do `Name`.
//! É por isso que hoje *a junta de uma cópia prende os corpos do MESTRE* (a cópia recebe
//! nome `" (1)"`). Quando a F1 os migrar para `StableId`, os campos entram aqui com
//! [`crate::RefKind::Object`], e é essa declaração que faz o remap da F4 acontecer **em toda
//! propagação**, não só na instanciação.

use crate::{ComponentCategory as C, ComponentDesc as D, ObjectKinds as O};

/// Um componente de física autorável. Sempre `Physics`, sempre `ANY` (ver o cabeçalho),
/// ainda sem campos descritos.
const fn p(canonical_name: &'static str, display_name: &'static str) -> D {
    D::authored(canonical_name, display_name, C::Physics, O::ANY, &[])
}

/// Ordenado por `canonical_name` (gate `the_catalog_is_sorted_and_unique`).
pub const DESCS: &[D] = &[
    p("ph2d::physics::AreaBuoyancy", "Buoyancy Zone"),
    p("ph2d::physics::AreaDrag", "Drag Zone"),
    p("ph2d::physics::AreaEffector", "Force Zone"),
    p("ph2d::physics::AreaFalloff", "Zone Falloff"),
    p("ph2d::physics::AreaForceWorldAxes", "Zone World Axes"),
    p("ph2d::physics::AreaFormDrag", "Form Drag Zone"),
    p("ph2d::physics::AreaTorque", "Torque Zone"),
    p("ph2d::physics::Ccd", "Continuous Collision"),
    p("ph2d::physics::Collider", "Collider"),
    p("ph2d::physics::DampingOverride", "Damping"),
    // ⚠️⚠️ `Dominance` e `MassOverride` são `Intrinsic` por uma CERCA, não por falta de
    // desenho — e a cerca está escrita no doc-comment deles (`components/overrides.rs`):
    // *"absent = the neutral default and the Inspector detaches it at 0 (a project file
    // stays free of the no-op)"*. Neles a PRESENÇA é que carrega o sentido, e o valor de
    // anexação teria de vir do CONTEXTO (a massa que o corpo tem agora) — que a paleta
    // genérica não conhece. ⇒ **A porta por-seção deles não é redundante com o `+`**: ela
    // SEMEIA do valor vivo, que é uma coisa que o `+` não pode fazer (ADR-0166).
    D::intrinsic("ph2d::physics::Dominance", "Dominance", C::Physics, &[]),
    p("ph2d::physics::GravityScale", "Gravity Scale"),
    p("ph2d::physics::InitialVelocity", "Initial Velocity"),
    p("ph2d::physics::JointWorldAnchor", "World Anchor"),
    p("ph2d::physics::LockPositionX", "Lock Position X"),
    p("ph2d::physics::LockPositionY", "Lock Position Y"),
    p("ph2d::physics::LockRotation", "Lock Rotation"),
    // Irmã do `Dominance` acima, pela mesma cerca (a massa e a densidade são a mesma
    // grandeza por dois caminhos; ausente = a densidade manda).
    D::intrinsic("ph2d::physics::MassOverride", "Mass", C::Physics, &[]),
    p("ph2d::physics::MaterialCombine", "Material Combine"),
    p("ph2d::physics::NoWallCling", "No Wall Cling"),
    p("ph2d::physics::OneWayPlatform", "One-Way Platform"),
    // ⚠️ `body_a`/`body_b` são `stable_name_id` hoje; F1 migra-os para `StableId` e eles
    // entram aqui como `RefKind::Object`. Sem isso, a junta de uma instância prende os
    // corpos do mestre — o gate `the_instance_joint_binds_the_instances_own_bodies` (F4).
    p("ph2d::physics::PhysicsJoint", "Joint"),
    p("ph2d::physics::PlatformPlayer", "Platform Player"),
    p("ph2d::physics::PlayerMode", "Player Mode"),
    p("ph2d::physics::PlayerSignals", "Player Signals"),
    // ⚠️ `rope`/`body` idem — mesma migração, mesma razão.
    p("ph2d::physics::PulleyWheel", "Pulley Wheel"),
    p("ph2d::physics::RigidBody", "Rigid Body"),
    p("ph2d::physics::RopeStops", "Rope Stops"),
    p("ph2d::physics::SignalOnHit", "Signal on Hit"),
    p("ph2d::physics::SignalOnLeave", "Signal on Leave"),
    p("ph2d::physics::WalkSurface", "Walk Surface"),
    p("ph2d::physics::WestonAxle", "Weston Axle"),
];
