//! ⭐ **A CAIXA POR EIXO de cada forma** — a irmã por EIXO do [`super::radius_tables`].
//!
//! # Por que um arquivo irmão
//!
//! O irmão responde *que tamanho a forma tem* (o menor) e *que ESFERA a contém* (o maior); este
//! responde *quão longe ela chega em CADA eixo*. A W120 acrescentou nove primitivas e o arquivo
//! passou as `700` linhas do gate de LOC. ⚠️ **Partir para irmão, nunca uma entrada na allowlist.**

use crate::Primitive;
use crate::radius::radius_tables::gear_planar_reach;

/// ⭐⭐⭐ **AS MEIAS-EXTENSÕES da caixa alinhada aos eixos que contém a peça** — a irmã por EIXO do
/// [`bounding_radius`] (Enio, 2026-08-31).
///
/// # ⛔⛔⛔ Por que ela existe: uma esfera não tem lados, e três dívidas medidas vinham daí
///
/// O [`bounding_radius`] devolve **um** número, e quem precisa de *«quão longe a peça chega NAQUELE
/// eixo»* tem de usar esse número em todos os três. Numa chapa alta e fina isso erra por muito, e a
/// conta paga-se em sítios que não se parecem uns com os outros:
///
/// | quem lê | o que ele queria | o que a bola dá | erro |
/// |---|---|---|---|
/// | a **parede da dobra** (`κ·W ≤ 0,9`) | a meia-espessura na direcção em que ela deflecte | o raio, dominado pela **altura** | **`17×`** |
/// | a **faixa da banda** (`Span::Along`) | a extensão no eixo do deformador | o raio | **`15×`** |
/// | o **bordo da dobra** (`bounds::step_mod`) | a meia-altura no eixo dobrado | o raio | **`1,9×`**, e ele entra num `sin` |
///
/// # ⚠️ A ASSIMETRIA é a mesma do [`bounding_radius`], e ela manda aqui
///
/// **Errar para CIMA custa resolução; errar para BAIXO corta a peça e não diz nada.** ⇒ onde a
/// orientação de uma forma no plano não é óbvia, esta tabela usa o **raio planar nos dois eixos**
/// do plano em vez de tentar ser esperta. Continua muito mais apertada do que a esfera, e não pode
/// cortar.
///
/// ⚠️ **O gate é o CAMPO, e não esta fórmula contra a outra** — ver
/// `ph2d_field_eval::the_bounding_half_extents_contain_the_piece`. Comparar duas contas nossas seria
/// cego a uma mutação que mexesse nas duas.
#[must_use]
#[allow(clippy::match_same_arms)]
pub fn bounding_half_extents(p: &Primitive) -> [f32; 3] {
    // Uma forma **plana em Z** (a família esmagadora desta tabela): raio no plano, altura no eixo.
    let chata = |r: f32, h: f32| [r, r, h];
    match p {
        Primitive::Box { half, .. } | Primitive::Wedge { half, .. } => *half,
        Primitive::BoxFrame { half, .. } => *half,
        Primitive::Sphere { radius } => [*radius; 3],
        Primitive::Cylinder {
            radius,
            half_height,
            ..
        } => chata(*radius, *half_height),
        // ⚠️ O toro vive no plano **XY**: a espessura fora dele é só o tubo.
        Primitive::Torus { major, minor } | Primitive::TorusArc { major, minor, .. } => {
            [major + minor, major + minor, *minor]
        }
        // O perfil dá as duas do plano; a extrusão dá a terceira.
        Primitive::Extrude {
            profile,
            half_height,
            ..
        } => {
            let (min, max) = profile.bounds();
            [
                min[0].abs().max(max[0].abs()),
                min[1].abs().max(max[1].abs()),
                *half_height,
            ]
        }
        // ⚠️ O torno gira em torno de **Y**: o `x` do perfil vira o raio (X e Z), o `y` a altura.
        Primitive::Revolve { profile } => {
            let (min, max) = profile.bounds();
            let r = min[0].abs().max(max[0].abs());
            [r, min[1].abs().max(max[1].abs()), r]
        }
        Primitive::Cone {
            bottom,
            top,
            half_height,
            ..
        }
        | Primitive::Prism {
            bottom,
            top,
            half_height,
            ..
        } => chata(bottom.max(*top), *half_height),
        // ⚠️ **A ponta está no EIXO, a `h + r`** — a mesma nota da [`bounding_radius`].
        Primitive::Capsule {
            radius,
            half_height,
        } => chata(*radius, half_height + radius),
        Primitive::RoundCone {
            bottom,
            top,
            half_height,
        } => {
            let r = bottom.max(*top);
            chata(r, half_height + r)
        }
        Primitive::Star {
            outer, half_height, ..
        } => chata(*outer, *half_height),
        // ⛔ Ver [`gear_planar_reach`] — a ponta de um dente é uma corda, e os cantos dela passam
        // do `outer`.
        Primitive::Gear {
            teeth,
            outer,
            half_height,
            ..
        } => chata(gear_planar_reach(*teeth, *outer), *half_height),
        // ⭐ **A única forma cuja caixa é EXACTA por eixo** — e é por isso que ela existe.
        Primitive::Ellipsoid { radii } => *radii,
        Primitive::Octahedron { radius, .. } | Primitive::CutSphere { radius, .. } => [*radius; 3],
        Primitive::SolidAngle { radius, .. } => [*radius; 3],
        Primitive::HollowDome {
            radius, thickness, ..
        } => [radius + thickness * 0.5; 3],
        // ⚠️ O elo é um estádio no plano **XY**, alongado em Y; fora dele é só o tubo.
        Primitive::Link {
            major,
            minor,
            length,
        } => [major + minor, length + major + minor, *minor],
        // ⚠️ **O canto do braço**, e não o meio da ponta — a nota que o report dos arcos pretos
        // deixou na [`bounding_radius`]. Aqui os dois braços dão a mesma extensão.
        Primitive::Cross {
            arm, half_height, ..
        } => chata(*arm, *half_height),
        // ⚠️ **O raio PLANAR nos dois eixos**, de propósito: a extensão exacta do coração em `x` e
        // em `y` é `s·(½ + 1/√2)`, mas as duas dependem da orientação dos lóbulos, e a assimetria
        // desta função manda errar para cima. Ver o doc.
        Primitive::Heart {
            size, half_height, ..
        } => chata(size * std::f32::consts::SQRT_2, *half_height),
        Primitive::Moon {
            radius,
            half_height,
            ..
        }
        | Primitive::Pie {
            radius,
            half_height,
            ..
        }
        | Primitive::Vesica {
            radius,
            half_height,
            ..
        } => chata(*radius, *half_height),
        Primitive::Drop {
            radius,
            height,
            half_height,
            ..
        } => chata(height.max(*radius), *half_height),
        // ⚠️ **As duas do plano são DIFERENTES aqui** — é a única da família plana em que a largura
        // e a profundidade do contorno não são a mesma grandeza.
        Primitive::Trapezoid {
            bottom,
            top,
            half_width,
            half_height,
            ..
        } => [bottom.max(*top), *half_width, *half_height],
        // ─────────────────────────── W119 ───────────────────────────
        Primitive::Arrow {
            half_length,
            head,
            half_height,
            ..
        } => [*half_length, *head, *half_height],
        Primitive::Chevron {
            half_length,
            half_span,
            thickness,
            half_height,
            ..
        } => [*half_length, half_span + thickness, *half_height],
        Primitive::BentArrow {
            run,
            rise,
            shaft,
            head,
            half_height,
            ..
        } => [(run - shaft + head).max(*run), *rise, *half_height],
        Primitive::Rhombus {
            half_width,
            half_span,
            half_height,
            ..
        } => [*half_width, *half_span, *half_height],
        Primitive::Tube {
            outer, half_height, ..
        }
        | Primitive::CircleSegment {
            radius: outer,
            half_height,
            ..
        } => chata(*outer, *half_height),
        // ─────────────────────────── W120 ───────────────────────────
        Primitive::SpeechRect {
            half_width,
            half_span,
            tail,
            half_height,
            ..
        }
        | Primitive::SpeechOval {
            half_width,
            half_span,
            tail,
            half_height,
            ..
        } => [*half_width, half_span + tail, *half_height],
        // ⚠️⚠️ **A união arredondada INCHA para fora das bossas**, e a caixa tem de a conter: o raio
        // da mistura é `0,35 × a menor bossa`, um `union_round` empurra a superfície até
        // `r·(√2 − 1)`, e a menor bossa nunca passa a `half_width` ⇒ o excesso é no máximo
        // `0,145 × half_width`. Medido: com o `Span` a `2,0` a peça lia `0,575` contra `0,500`
        // declarados, e quem lê esta caixa **corta a peça e não diz nada**.
        Primitive::Cloud {
            half_width,
            half_span,
            tail,
            half_height,
            ..
        } => [
            half_width * crate::primitive_limits::CLOUD_BLEND_SWELL,
            half_span.mul_add(
                crate::primitive_limits::CLOUD_BLEND_SWELL - 1.0,
                tail.mul_add(1.6, *half_span),
            ),
            *half_height,
        ],
        Primitive::Bolt {
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
        | Primitive::Tag {
            half_width,
            half_span,
            half_height,
            ..
        }
        | Primitive::Banner {
            half_width,
            half_span,
            half_height,
            ..
        } => [*half_width, *half_span, *half_height],
        Primitive::Check {
            half_width,
            half_span,
            thickness,
            half_height,
            ..
        } => [half_width + thickness, half_span + thickness, *half_height],
        Primitive::Brace {
            half_span,
            thickness,
            half_height,
            ..
        } => [
            half_span.mul_add(1.1, *thickness),
            half_span + thickness,
            *half_height,
        ],
        // ─────────────────────────── W122 ───────────────────────────
        // ⚠️ **A inclinação SAI da caixa em X** — o vértice de cima está em `half_width + skew`.
        Primitive::Parallelogram {
            half_width,
            half_span,
            skew,
            half_height,
            ..
        } => [half_width + skew.abs(), *half_span, *half_height],
        // ⭐ **As três seguintes são INTERSECÇÕES, e uma intersecção arredondada só ENCOLHE** — não
        // há termo de inchaço a acrescentar (a nuvem tem, porque é uma união).
        Primitive::Delay {
            half_width,
            half_span,
            half_height,
            ..
        } => [*half_width, *half_span, *half_height],
        Primitive::Display {
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
        } => [*half_width, *half_span, *half_height],
        // ─────────────────────────── W123 ───────────────────────────
        // ⚠️ **O raio do FIM mais a espessura** — a fita acaba num corte radial.
        Primitive::Spiral {
            radius,
            pitch,
            turns,
            thickness,
            half_height,
            ..
        } => {
            let fora = pitch.mul_add(*turns, *radius) + thickness;
            [fora, fora, *half_height]
        }
        // ⚠️ **A onda SAI da caixa em baixo**, e a caixa é simétrica: `half_span + wave`.
        Primitive::Document {
            half_width,
            half_span,
            wave,
            half_height,
            ..
        } => [*half_width, half_span + wave, *half_height],
        // ─────────────────────────── W124 ───────────────────────────
        // ⚠️ **A altura é `pitch × turns`, e a laje corta lá** — o tubo não passa dela.
        Primitive::Helix {
            radius,
            pitch,
            turns,
            thickness,
            ..
        } => {
            let fora = radius + thickness;
            [fora, fora, pitch * turns * 0.5]
        }
        // ⭐ **A caixa É a peça** — o gyroid é cortado por ela.
        Primitive::Gyroid { half, .. } => *half,
        // ─────────────────────────── W125 ───────────────────────────
        Primitive::RoundedCylinder {
            radius,
            half_height,
            ..
        } => [*radius, *radius, *half_height],
    }
}
