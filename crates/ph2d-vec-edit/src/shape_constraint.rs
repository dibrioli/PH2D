//! As RESTRIÇÕES de teclado do gesto de desenho de forma — módulo irmão de
//! [`crate::shape`] (que fica com a máquina de estados; separados pelo teto de LOC).
//!
//! Duas, as de todo editor vetorial (Illustrator / Figma / Inkscape), e elas compõem.
//! Tudo aqui é função PURA do retângulo do arrasto: é o único lugar onde Shift e Alt
//! viram geometria, então o preview e a forma viva nunca podem divergir.

use ph2d_vec_scene::ShapeKind;

/// Restrições de teclado do gesto de desenho — as duas de todo editor vetorial
/// (Illustrator / Figma / Inkscape), aplicadas ao retângulo do arrasto:
///
/// - **Shift** (`uniform`): trava a proporção **1:1** — quadrado, círculo, polígono
///   não-distorcido. Na `Line`, vira **snap de 45°** (é o que "proporção" significa
///   num segmento: o ângulo, não a caixa).
/// - **Alt** (`from_center`): o ponto pressionado é o **CENTRO**, não a quina — a forma
///   cresce nos dois sentidos.
///
/// Combinam (Alt+Shift = quadrado a partir do centro).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct ShapeConstraint {
    pub uniform: bool,
    pub from_center: bool,
}

/// O retângulo autorado do gesto — de `start` até o cursor `cur` — com as restrições
/// de teclado aplicadas. É a ÚNICA fonte da geometria do gesto (o `build` e o `bounds`
/// passam por aqui), então preview e forma viva nunca divergem.
///
/// - `uniform` (Shift): iguala os dois eixos pelo **maior** deles, preservando o
///   quadrante do arrasto — quadrado / círculo. Numa `Line` não há caixa a igualar, e o
///   que o Shift significa é **ângulo**: o segmento snapa ao múltiplo de 45° mais
///   próximo, mantendo o comprimento.
/// - `from_center` (Alt): `start` vira o CENTRO — o retângulo cresce simetricamente.
#[must_use]
pub(crate) fn constrained_rect(
    start: [f64; 2],
    cur: [f64; 2],
    kind: ShapeKind,
    c: ShapeConstraint,
) -> ([f64; 2], [f64; 2]) {
    let (mut dx, mut dy) = (cur[0] - start[0], cur[1] - start[1]);
    if c.uniform {
        if kind == ShapeKind::Line {
            // Snap de 45°: mesmo comprimento, ângulo no múltiplo mais próximo.
            let len = dx.hypot(dy);
            if len > f64::EPSILON {
                const STEP: f64 = std::f64::consts::FRAC_PI_4;
                let ang = (dy.atan2(dx) / STEP).round() * STEP;
                dx = len * ang.cos();
                dy = len * ang.sin();
            }
        } else {
            let m = dx.abs().max(dy.abs());
            dx = m.copysign(dx);
            dy = m.copysign(dy);
        }
    }
    if c.from_center {
        (
            [start[0] - dx, start[1] - dy],
            [start[0] + dx, start[1] + dy],
        )
    } else {
        (start, [start[0] + dx, start[1] + dy])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHIFT: ShapeConstraint = ShapeConstraint {
        uniform: true,
        from_center: false,
    };
    const ALT: ShapeConstraint = ShapeConstraint {
        uniform: false,
        from_center: true,
    };
    const BOTH: ShapeConstraint = ShapeConstraint {
        uniform: true,
        from_center: true,
    };

    /// **Shift trava 1:1** pelo MAIOR eixo, preservando o quadrante do arrasto: um
    /// arrasto de (10, 3) para a direita-e-baixo vira um quadrado 10×10 para a
    /// direita-e-baixo (não 3×3, e não espelhado).
    #[test]
    fn shift_locks_a_square_by_the_larger_axis_keeping_the_drag_quadrant() {
        let (a, b) = constrained_rect([0.0, 0.0], [10.0, 3.0], ShapeKind::Rectangle, SHIFT);
        assert_eq!(a, [0.0, 0.0]);
        assert_eq!(b, [10.0, 10.0], "1:1 pelo maior eixo");
        // Quadrante preservado: arrastando para cima-e-esquerda o quadrado vai junto.
        let (_, b2) = constrained_rect([0.0, 0.0], [-2.0, -9.0], ShapeKind::Rectangle, SHIFT);
        assert_eq!(b2, [-9.0, -9.0]);
    }

    /// **Alt desenha do CENTRO**: o ponto pressionado vira o meio da forma e ela cresce
    /// para os dois lados — o retângulo autorado fica simétrico em torno dele.
    #[test]
    fn alt_grows_the_shape_from_the_press_point_as_center() {
        let (a, b) = constrained_rect([5.0, 5.0], [8.0, 7.0], ShapeKind::Rectangle, ALT);
        assert_eq!(a, [2.0, 3.0]);
        assert_eq!(b, [8.0, 7.0]);
        // O centro do retângulo é EXATAMENTE o ponto pressionado.
        assert_eq!([(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5], [5.0, 5.0]);
    }

    /// As duas combinam: Alt+Shift = quadrado centrado no clique.
    #[test]
    fn alt_and_shift_compose_into_a_square_from_the_center() {
        let (a, b) = constrained_rect([0.0, 0.0], [10.0, 3.0], ShapeKind::Ellipse, BOTH);
        assert_eq!(a, [-10.0, -10.0]);
        assert_eq!(b, [10.0, 10.0], "círculo centrado no clique");
    }

    /// Numa LINHA não há caixa a igualar — o que o Shift trava é o **ângulo**: o
    /// segmento snapa ao múltiplo de 45° mais próximo, mantendo o comprimento. (Com a
    /// regra da caixa, um arrasto quase-horizontal viraria 45°, que é o bug clássico.)
    #[test]
    fn shift_on_a_line_snaps_the_angle_to_45_degrees_not_the_bbox() {
        // Arrasto quase horizontal (10, 1) → snapa para a horizontal, não para 45°.
        let (a, b) = constrained_rect([0.0, 0.0], [10.0, 1.0], ShapeKind::Line, SHIFT);
        let len = 10.0_f64.hypot(1.0);
        assert_eq!(a, [0.0, 0.0]);
        assert!(
            (b[0] - len).abs() < 1e-9 && b[1].abs() < 1e-9,
            "esperava horizontal de comprimento {len}, veio {b:?}"
        );
        // Arrasto a ~50° → snapa para 45° exatos (componentes iguais).
        let (_, b2) = constrained_rect([0.0, 0.0], [6.0, 7.0], ShapeKind::Line, SHIFT);
        assert!((b2[0] - b2[1]).abs() < 1e-9, "45°: componentes iguais");
    }
}
