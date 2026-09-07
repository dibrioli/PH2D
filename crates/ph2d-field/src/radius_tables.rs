//! ⭐ **A ESCALA de cada forma** — a tabela por-primitiva da **menor** medida que a define.
//!
//! # Por que ela saiu do [`super::radius`]
//!
//! O irmão responde *que raio um nó tem e até onde ele vai* (a promessa central do módulo); este
//! responde *que tamanho a forma tem*. A W106 acrescentou catorze primitivas e o arquivo passou dos
//! **700** do gate de LOC.
//!
//! ⚠️ **Partir para irmão, nunca uma entrada na allowlist.**
//!
//! # ⚠️ E o BORDO saiu daqui em 2026-09-06, porque a pergunta dele é OPOSTA
//!
//! A [`characteristic_size`] procura a **menor** medida (a escala do documento) e a
//! [`super::radius_bounding::bounding_radius`] a **maior** (o bordo do extrator), e esta erra
//! sempre para CIMA de propósito — um bordo maior custa resolução, um bordo menor CORTA a peça e
//! não diz nada. As duas juntas voltaram a passar os **700** quando a W131 trouxe o triângulo, e
//! o corte por responsabilidade estava escrito no doc que elas partilhavam.

use super::apothem_ratio;
use crate::Primitive;

/// **O tamanho característico de uma primitiva** — a menor dimensão que a define.
///
/// É o que dá escala a um raio de mistura: um filete maior do que a peça menor que ele junta
/// engole-a. Não é uma regra de validade (não existe nenhuma), é a escala do documento.
///
/// ⚠️ **Pública porque a mesma pergunta é feita de fora**: quando a árvore vive na cena
/// (`ph2d-field-ecs`), o limite *suave* de uma operação sai da menor peça sob ela — e ele tem de
/// ser calculado por esta função, não por uma segunda cópia. É a mesma regra do [`round_limit`].
#[must_use]
pub fn characteristic_size(p: &Primitive) -> f32 {
    match p {
        Primitive::Box { half, .. } => half[0].min(half[1]).min(half[2]),
        Primitive::Sphere { radius } => *radius,
        Primitive::Cylinder {
            radius,
            half_height,
            ..
        } => radius.min(*half_height),
        Primitive::Torus { minor, .. } => *minor,
        // ⚠️ O polígono responde como a extrusão: o contorno dá as duas do plano, a
        // altura dá a terceira. *Mesma superfície, mesma caixa.*
        Primitive::Extrude {
            profile,
            half_height,
            ..
        }
        | Primitive::Polygon {
            profile,
            half_height,
            ..
        } => {
            let (min, max) = profile.bounds();
            half_height.min((max[0] - min[0]).min(max[1] - min[1]) * 0.5)
        }
        Primitive::Revolve { profile } => {
            let (min, max) = profile.bounds();
            (max[0] - min[0]).min(max[1] - min[1]) * 0.5
        }
        // ⚠️ **O raio MAIOR, não o menor**: num cone fechado o `top` é zero, e a menor dimensão
        // seria zero — um filete de escala zero, num nó cuja peça é perfeitamente visível. *A
        // escala do documento é o tamanho da peça, e uma ponta não é o tamanho dela.*
        Primitive::Cone {
            bottom,
            top,
            half_height,
            ..
        } => bottom.max(*top).min(*half_height),
        Primitive::Capsule {
            radius,
            half_height,
        } => radius.min(*half_height),
        // ⚠️ O apótema, pela razão do [`round_limit`]: é a parede que está mais perto do eixo.
        Primitive::Prism {
            sides,
            bottom,
            top,
            half_height,
            ..
        } => (bottom.max(*top) * apothem_ratio(*sides)).min(*half_height),
        Primitive::Wedge { half, .. } => half[0].min(half[1]).min(half[2]),
        Primitive::TorusArc { minor, .. } => *minor,
        // ⚠️ **O raio do VALE, não o da ponta** — é a menor dimensão que define a estrela, e é
        // aquela contra a qual um filete de junção se mede (um filete maior do que o vale engole o
        // miolo e deixa só as pontas).
        Primitive::Star {
            inner, half_height, ..
        } => inner.min(*half_height),
        // ⚠️ **A ESPESSURA da viga**, e não a caixa: a peça mais fina de uma gaiola é a aresta, e
        // um filete de junção da escala da caixa engoliria a moldura inteira.
        Primitive::BoxFrame { thickness, .. } => *thickness,
        Primitive::Ellipsoid { radii } => radii[0].min(radii[1]).min(radii[2]),
        // ─────────────────────────── W106 ───────────────────────────
        // ⚠️ **A MENOR medida que a peca de facto tem** — e nunca uma que possa ser ZERO num
        // valor legitimo do controlo: uma escala zero daria um filete de juncao invisivel num no
        // perfeitamente visivel (a licao que o cone deixou escrita acima).
        Primitive::Octahedron { radius, .. } => *radius / 3.0_f32.sqrt(),
        // O menor dos dois raios, com o comprimento a limitar: e' a espessura da peca.
        Primitive::RoundCone {
            bottom,
            top,
            half_height,
        } => bottom.max(*top).min(*half_height + bottom.max(*top)),
        Primitive::CutSphere { radius, cut, .. } => (radius - cut).min(*radius),
        Primitive::HollowDome { thickness, .. } => *thickness,
        Primitive::Link { minor, .. } => *minor,
        Primitive::SolidAngle { radius, angle, .. } => radius * angle.sin().abs().max(0.05),
        // ⚠️ **O corpo, nao o dente**: o dente pode ser fino de propósito, e a escala do documento
        // e' o tamanho da peca.
        Primitive::Gear {
            root, half_height, ..
        } => root.min(*half_height),
        Primitive::Cross {
            width, half_height, ..
        } => width.min(*half_height),
        Primitive::Heart {
            size, half_height, ..
        } => size.min(*half_height),
        Primitive::Moon {
            radius,
            bite,
            offset,
            half_height,
            ..
        } => (radius - bite + offset).max(radius * 0.1).min(*half_height),
        Primitive::Drop {
            radius,
            half_height,
            ..
        } => radius.min(*half_height),
        Primitive::Pie {
            radius,
            half_height,
            ..
        } => radius.min(*half_height),
        Primitive::Trapezoid {
            bottom,
            top,
            half_width,
            half_height,
            ..
        } => bottom.max(*top).min(*half_width).min(*half_height),
        Primitive::Vesica {
            radius,
            offset,
            half_height,
            ..
        } => (radius - offset).max(radius * 0.1).min(*half_height),
        // ─────────────────────────── W119 ───────────────────────────
        // ⚠️ **A MENOR medida que define a forma** — é a escala do documento, e é ela que dá sentido
        // a um raio de mistura: a haste de uma seta, a banda de um chevron, a parede de um tubo.
        Primitive::Arrow {
            shaft, half_height, ..
        }
        | Primitive::BentArrow {
            shaft, half_height, ..
        } => shaft.min(*half_height),
        Primitive::Chevron {
            thickness,
            half_height,
            ..
        } => thickness.min(*half_height),
        Primitive::Rhombus {
            half_width,
            half_span,
            half_height,
            ..
        } => half_width.min(*half_span).min(*half_height),
        Primitive::Tube {
            outer,
            inner,
            half_height,
            ..
        } => (outer - inner).min(*half_height),
        Primitive::CircleSegment {
            radius,
            cut,
            half_height,
            ..
        } => (radius - cut).max(radius * 0.1).min(*half_height),
        // ─────────────────────────── W120 ───────────────────────────
        Primitive::SpeechRect {
            half_width,
            half_span,
            half_height,
            ..
        }
        | Primitive::SpeechOval {
            half_width,
            half_span,
            half_height,
            ..
        }
        | Primitive::Shield {
            half_width,
            half_span,
            half_height,
            ..
        }
        | Primitive::Bolt {
            half_width,
            half_span,
            half_height,
            ..
        } => half_width.min(*half_span).min(*half_height),
        Primitive::Cloud {
            half_span,
            half_height,
            ..
        } => (half_span * 0.52).min(*half_height),
        Primitive::Tag {
            half_span,
            point,
            half_height,
            ..
        } => half_span.min(*point).min(*half_height),
        Primitive::Check {
            thickness,
            half_height,
            ..
        }
        | Primitive::Brace {
            thickness,
            half_height,
            ..
        } => thickness.min(*half_height),
        Primitive::Banner {
            half_width,
            half_span,
            notch,
            half_height,
            ..
        } => (half_width - notch)
            .max(half_width * 0.1)
            .min(*half_span)
            .min(*half_height),
        // ─────────────────────────── W122 ───────────────────────────
        Primitive::Parallelogram {
            half_width,
            half_span,
            half_height,
            ..
        }
        | Primitive::Delay {
            half_width,
            half_span,
            half_height,
            ..
        }
        | Primitive::Display {
            half_width,
            half_span,
            half_height,
            ..
        }
        | Primitive::OffPage {
            half_width,
            half_span,
            half_height,
            ..
        } => half_width.min(*half_span).min(*half_height),
        // ─────────────────────────── W123 ───────────────────────────
        Primitive::Spiral {
            thickness,
            half_height,
            ..
        } => thickness.min(*half_height),
        Primitive::Document {
            half_width,
            half_span,
            half_height,
            ..
        } => half_width.min(*half_span).min(*half_height),
        // ─────────────────────────── W124 ───────────────────────────
        Primitive::Helix { thickness, .. } => *thickness,
        Primitive::Gyroid {
            cell, thickness, ..
        } => thickness.min(*cell),
        // ─────────────────────────── W125 ───────────────────────────
        Primitive::RoundedCylinder {
            radius,
            half_height,
            ..
        } => radius.min(*half_height),
        // ─────────────────────────── W127 ───────────────────────────
        // ⚠️ **A CORDA, e não o raio do anel**: é ela a menor medida que define a peça, e é dela
        // que um filete de junção tem de saber.
        Primitive::TorusKnot { cord, .. } => *cord,
        // ⚠️ **A PROFUNDIDADE do filete** — é a menor medida que define a rosca, e é dela que um
        // filete de junção tem de saber.
        Primitive::Thread { depth, .. } => *depth,
        Primitive::Superquadric { half, .. } | Primitive::Superformula { half, .. } => {
            half[0].min(half[1]).min(half[2])
        }
        // ⭐ **O INRAIO** — o maior disco que cabe no triângulo, e o tecto exacto dos dois recuos.
        // ⚠️ Não é uma escolha: um recuo maior come a peça inteira.
        Primitive::Triangle {
            a,
            b,
            c,
            half_height,
            ..
        } => {
            let f = |p: &[f32; 2]| [f64::from(p[0]), f64::from(p[1])];
            #[allow(clippy::cast_possible_truncation)]
            let r = crate::triangle_inradius(f(a), f(b), f(c)) as f32;
            r.min(*half_height)
        }
    }
}
