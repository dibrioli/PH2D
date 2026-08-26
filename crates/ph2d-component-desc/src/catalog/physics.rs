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
//! # ✅ As quatro referências por identidade — DECLARADAS (F1 migrou, F4.2 remapeia)
//!
//! `PhysicsJoint.body_a/b` e `PulleyWheel.rope/.body` **eram** `stable_name_id` — hash do
//! `Name` —, e era por isso que *a junta de uma cópia prendia os corpos do MESTRE* (a cópia
//! recebe nome `" (1)"`, o hash muda, e a referência continuava a nomear o original). A F1
//! migrou-os para [`ph2d_ecs::StableId`], e desde a **F4.2** eles estão declarados aqui com
//! [`crate::RefKind::Object`].
//!
//! ⚠️ **É esta declaração que FAZ o remap acontecer**, e não uma lista de casos escrita à mão
//! do outro lado: a tabela de remapeadores da shell é conferida contra estes campos por um
//! censo de dois lados, então declarar uma referência nova sem quem a reescreva **reprova**.
//! [`ph2d_ecs::StableId`]: https://github.com/dibrioli/PH2D/blob/main/crates/ph2d-ecs/src/stable_id.rs

use crate::{
    ComponentCategory as C, ComponentDesc as D, FieldDesc, FieldKind as K, ObjectKinds as O,
    Propagation, RefKind,
};

/// **Uma referência a outro objeto**, por `StableId` — ver o cabeçalho do módulo.
///
/// ⚠️ `Propagate`: o campo segue o mestre **depois de remapeado**. Não é `RuntimeOwned` — a
/// referência é AUTORIA (*"prende neste corpo"*), e quem o solver possui é a pose, não o elo.
const fn r(field_id: u16, name: &'static str) -> FieldDesc {
    FieldDesc {
        field_id,
        name,
        kind: K::Ref,
        policy: Propagation::Propagate,
        is_ref: Some(RefKind::Object),
    }
}

/// **`PhysicsJoint`** — os dois corpos que ele prende. ⚠️ Os `field_id` seguem a ordem de
/// declaração da struct (`joint.rs`), para que uma wave que descreva o resto do tipo não tenha
/// de saltar por cima destes dois.
const JOINT: &[FieldDesc] = &[r(1, "Body A"), r(2, "Body B")];

/// **`PulleyWheel`** — a CORDA a que ela pertence (`rope`) e o CORPO em que é montada
/// (`body`, `0` = pregada no cenário). Os ids seguem a struct (`components/rope.rs`):
/// `rope` é o 1.º campo e `body` é o 7.º — os do meio ficam por descrever, e o id declarado
/// é precisamente o que torna isso seguro.
const PULLEY_WHEEL: &[FieldDesc] = &[r(1, "Rope"), r(7, "Body")];

/// Um componente de física autorável. Sempre `Physics`, sempre `ANY` (ver o cabeçalho),
/// ainda sem campos descritos.
const fn p(canonical_name: &'static str, display_name: &'static str) -> D {
    D::authored(canonical_name, display_name, C::Physics, O::ANY, &[])
}

/// Irmã do [`p`], para quem **não funciona sem outro componente** — ver [`D::requires`].
///
/// ⚠️ **Duas entradas em toda a família, e as duas são a MESMA query.** A ponte consulta
/// `(Entity, &RigidBody, &Collider, &Transform)`: um corpo sem collider nunca entra no solver, e um
/// player é uma lei que corre sobre um corpo. ⛔ A barra é *inerte sem aquele*, nunca boa prática —
/// as zonas, os joints e os markers ficam de fora de propósito.
const fn pr(
    canonical_name: &'static str,
    display_name: &'static str,
    requires: &'static [&'static str],
) -> D {
    D::authored_requiring(
        canonical_name,
        display_name,
        C::Physics,
        O::ANY,
        &[],
        requires,
    )
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
    // ✅ `body_a`/`body_b` DECLARADOS (F4.2). Sem isto a junta de uma instância prende os
    // corpos do mestre — o gate `the_instance_joint_binds_the_instances_own_bodies`.
    D::authored(
        "ph2d::physics::PhysicsJoint",
        "Joint",
        C::Physics,
        O::ANY,
        JOINT,
    ),
    pr(
        "ph2d::physics::PlatformPlayer",
        "Platform Player",
        &["ph2d::physics::RigidBody"],
    ),
    p("ph2d::physics::PlayerMode", "Player Mode"),
    p("ph2d::physics::PlayerSignals", "Player Signals"),
    // ✅ `rope`/`body` idem — e a roldana é a SEXTA consulta da ponte, a que a refutação não
    // nomeava (F4.1): ela é alcançada pelo nome da corda, então uma referência por remapear
    // faria a roldana da instância disputar a corda do mestre.
    D::authored(
        "ph2d::physics::PulleyWheel",
        "Pulley Wheel",
        C::Physics,
        O::ANY,
        PULLEY_WHEEL,
    ),
    pr(
        "ph2d::physics::RigidBody",
        "Rigid Body",
        &["ph2d::physics::Collider"],
    ),
    p("ph2d::physics::RopeStops", "Rope Stops"),
    p("ph2d::physics::SignalOnHit", "Signal on Hit"),
    p("ph2d::physics::SignalOnLeave", "Signal on Leave"),
    p("ph2d::physics::WalkSurface", "Walk Surface"),
    p("ph2d::physics::WestonAxle", "Weston Axle"),
];
