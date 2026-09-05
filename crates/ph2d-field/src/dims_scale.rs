//! ⭐ **ESCALAR uma primitiva pelas DIMENSÕES dela** — e não pela pose.
//!
//! # Por que um arquivo irmão
//!
//! O [`crate::dims_write`] responde a *«o que acontece quando alguém escreve UM número»*; este
//! responde a *«o que acontece quando a forma inteira muda de tamanho»*. O arquivo passou as `700`
//! linhas do gate de LOC da workspace quando o **chanfro** entrou (Enio, 2026-08-30), e cada uma das
//! 21 formas com aresta ganhou uma linha a escalá-lo. ⛔ *Split, nunca allowlist.*
//!
//! ⚠️ O `pub use` no [`super`] mantém `ph2d_field::scale_primitive` — cortar um arquivo não pode
//! custar uma reescrita em cada sítio que o chamava.

use crate::Primitive;

/// ⭐ **Escala uma primitiva multiplicando as DIMENSÕES dela**, e não a pose.
///
/// # Por que uma folha não usa `Xform::scale`
///
/// ⚠️ **Uma folha escalada teria DUAS verdades sobre o mesmo tamanho visível**: a largura que o
/// painel mostra e o fator da pose. Uma caixa de 1 de largura escalada 2× mede 2 na tela e continua
/// a dizer «1» — e o artista não tem como saber qual das duas o próximo gesto vai mexer.
///
/// Multiplicar as dimensões dá **exatamente a mesma forma** (a escala uniforme é isso, aplicada ao
/// campo) com **um** número a mudar — o que o painel já mostra.
///
/// ⚠️ Um grupo é o contrário: ele **não tem dimensões próprias**, então o fator da pose é a única
/// resposta, e ali ele não compete com nada.
///
/// Devolve `false` para um fator não-positivo ou não-finito, sem tocar na forma.
pub fn scale_primitive(p: &mut Primitive, factor: f32) -> bool {
    if !factor.is_finite() || factor <= 0.0 {
        return false;
    }
    match p {
        Primitive::Box {
            half,
            round,
            chamfer,
        } => {
            for h in half.iter_mut() {
                *h *= factor;
            }
            *round *= factor;
            // ⭐ **O chanfro escala junto com o filete** — os dois são comprimentos da peça, e um
            // deles fixo faria a aresta mudar de carácter ao redimensionar a forma.
            *chamfer *= factor;
        }
        Primitive::Sphere { radius } => *radius *= factor,
        Primitive::Cylinder {
            radius,
            half_height,
            round,
            chamfer,
        } => {
            *radius *= factor;
            *half_height *= factor;
            *round *= factor;
            // ⭐ **O chanfro escala junto com o filete** — os dois são comprimentos da peça, e um
            // deles fixo faria a aresta mudar de carácter ao redimensionar a forma.
            *chamfer *= factor;
        }
        Primitive::Torus { major, minor } => {
            *major *= factor;
            *minor *= factor;
        }
        Primitive::Extrude {
            half_height,
            round,
            chamfer,
            ..
        } => {
            // ⚠️ O **perfil** não é escalado: ele é o desenho, e o dono dele é o editor vetorial. O
            // que esta escala mexe é a altura da extrusão e o aro — as duas grandezas que este
            // módulo autora. Escalar um perfil aqui seria reescrever, em silêncio, um documento de
            // outro módulo.
            *half_height *= factor;
            *round *= factor;
            // ⭐ **O chanfro escala junto com o filete** — os dois são comprimentos da peça, e um
            // deles fixo faria a aresta mudar de carácter ao redimensionar a forma.
            *chamfer *= factor;
        }
        // Um torno é só o perfil: não há nada aqui que este módulo possua.
        Primitive::Revolve { .. } => return false,
        Primitive::Cone {
            bottom,
            top,
            half_height,
            round,
            chamfer,
        } => {
            *bottom *= factor;
            // ⚠️ **O topo escala como os outros, e o zero fica zero** — é o que mantém um cone
            // fechado fechado ao redimensionar. Uma escala que somasse seria a que o abriria.
            *top *= factor;
            *half_height *= factor;
            *round *= factor;
            // ⭐ **O chanfro escala junto com o filete** — os dois são comprimentos da peça, e um
            // deles fixo faria a aresta mudar de carácter ao redimensionar a forma.
            *chamfer *= factor;
        }
        Primitive::Capsule {
            radius,
            half_height,
        } => {
            *radius *= factor;
            *half_height *= factor;
        }
        Primitive::Prism {
            sides: _,
            bottom,
            top,
            half_height,
            round,
            chamfer,
        } => {
            // ⚠️ **A contagem NÃO escala** — ela não é um comprimento. Multiplicá-la faria um
            // hexágono virar um dodecágono ao aumentar a peça, que é mudar a forma e não o tamanho.
            *bottom *= factor;
            *top *= factor;
            *half_height *= factor;
            *round *= factor;
            // ⭐ **O chanfro escala junto com o filete** — os dois são comprimentos da peça, e um
            // deles fixo faria a aresta mudar de carácter ao redimensionar a forma.
            *chamfer *= factor;
        }
        Primitive::Wedge {
            half,
            round,
            chamfer,
        } => {
            for h in half.iter_mut() {
                *h *= factor;
            }
            *round *= factor;
            // ⭐ **O chanfro escala junto com o filete** — os dois são comprimentos da peça, e um
            // deles fixo faria a aresta mudar de carácter ao redimensionar a forma.
            *chamfer *= factor;
        }
        Primitive::TorusArc {
            major,
            minor,
            angle: _,
            round,
            chamfer,
        } => {
            *round *= factor;
            // ⭐ **O chanfro escala junto com o filete** — os dois são comprimentos da peça, e um
            // deles fixo faria a aresta mudar de carácter ao redimensionar a forma.
            *chamfer *= factor;
            // ⚠️ **O ÂNGULO não escala** — ele é adimensional. Multiplicá-lo faria o arco fechar-se
            // ao aumentar a peça, que é mudar a forma e não o tamanho (a mesma lei da contagem).
            *major *= factor;
            *minor *= factor;
        }
        Primitive::Star {
            points: _,
            outer,
            inner,
            half_height,
            round,
            chamfer,
        } => {
            // ⚠️ **A contagem de pontas NÃO escala** — a lei da contagem de lados: multiplicá-la
            // faria uma estrela de 5 virar uma de 10 ao aumentar a peça, que é mudar a forma.
            *outer *= factor;
            *inner *= factor;
            *half_height *= factor;
            *round *= factor;
            // ⭐ **O chanfro escala junto com o filete** — os dois são comprimentos da peça, e um
            // deles fixo faria a aresta mudar de carácter ao redimensionar a forma.
            *chamfer *= factor;
        }
        Primitive::BoxFrame {
            half,
            thickness,
            round,
            chamfer,
        } => {
            for h in half.iter_mut() {
                *h *= factor;
            }
            // ⚠️ **A espessura escala com o resto**, e é o que mantém a proporção da gaiola: uma
            // moldura ampliada com a viga fina seria outra peça, não a mesma maior.
            *thickness *= factor;
            *round *= factor;
            // ⭐ **O chanfro escala junto com o filete** — os dois são comprimentos da peça, e um
            // deles fixo faria a aresta mudar de carácter ao redimensionar a forma.
            *chamfer *= factor;
        }
        Primitive::Ellipsoid { radii } => {
            for r in radii.iter_mut() {
                *r *= factor;
            }
        }
        // ─────────────────────────── W106 ───────────────────────────
        // ⚠️ **Toda grandeza de COMPRIMENTO escala; contagens e ÂNGULOS não.** Um ângulo
        // multiplicado por um factor abriria a fatia ao ampliar a peça, o que não é ampliar — é
        // outra forma. É a mesma lei que deixa `sides` e `points` em paz.
        Primitive::Octahedron {
            radius,
            round,
            chamfer,
        } => {
            *radius *= factor;
            *round *= factor;
            // ⭐ **O chanfro escala junto com o filete** — os dois são comprimentos da peça, e um
            // deles fixo faria a aresta mudar de carácter ao redimensionar a forma.
            *chamfer *= factor;
        }
        Primitive::RoundCone {
            bottom,
            top,
            half_height,
        } => {
            *bottom *= factor;
            *top *= factor;
            *half_height *= factor;
        }
        Primitive::CutSphere {
            radius,
            cut,
            round,
            chamfer,
        } => {
            *radius *= factor;
            // ⚠️ O corte é uma POSIÇÃO em Z, e escala com a peça — senão ampliar uma cúpula
            // transforma-a numa esfera quase inteira.
            *cut *= factor;
            *round *= factor;
            // ⭐ **O chanfro escala junto com o filete** — os dois são comprimentos da peça, e um
            // deles fixo faria a aresta mudar de carácter ao redimensionar a forma.
            *chamfer *= factor;
        }
        Primitive::HollowDome {
            radius,
            cut,
            thickness,
            round,
            chamfer,
        } => {
            *radius *= factor;
            *cut *= factor;
            *thickness *= factor;
            *round *= factor;
            // ⭐ **O chanfro escala junto com o filete** — os dois são comprimentos da peça, e um
            // deles fixo faria a aresta mudar de carácter ao redimensionar a forma.
            *chamfer *= factor;
        }
        Primitive::Link {
            major,
            minor,
            length,
        } => {
            *major *= factor;
            *minor *= factor;
            *length *= factor;
        }
        Primitive::SolidAngle {
            radius,
            round,
            chamfer,
            ..
        } => {
            *radius *= factor;
            *round *= factor;
            // ⭐ **O chanfro escala junto com o filete** — os dois são comprimentos da peça, e um
            // deles fixo faria a aresta mudar de carácter ao redimensionar a forma.
            *chamfer *= factor;
        }
        // ⚠️ `teeth` e `tooth` ficam: um é contagem, o outro é uma FRAÇÃO do passo.
        Primitive::Gear {
            root,
            outer,
            half_height,
            round,
            chamfer,
            ..
        } => {
            *root *= factor;
            *outer *= factor;
            *half_height *= factor;
            *round *= factor;
            // ⭐ **O chanfro escala junto com o filete** — os dois são comprimentos da peça, e um
            // deles fixo faria a aresta mudar de carácter ao redimensionar a forma.
            *chamfer *= factor;
        }
        Primitive::Cross {
            arm,
            width,
            half_height,
            round,
            chamfer,
        } => {
            *arm *= factor;
            *width *= factor;
            *half_height *= factor;
            *round *= factor;
            // ⭐ **O chanfro escala junto com o filete** — os dois são comprimentos da peça, e um
            // deles fixo faria a aresta mudar de carácter ao redimensionar a forma.
            *chamfer *= factor;
        }
        Primitive::Heart {
            size,
            half_height,
            round,
            chamfer,
        } => {
            *size *= factor;
            *half_height *= factor;
            *round *= factor;
            // ⭐ **O chanfro escala junto com o filete** — os dois são comprimentos da peça, e um
            // deles fixo faria a aresta mudar de carácter ao redimensionar a forma.
            *chamfer *= factor;
        }
        Primitive::Moon {
            radius,
            bite,
            offset,
            half_height,
            round,
            chamfer,
        } => {
            *radius *= factor;
            *bite *= factor;
            *offset *= factor;
            *half_height *= factor;
            *round *= factor;
            // ⭐ **O chanfro escala junto com o filete** — os dois são comprimentos da peça, e um
            // deles fixo faria a aresta mudar de carácter ao redimensionar a forma.
            *chamfer *= factor;
        }
        Primitive::Drop {
            radius,
            height,
            half_height,
            round,
            chamfer,
        } => {
            *radius *= factor;
            *height *= factor;
            *half_height *= factor;
            *round *= factor;
            // ⭐ **O chanfro escala junto com o filete** — os dois são comprimentos da peça, e um
            // deles fixo faria a aresta mudar de carácter ao redimensionar a forma.
            *chamfer *= factor;
        }
        Primitive::Pie {
            radius,
            half_height,
            round,
            chamfer,
            ..
        } => {
            *radius *= factor;
            *half_height *= factor;
            *round *= factor;
            // ⭐ **O chanfro escala junto com o filete** — os dois são comprimentos da peça, e um
            // deles fixo faria a aresta mudar de carácter ao redimensionar a forma.
            *chamfer *= factor;
        }
        Primitive::Trapezoid {
            bottom,
            top,
            half_width,
            half_height,
            round,
            chamfer,
        } => {
            *bottom *= factor;
            *top *= factor;
            *half_width *= factor;
            *half_height *= factor;
            *round *= factor;
            // ⭐ **O chanfro escala junto com o filete** — os dois são comprimentos da peça, e um
            // deles fixo faria a aresta mudar de carácter ao redimensionar a forma.
            *chamfer *= factor;
        }
        Primitive::Vesica {
            radius,
            offset,
            half_height,
            round,
            chamfer,
        } => {
            *radius *= factor;
            *offset *= factor;
            *half_height *= factor;
            *round *= factor;
            // ⭐ **O chanfro escala junto com o filete** — os dois são comprimentos da peça, e um
            // deles fixo faria a aresta mudar de carácter ao redimensionar a forma.
            *chamfer *= factor;
        }
        // ─────────────────────────── W119 ───────────────────────────
        // ⚠️ **A contagem de pontas NÃO escala** (o `heads`), e o **ângulo** do sector também não:
        // os dois são adimensionais, e multiplicá-los mudaria a FORMA em vez do tamanho.
        Primitive::Arrow {
            heads: _,
            half_length,
            shaft,
            head,
            head_length,
            half_height,
            round,
            chamfer,
        } => {
            for v in [
                half_length,
                shaft,
                head,
                head_length,
                half_height,
                round,
                chamfer,
            ] {
                *v *= factor;
            }
        }
        Primitive::Chevron {
            half_length,
            half_span,
            thickness,
            half_height,
            round,
            chamfer,
        } => {
            for v in [
                half_length,
                half_span,
                thickness,
                half_height,
                round,
                chamfer,
            ] {
                *v *= factor;
            }
        }
        Primitive::BentArrow {
            run,
            rise,
            shaft,
            head,
            head_length,
            half_height,
            round,
            chamfer,
        } => {
            for v in [
                run,
                rise,
                shaft,
                head,
                head_length,
                half_height,
                round,
                chamfer,
            ] {
                *v *= factor;
            }
        }
        Primitive::Rhombus {
            half_width,
            half_span,
            half_height,
            round,
            chamfer,
        } => {
            for v in [half_width, half_span, half_height, round, chamfer] {
                *v *= factor;
            }
        }
        Primitive::Tube {
            outer,
            inner,
            angle: _,
            half_height,
            round,
            chamfer,
        } => {
            for v in [outer, inner, half_height, round, chamfer] {
                *v *= factor;
            }
        }
        Primitive::CircleSegment {
            radius,
            cut,
            half_height,
            round,
            chamfer,
        } => {
            for v in [radius, cut, half_height, round, chamfer] {
                *v *= factor;
            }
        }
    }
    true
}
