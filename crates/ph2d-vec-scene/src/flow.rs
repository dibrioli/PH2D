//! **Vocabulário de fluxograma** (ANSI/ISO 5807) — módulo irmão de [`crate::shapes`].
//!
//! É o alfabeto de qualquer app de diagrama: cada símbolo tem um SIGNIFICADO fixo
//! (losango = decisão, pílula = início/fim, paralelogramo = dados, cilindro = base de
//! dados…), e é por isso que eles valem como formas próprias em vez de "um polígono que
//! você deforma": o leitor do diagrama reconhece a silhueta.
//!
//! Todos cabem na caixa do gesto e usam **frações** dela nos parâmetros, para guardarem a
//! proporção ao redimensionar.

use crate::{Contour, VecPath, VecVertex};

/// Contorno fechado de quinas a partir de pontos crus.
fn corners(pts: Vec<[f64; 2]>) -> VecPath {
    VecPath {
        verts: pts.into_iter().map(VecVertex::corner).collect(),
        closed: true,
        ..VecPath::default()
    }
}

/// Centro + semi-eixos da caixa do gesto.
fn box_of(a: [f64; 2], b: [f64; 2]) -> (f64, f64, f64, f64) {
    (
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (b[0] - a[0]).abs() * 0.5,
        (b[1] - a[1]).abs() * 0.5,
    )
}

/// **Decisão** — o losango. Quatro pontos nos meios das bordas.
#[must_use]
pub fn diamond(a: [f64; 2], b: [f64; 2]) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    corners(vec![
        [cx, cy - hh],
        [cx + hw, cy],
        [cx, cy + hh],
        [cx - hw, cy],
    ])
}

/// **Início / fim** — a pílula (stadium): um retângulo cujas pontas são semicírculos.
/// Não é um round-rect com raio grande: o raio é SEMPRE metade do lado menor, então a
/// silhueta é estável em qualquer proporção (é o que a torna reconhecível).
#[must_use]
pub fn pill(a: [f64; 2], b: [f64; 2]) -> VecPath {
    let (_, _, hw, hh) = box_of(a, b);
    crate::rounded_rect(a, b, hw.min(hh))
}

/// **Dados / entrada-saída** — o paralelogramo. `slant` = quanto a base desliza (fração
/// da largura).
#[must_use]
pub fn parallelogram(a: [f64; 2], b: [f64; 2], slant: f64) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let s = 2.0 * hw * slant.clamp(0.0, 0.9);
    corners(vec![
        [cx - hw + s, cy - hh],
        [cx + hw, cy - hh],
        [cx + hw - s, cy + hh],
        [cx - hw, cy + hh],
    ])
}

/// **Trapézio** (operação manual, quando invertido). `slant` = o recuo do topo.
#[must_use]
pub fn trapezoid(a: [f64; 2], b: [f64; 2], slant: f64, flip: bool) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let s = 2.0 * hw * slant.clamp(0.0, 0.49);
    // `flip` troca qual base é a curta — é a diferença entre "entrada manual" e
    // "operação manual" no vocabulário ANSI.
    let (top_in, bot_in) = if flip { (0.0, s) } else { (s, 0.0) };
    corners(vec![
        [cx - hw + top_in, cy - hh],
        [cx + hw - top_in, cy - hh],
        [cx + hw - bot_in, cy + hh],
        [cx - hw + bot_in, cy + hh],
    ])
}

/// **Hexágono achatado** — "preparação" no fluxograma. `cut` = o recuo das pontas.
#[must_use]
pub fn hexagon_flat(a: [f64; 2], b: [f64; 2], cut: f64) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let c = 2.0 * hw * cut.clamp(0.02, 0.49);
    corners(vec![
        [cx - hw + c, cy - hh],
        [cx + hw - c, cy - hh],
        [cx + hw, cy],
        [cx + hw - c, cy + hh],
        [cx - hw + c, cy + hh],
        [cx - hw, cy],
    ])
}

/// **Base de dados** — o cilindro: um retângulo com a elipse do topo e a barriga da base.
/// `lip` = a altura da elipse (fração da altura).
#[must_use]
pub fn cylinder(a: [f64; 2], b: [f64; 2], lip: f64) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let e = hh * lip.clamp(0.05, 0.9); // semi-altura da elipse
    let (y_top, y_bot) = (cy - hh + e, cy + hh - e);
    let k = crate::corners::KAPPA;
    let v = |anchor: [f64; 2], i: [f64; 2], o: [f64; 2]| VecVertex {
        anchor,
        in_handle: i,
        out_handle: o,
        kind: crate::VertexKind::Corner,
    };
    // Contorno: meia-elipse do topo → lateral direita → elipse INTEIRA da base (a
    // barriga) → lateral esquerda. O topo mostra a "tampa" e a base a curvatura.
    corners_with(vec![
        // topo esquerda
        v([cx - hw, y_top], [cx - hw, y_top], [cx - hw, y_top - e * k]),
        v(
            [cx, y_top - e],
            [cx - hw * k, y_top - e],
            [cx + hw * k, y_top - e],
        ),
        v([cx + hw, y_top], [cx + hw, y_top - e * k], [cx + hw, y_top]),
        // lateral direita → base
        v([cx + hw, y_bot], [cx + hw, y_bot], [cx + hw, y_bot + e * k]),
        v(
            [cx, y_bot + e],
            [cx + hw * k, y_bot + e],
            [cx - hw * k, y_bot + e],
        ),
        v([cx - hw, y_bot], [cx - hw, y_bot + e * k], [cx - hw, y_bot]),
    ])
}

/// **Documento** — retângulo com a base ONDULADA (o papel). `wave` = a amplitude da onda
/// (fração da altura).
#[must_use]
pub fn document(a: [f64; 2], b: [f64; 2], wave: f64) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let w = hh * wave.clamp(0.02, 0.5);
    let y_base = cy + hh - w;
    let v = |anchor: [f64; 2], i: [f64; 2], o: [f64; 2]| VecVertex {
        anchor,
        in_handle: i,
        out_handle: o,
        kind: crate::VertexKind::Corner,
    };
    // A onda é uma cúbica que desce e sobe: sai da esquerda, mergulha, e volta pela
    // direita — a silhueta clássica de "folha de papel".
    corners_with(vec![
        v([cx - hw, cy - hh], [cx - hw, cy - hh], [cx - hw, cy - hh]),
        v([cx + hw, cy - hh], [cx + hw, cy - hh], [cx + hw, cy - hh]),
        v(
            [cx + hw, y_base],
            [cx + hw, y_base],
            [cx + hw * 0.45, y_base + 2.0 * w],
        ),
        v(
            [cx - hw, y_base + w],
            [cx - hw * 0.45, y_base - w],
            [cx - hw, y_base + w],
        ),
    ])
}

/// **Delay** — o "D": retângulo com o lado direito em meia-elipse.
#[must_use]
pub fn delay(a: [f64; 2], b: [f64; 2]) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let k = crate::corners::KAPPA;
    let r = hw.min(hh); // o arco é um semicírculo do lado direito
    let x_arc = cx + hw - r;
    let v = |anchor: [f64; 2], i: [f64; 2], o: [f64; 2]| VecVertex {
        anchor,
        in_handle: i,
        out_handle: o,
        kind: crate::VertexKind::Corner,
    };
    corners_with(vec![
        v([cx - hw, cy - hh], [cx - hw, cy - hh], [cx - hw, cy - hh]),
        v([x_arc, cy - hh], [x_arc, cy - hh], [x_arc + r * k, cy - hh]),
        v(
            [cx + hw, cy],
            [cx + hw, cy - hh * k],
            [cx + hw, cy + hh * k],
        ),
        v([x_arc, cy + hh], [x_arc + r * k, cy + hh], [x_arc, cy + hh]),
        v([cx - hw, cy + hh], [cx - hw, cy + hh], [cx - hw, cy + hh]),
    ])
}

/// **Display** — a "tela": lado esquerdo em bico, direito arredondado.
#[must_use]
pub fn display(a: [f64; 2], b: [f64; 2], nose: f64) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let n = 2.0 * hw * nose.clamp(0.02, 0.45);
    let k = crate::corners::KAPPA;
    let v = |anchor: [f64; 2], i: [f64; 2], o: [f64; 2]| VecVertex {
        anchor,
        in_handle: i,
        out_handle: o,
        kind: crate::VertexKind::Corner,
    };
    let x_arc = cx + hw - n;
    corners_with(vec![
        v(
            [cx - hw + n, cy - hh],
            [cx - hw + n, cy - hh],
            [cx - hw + n, cy - hh],
        ),
        v([x_arc, cy - hh], [x_arc, cy - hh], [x_arc + n * k, cy - hh]),
        v(
            [cx + hw, cy],
            [cx + hw, cy - hh * k],
            [cx + hw, cy + hh * k],
        ),
        v([x_arc, cy + hh], [x_arc + n * k, cy + hh], [x_arc, cy + hh]),
        v(
            [cx - hw + n, cy + hh],
            [cx - hw + n, cy + hh],
            [cx - hw + n, cy + hh],
        ),
        v([cx - hw, cy], [cx - hw, cy], [cx - hw, cy]),
    ])
}

/// **Processo predefinido** — retângulo com duas barras verticais (a "sub-rotina").
/// Compound: as barras são o mesmo contorno externo + duas linhas internas seriam furos,
/// então elas entram como CONTORNOS de furo (o que dá o desenho certo em fill e em traço).
#[must_use]
pub fn predefined_process(a: [f64; 2], b: [f64; 2], bar: f64) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let d = 2.0 * hw * bar.clamp(0.02, 0.4);
    let mut p = corners(vec![
        [cx - hw, cy - hh],
        [cx + hw, cy - hh],
        [cx + hw, cy + hh],
        [cx - hw, cy + hh],
    ]);
    for x in [cx - hw + d, cx + hw - d] {
        p.subpaths.push(Contour {
            verts: vec![
                VecVertex::corner([x, cy - hh]),
                VecVertex::corner([x, cy + hh]),
            ],
            closed: false,
        });
    }
    p
}

/// **Conector fora-de-página** — o pentágono apontando para baixo. `tip` = a profundidade
/// do bico (fração da altura).
#[must_use]
pub fn offpage(a: [f64; 2], b: [f64; 2], tip: f64) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let t = 2.0 * hh * tip.clamp(0.05, 0.9);
    corners(vec![
        [cx - hw, cy - hh],
        [cx + hw, cy - hh],
        [cx + hw, cy + hh - t],
        [cx, cy + hh],
        [cx - hw, cy + hh - t],
    ])
}

/// **Junção / somatório** — o círculo com uma cruz (ou um X, girando 45°). Compound: o
/// círculo + as duas linhas.
#[must_use]
pub fn junction(a: [f64; 2], b: [f64; 2]) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let mut p = crate::ellipse([cx, cy], hw, hh);
    p.subpaths.push(Contour {
        verts: vec![
            VecVertex::corner([cx - hw, cy]),
            VecVertex::corner([cx + hw, cy]),
        ],
        closed: false,
    });
    p.subpaths.push(Contour {
        verts: vec![
            VecVertex::corner([cx, cy - hh]),
            VecVertex::corner([cx, cy + hh]),
        ],
        closed: false,
    });
    p
}

/// **Nota / anotação** — o colchete: um "[" que abraça o conteúdo (ABERTO, é um traço).
#[must_use]
pub fn note_bracket(a: [f64; 2], b: [f64; 2], lip: f64) -> VecPath {
    let (cx, cy, hw, hh) = box_of(a, b);
    let l = 2.0 * hw * lip.clamp(0.05, 1.0);
    VecPath {
        verts: vec![
            VecVertex::corner([cx - hw + l, cy - hh]),
            VecVertex::corner([cx - hw, cy - hh]),
            VecVertex::corner([cx - hw, cy + hh]),
            VecVertex::corner([cx - hw + l, cy + hh]),
        ],
        closed: false,
        ..VecPath::default()
    }
}

/// Contorno fechado a partir de vértices já com handles.
fn corners_with(verts: Vec<VecVertex>) -> VecPath {
    VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: [f64; 2] = [-2.0, -1.0];
    const B: [f64; 2] = [2.0, 1.0];

    /// Todas as formas de fluxo cabem na caixa do gesto — se uma extravasar, a bbox
    /// mente e o gizmo desalinha do desenho.
    #[test]
    fn every_flow_symbol_fits_inside_the_gesture_box() {
        let all = [
            ("diamond", diamond(A, B)),
            ("pill", pill(A, B)),
            ("parallelogram", parallelogram(A, B, 0.2)),
            ("trapezoid", trapezoid(A, B, 0.2, false)),
            ("hexagon", hexagon_flat(A, B, 0.2)),
            ("cylinder", cylinder(A, B, 0.2)),
            ("document", document(A, B, 0.15)),
            ("delay", delay(A, B)),
            ("display", display(A, B, 0.2)),
            ("predefined", predefined_process(A, B, 0.12)),
            ("offpage", offpage(A, B, 0.3)),
            ("junction", junction(A, B)),
            ("note", note_bracket(A, B, 0.15)),
        ];
        for (name, p) in all {
            assert!(!p.verts.is_empty(), "{name}: vazio");
            for v in p.verts_all() {
                assert!(
                    v.anchor[0] >= -2.0 - 1e-6
                        && v.anchor[0] <= 2.0 + 1e-6
                        && v.anchor[1] >= -1.0 - 1e-6
                        && v.anchor[1] <= 1.0 + 1e-6,
                    "{name}: ancora {:?} fora da caixa",
                    v.anchor
                );
            }
        }
    }

    /// O losango toca os QUATRO meios das bordas — é o que o torna um losango e não um
    /// quadrado girado (que não caberia na caixa).
    #[test]
    fn the_diamond_touches_the_four_edge_midpoints() {
        let p = diamond(A, B);
        let want = [[0.0, -1.0], [2.0, 0.0], [0.0, 1.0], [-2.0, 0.0]];
        for (v, w) in p.verts.iter().zip(want) {
            assert_eq!(v.anchor, w);
        }
    }

    /// A pílula tem o raio SEMPRE em metade do lado menor — a silhueta não muda com a
    /// proporção (é o que a distingue de um round-rect qualquer).
    #[test]
    fn the_pill_radius_is_always_half_the_short_side() {
        let wide = pill([-4.0, -1.0], [4.0, 1.0]);
        // Com raio = 1 (metade da altura), as âncoras verticais ficam nos extremos.
        let ys: Vec<f64> = wide.verts.iter().map(|v| v.anchor[1]).collect();
        assert!(ys.iter().any(|y| (*y - 1.0).abs() < 1e-9));
        assert!(ys.iter().any(|y| (*y + 1.0).abs() < 1e-9));
    }

    /// O processo predefinido e a junção são COMPOUND: as barras / a cruz são contornos
    /// próprios. Sem isso o símbolo seria um retângulo liso e perderia o significado.
    #[test]
    fn predefined_and_junction_carry_their_inner_strokes() {
        assert_eq!(
            predefined_process(A, B, 0.12).subpaths.len(),
            2,
            "duas barras"
        );
        assert_eq!(junction(A, B).subpaths.len(), 2, "a cruz");
    }
}
