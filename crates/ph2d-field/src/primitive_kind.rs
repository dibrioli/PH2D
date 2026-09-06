//! ⭐ **A LISTA DAS FAMÍLIAS de forma, sem os números delas** — o [`PrimitiveKind`], que é o que um
//! gate percorre.
//!
//! # Por que um arquivo irmão
//!
//! O [`super::primitive`] responde *que forma, e com que números*; este responde *que formas
//! existem*. A W120 acrescentou nove primitivas e o arquivo passou as `700` linhas do gate de LOC
//! da workspace. ⚠️ **Partir para irmão, nunca uma entrada na allowlist.**

/// ⭐⭐⭐ **A FAMÍLIA de uma primitiva, sem os números dela** (2026-08-26) — a lista que um gate pode
/// percorrer.
///
/// # ⛔ Por que ela nasceu
///
/// O gate `every_primitive_the_engine_can_make_has_a_button` promete, no próprio doc, que *«uma
/// primitiva nova aparece aqui **sozinha**, no dia em que nascer»*. ⚠️ **Não aparecia:** ele
/// percorria uma lista **escrita à mão** (*«uma de cada, construída à mão: é a enumeração que o
/// `Primitive` não oferece»*), e a contagem no fim só defendia a lista **de si mesma**. Um
/// `Primitive` novo compilava, o painel não lhe dava botão, e o gate ficava **verde** — que é
/// exactamente o defeito que a W53 pagou com uma **família de features inteira, completa e
/// invisível** (o `Extrude`/`Revolve` existiam desde a W3 sem nenhum botão a alcançá-los).
///
/// ⭐ **A corrente que fecha o buraco:** um `Primitive` novo é erro de compilação em
/// [`Primitive::kind`] ⇒ obriga uma variante nova aqui ⇒ [`PrimitiveKind::ALL`] é um array de
/// tamanho fixo, e não compila sem ela. *É a mesma corrente do [`crate::UnaryKind`], e ela existia
/// para os modificadores enquanto as formas ficavam com uma lista à mão.*
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrimitiveKind {
    Box,
    Sphere,
    Cylinder,
    Torus,
    Extrude,
    Revolve,
    Cone,
    Capsule,
    Prism,
    Wedge,
    TorusArc,
    Star,
    BoxFrame,
    Ellipsoid,
    Octahedron,
    RoundCone,
    CutSphere,
    HollowDome,
    Link,
    SolidAngle,
    Gear,
    Cross,
    Heart,
    Moon,
    Drop,
    Pie,
    Trapezoid,
    Vesica,
    Arrow,
    Chevron,
    BentArrow,
    Rhombus,
    Tube,
    CircleSegment,
    SpeechRect,
    SpeechOval,
    Cloud,
    Bolt,
    Shield,
    Tag,
    Check,
    Banner,
    Brace,
    // ─────────────────────────── W122 — o fluxograma ───────────────────────────
    Parallelogram,
    Delay,
    Display,
    OffPage,
    // ─────────────────────────── W123 ───────────────────────────
    Spiral,
    Document,
    // ─────────────────────────── W124 ───────────────────────────
    Helix,
    Gyroid,
    // ─────────────────────────── W125 ───────────────────────────
    RoundedCylinder,
}

impl PrimitiveKind {
    /// **A fonte da contagem** — quem quiser saber *«que formas o motor sabe fazer?»* pergunta aqui.
    pub const ALL: [PrimitiveKind; 52] = [
        PrimitiveKind::Box,
        PrimitiveKind::Sphere,
        PrimitiveKind::Cylinder,
        PrimitiveKind::Torus,
        PrimitiveKind::Extrude,
        PrimitiveKind::Revolve,
        PrimitiveKind::Cone,
        PrimitiveKind::Capsule,
        PrimitiveKind::Prism,
        PrimitiveKind::Wedge,
        PrimitiveKind::TorusArc,
        PrimitiveKind::Star,
        PrimitiveKind::BoxFrame,
        PrimitiveKind::Ellipsoid,
        PrimitiveKind::Octahedron,
        PrimitiveKind::RoundCone,
        PrimitiveKind::CutSphere,
        PrimitiveKind::HollowDome,
        PrimitiveKind::Link,
        PrimitiveKind::SolidAngle,
        PrimitiveKind::Gear,
        PrimitiveKind::Cross,
        PrimitiveKind::Heart,
        PrimitiveKind::Moon,
        PrimitiveKind::Drop,
        PrimitiveKind::Pie,
        PrimitiveKind::Trapezoid,
        PrimitiveKind::Vesica,
        PrimitiveKind::Arrow,
        PrimitiveKind::Chevron,
        PrimitiveKind::BentArrow,
        PrimitiveKind::Rhombus,
        PrimitiveKind::Tube,
        PrimitiveKind::CircleSegment,
        PrimitiveKind::SpeechRect,
        PrimitiveKind::SpeechOval,
        PrimitiveKind::Cloud,
        PrimitiveKind::Bolt,
        PrimitiveKind::Shield,
        PrimitiveKind::Tag,
        PrimitiveKind::Check,
        PrimitiveKind::Banner,
        PrimitiveKind::Brace,
        PrimitiveKind::Parallelogram,
        PrimitiveKind::Delay,
        PrimitiveKind::Display,
        PrimitiveKind::OffPage,
        PrimitiveKind::Spiral,
        PrimitiveKind::Document,
        PrimitiveKind::Helix,
        PrimitiveKind::Gyroid,
        PrimitiveKind::RoundedCylinder,
    ];

    /// O sufixo da chave do botão que a cria — `panel.model3d.add.<key>`.
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            PrimitiveKind::Box => "box",
            PrimitiveKind::Sphere => "sphere",
            PrimitiveKind::Cylinder => "cylinder",
            PrimitiveKind::Torus => "torus",
            PrimitiveKind::Extrude => "extrude",
            PrimitiveKind::Revolve => "revolve",
            PrimitiveKind::Cone => "cone",
            PrimitiveKind::Capsule => "capsule",
            PrimitiveKind::Prism => "prism",
            PrimitiveKind::Wedge => "wedge",
            PrimitiveKind::TorusArc => "torus_arc",
            PrimitiveKind::Star => "star",
            PrimitiveKind::BoxFrame => "box_frame",
            PrimitiveKind::Ellipsoid => "ellipsoid",
            PrimitiveKind::Octahedron => "octahedron",
            PrimitiveKind::RoundCone => "round_cone",
            PrimitiveKind::CutSphere => "cut_sphere",
            PrimitiveKind::HollowDome => "hollow_dome",
            PrimitiveKind::Link => "link",
            PrimitiveKind::SolidAngle => "solid_angle",
            PrimitiveKind::Gear => "gear",
            PrimitiveKind::Cross => "cross",
            PrimitiveKind::Heart => "heart",
            PrimitiveKind::Moon => "moon",
            PrimitiveKind::Drop => "drop",
            PrimitiveKind::Pie => "pie",
            PrimitiveKind::Trapezoid => "trapezoid",
            PrimitiveKind::Vesica => "vesica",
            PrimitiveKind::Arrow => "arrow",
            PrimitiveKind::Chevron => "chevron",
            PrimitiveKind::BentArrow => "bent_arrow",
            PrimitiveKind::Rhombus => "rhombus",
            PrimitiveKind::Tube => "tube",
            PrimitiveKind::CircleSegment => "circle_segment",
            PrimitiveKind::SpeechRect => "speech_rect",
            PrimitiveKind::SpeechOval => "speech_oval",
            PrimitiveKind::Cloud => "cloud",
            PrimitiveKind::Bolt => "bolt",
            PrimitiveKind::Shield => "shield",
            PrimitiveKind::Tag => "tag",
            PrimitiveKind::Check => "check",
            PrimitiveKind::Banner => "banner",
            PrimitiveKind::Brace => "brace",
            PrimitiveKind::Parallelogram => "parallelogram",
            PrimitiveKind::Delay => "delay",
            PrimitiveKind::Display => "display",
            PrimitiveKind::OffPage => "off_page",
            PrimitiveKind::Spiral => "spiral",
            PrimitiveKind::Document => "document",
            PrimitiveKind::Helix => "helix",
            PrimitiveKind::Gyroid => "gyroid",
            PrimitiveKind::RoundedCylinder => "rounded_cylinder",
        }
    }
}
