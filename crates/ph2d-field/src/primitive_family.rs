//! ⭐ **DE FORMA PARA FAMÍLIA** — o `match` que dá a cada [`Primitive`] o [`PrimitiveKind`] dela.
//!
//! # Por que ele saiu do irmão
//!
//! O [`super::primitive`] responde por *que números cada forma tem*; aqui responde-se a *qual é a
//! família dela*. A W127 acrescentou a superquadrática e o arquivo passou dos **700** do gate de
//! LOC. ⚠️ **A cura é partir para irmão, nunca uma entrada na allowlist.**
//!
//! ⚠️ E o `match` é **exaustivo de propósito**: é ele que fecha a corrente entre a enumeração das
//! formas e a das famílias — uma forma nova é **erro de compilação** aqui até alguém dizer a que
//! família ela pertence.

use super::PrimitiveKind;
use super::primitive::Primitive;

impl Primitive {
    /// A família desta forma. ⚠️ **O `match` é exaustivo, e é ele que fecha a corrente** — ver
    /// [`PrimitiveKind`].
    #[must_use]
    pub fn kind(&self) -> PrimitiveKind {
        match self {
            Primitive::Box { .. } => PrimitiveKind::Box,
            Primitive::Sphere { .. } => PrimitiveKind::Sphere,
            Primitive::Cylinder { .. } => PrimitiveKind::Cylinder,
            Primitive::Torus { .. } => PrimitiveKind::Torus,
            Primitive::Extrude { .. } => PrimitiveKind::Extrude,
            Primitive::Revolve { .. } => PrimitiveKind::Revolve,
            Primitive::Cone { .. } => PrimitiveKind::Cone,
            Primitive::Capsule { .. } => PrimitiveKind::Capsule,
            Primitive::Prism { .. } => PrimitiveKind::Prism,
            Primitive::Wedge { .. } => PrimitiveKind::Wedge,
            Primitive::TorusArc { .. } => PrimitiveKind::TorusArc,
            Primitive::Star { .. } => PrimitiveKind::Star,
            Primitive::BoxFrame { .. } => PrimitiveKind::BoxFrame,
            Primitive::Ellipsoid { .. } => PrimitiveKind::Ellipsoid,
            Primitive::Octahedron { .. } => PrimitiveKind::Octahedron,
            Primitive::RoundCone { .. } => PrimitiveKind::RoundCone,
            Primitive::CutSphere { .. } => PrimitiveKind::CutSphere,
            Primitive::HollowDome { .. } => PrimitiveKind::HollowDome,
            Primitive::Link { .. } => PrimitiveKind::Link,
            Primitive::SolidAngle { .. } => PrimitiveKind::SolidAngle,
            Primitive::Gear { .. } => PrimitiveKind::Gear,
            Primitive::Cross { .. } => PrimitiveKind::Cross,
            Primitive::Heart { .. } => PrimitiveKind::Heart,
            Primitive::Moon { .. } => PrimitiveKind::Moon,
            Primitive::Drop { .. } => PrimitiveKind::Drop,
            Primitive::Pie { .. } => PrimitiveKind::Pie,
            Primitive::Trapezoid { .. } => PrimitiveKind::Trapezoid,
            Primitive::Vesica { .. } => PrimitiveKind::Vesica,
            Primitive::Arrow { .. } => PrimitiveKind::Arrow,
            Primitive::Chevron { .. } => PrimitiveKind::Chevron,
            Primitive::BentArrow { .. } => PrimitiveKind::BentArrow,
            Primitive::Rhombus { .. } => PrimitiveKind::Rhombus,
            Primitive::Tube { .. } => PrimitiveKind::Tube,
            Primitive::CircleSegment { .. } => PrimitiveKind::CircleSegment,
            Primitive::SpeechRect { .. } => PrimitiveKind::SpeechRect,
            Primitive::SpeechOval { .. } => PrimitiveKind::SpeechOval,
            Primitive::Cloud { .. } => PrimitiveKind::Cloud,
            Primitive::Bolt { .. } => PrimitiveKind::Bolt,
            Primitive::Shield { .. } => PrimitiveKind::Shield,
            Primitive::Tag { .. } => PrimitiveKind::Tag,
            Primitive::Check { .. } => PrimitiveKind::Check,
            Primitive::Banner { .. } => PrimitiveKind::Banner,
            Primitive::Brace { .. } => PrimitiveKind::Brace,
            Primitive::Parallelogram { .. } => PrimitiveKind::Parallelogram,
            Primitive::Delay { .. } => PrimitiveKind::Delay,
            Primitive::Display { .. } => PrimitiveKind::Display,
            Primitive::OffPage { .. } => PrimitiveKind::OffPage,
            Primitive::Spiral { .. } => PrimitiveKind::Spiral,
            Primitive::Document { .. } => PrimitiveKind::Document,
            Primitive::Helix { .. } => PrimitiveKind::Helix,
            Primitive::Gyroid { .. } => PrimitiveKind::Gyroid,
            Primitive::RoundedCylinder { .. } => PrimitiveKind::RoundedCylinder,
            Primitive::Superquadric { .. } => PrimitiveKind::Superquadric,
            Primitive::Superformula { .. } => PrimitiveKind::Superformula,
        }
    }
}
