//! **O tracejado alinhado** (`BUGS_vector.md` #43, foto do dono de 15/10, cena 24): um traço Inner
//! ou Outer tracejado guarda os traços e os pontos INTEIROS, deitados de um lado só do caminho.
//!
//! ⚠️ **A régua é a PEÇA, medida na tinta** — área e caixa de cada traço/ponto, nunca a fórmula. O
//! CONTROLO é o mesmo tracejado CENTRADO: a régua tem de ver nele um disco inteiro de diâmetro `w`,
//! senão não vê nada.

use super::*;
use kurbo::Shape as _;
use ph2d_vec_scene::{Contour, DashPattern, FillRule, LineCap, LineJoin, Rgba8, VecVertex};

/// A largura da cena 24 (m).
const W: f64 = 0.05;
/// A área de um ponto INTEIRO: o disco de diâmetro `w`.
const DISCO: f64 = std::f64::consts::PI * W * W / 4.0;

fn v(x: f64, y: f64) -> VecVertex {
    VecVertex::corner([x, y])
}

/// O traço da cena 24: ponta redonda, traço-ponto-ponto (`4 · 1,5`, `0 · 1,5`, `0 · 1,5`).
fn tracejado(align: StrokeAlign, join: LineJoin) -> StrokeSpec {
    StrokeSpec {
        cap: LineCap::Round,
        join,
        align,
        dash: Some(DashPattern {
            pairs: [[4.0, 1.5], [0.0, 1.5], [0.0, 1.5]],
        }),
        ..StrokeSpec::new(Rgba8::new(255, 255, 255, 255), W)
    }
}

/// O rectângulo da cena 24 (lado `1,6`, à volta da origem).
const R: [f64; 4] = [-0.8, -0.8, 0.8, 0.8];

fn rect(s: StrokeSpec) -> VecPath {
    VecPath {
        verts: vec![v(R[0], R[1]), v(R[2], R[1]), v(R[2], R[3]), v(R[0], R[3])],
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(80, 140, 230, 255))),
        stroke: Some(s),
        ..VecPath::default()
    }
}

/// Uma rosquinha (o furo é a pergunta «dentro» no sentido da REGIÃO).
fn rosquinha(s: StrokeSpec) -> VecPath {
    VecPath {
        subpaths: vec![Contour::new_closed(vec![
            v(-0.3, -0.3),
            v(0.3, -0.3),
            v(0.3, 0.3),
            v(-0.3, 0.3),
        ])],
        fill_rule: FillRule::EvenOdd,
        ..rect(s)
    }
}

/// Um círculo de raio `r` por quatro cúbicas — a borda CURVA.
fn circulo(r: f64, s: StrokeSpec) -> VecPath {
    const K: f64 = 0.552_284_749_830_793_4;
    let k = K * r;
    let pts = [(r, 0.0), (0.0, r), (-r, 0.0), (0.0, -r)];
    let tans = [(0.0, k), (-k, 0.0), (0.0, -k), (k, 0.0)];
    let verts = pts
        .iter()
        .zip(tans.iter())
        .map(|(&(x, y), &(tx, ty))| VecVertex {
            in_handle: [x - tx, y - ty],
            out_handle: [x + tx, y + ty],
            ..VecVertex::corner([x, y])
        })
        .collect();
    VecPath {
        verts,
        closed: true,
        stroke: Some(s),
        ..VecPath::default()
    }
}

fn nua(p: &VecPath) -> VecPath {
    VecPath {
        stroke: None,
        ..p.clone()
    }
}

/// Os PONTOS da tinta: as peças com área abaixo de `2w²` (um traço de `4w` mede `≈ 4,8 w²`).
fn pontos(tinta: &[VecPath]) -> Vec<(f64, kurbo::Rect)> {
    tinta
        .iter()
        .map(|p| (crate::area(p), crate::to_bez(p).bounding_box()))
        .filter(|(a, _)| *a < 2.0 * W * W)
        .collect()
}

/// A tinta como o desenho a pinta: alinhada se o motor alinhar, centrada senão.
fn tinta(p: &VecPath) -> Vec<VecPath> {
    aligned_stroke(p).unwrap_or_else(|| outline_stroke(p))
}

/// Cada ponto é um disco INTEIRO de diâmetro `w`: área `π w²/4` e caixa `w × w`, a 3 %.
fn assert_discos(nome: &str, tinta: &[VecPath]) {
    let ps = pontos(tinta);
    assert!(
        ps.len() >= 10,
        "{nome}: a fixture tem de conter pontos (achou {})",
        ps.len()
    );
    for (a, bb) in &ps {
        assert!(
            (a / DISCO - 1.0).abs() < 0.03,
            "{nome}: um ponto mede {:.3}× o disco de diâmetro w (caixa {:.3}w × {:.3}w)",
            a / DISCO,
            bb.width() / W,
            bb.height() / W
        );
        for lado in [bb.width(), bb.height()] {
            assert!(
                (lado / W - 1.0).abs() < 0.03,
                "{nome}: um ponto tem caixa {:.3}w × {:.3}w — não é o disco de diâmetro w",
                bb.width() / W,
                bb.height() / W
            );
        }
    }
}

/// ⭐ CONTROLO: a régua vê um disco inteiro no tracejado CENTRADO (que não passa pelo alinhamento).
#[test]
fn o_controlo_centrado_tem_pontos_inteiros() {
    let p = rect(tracejado(StrokeAlign::Centre, LineJoin::Miter));
    assert!(aligned_stroke(&p).is_none(), "Centre não alinha");
    assert_discos("Centre", &tinta(&p));
}

/// ⭐ **Inner e Outer guardam os pontos INTEIROS** — o defeito da foto: com a banda dupla cortada,
/// cada ponto era MEIO disco de raio `w` (área `2×`, caixa `2w` ao longo do caminho).
#[test]
fn o_tracejado_alinhado_guarda_os_pontos_inteiros() {
    for align in [StrokeAlign::Inner, StrokeAlign::Outer] {
        for join in [LineJoin::Miter, LineJoin::Round, LineJoin::Bevel] {
            let p = rect(tracejado(align, join));
            assert_discos(&format!("{align:?}/{join:?}"), &tinta(&p));
        }
    }
}

/// ⭐ **E deitados de um lado só, ENCOSTADOS ao caminho**: cada ponto do Inner fica dentro do
/// rectângulo com a caixa a tocar a aresta mais próxima; o do Outer, fora e a tocá-la por fora.
#[test]
fn os_pontos_alinhados_encostam_no_caminho_do_lado_certo() {
    for align in [StrokeAlign::Inner, StrokeAlign::Outer] {
        let t = tinta(&rect(tracejado(align, LineJoin::Miter)));
        let ps = pontos(&t);
        assert!(ps.len() >= 10, "{align:?}: a fixture tem de conter pontos");
        for (_, bb) in ps {
            // A distância (com sinal, + = fora) da caixa à aresta mais próxima.
            let (dentro, fora) = match align {
                StrokeAlign::Inner => (
                    (bb.x0 - R[0])
                        .min(R[2] - bb.x1)
                        .min(bb.y0 - R[1])
                        .min(R[3] - bb.y1),
                    0.0,
                ),
                _ => (
                    0.0,
                    (R[0] - bb.x1)
                        .max(bb.x0 - R[2])
                        .max(R[1] - bb.y1)
                        .max(bb.y0 - R[3]),
                ),
            };
            let folga = dentro.max(fora);
            assert!(
                folga.abs() < 0.03 * W,
                "{align:?}: um ponto fica a {:.3}w do caminho (caixa {bb:?})",
                folga / W
            );
        }
    }
}

/// ⭐ **A tinta não atravessa o caminho** — Inner toda dentro, Outer toda fora, com os traços e as
/// pontas inteiros (na borda recta e na CURVA, e no FURO da rosquinha, no sentido da região).
#[test]
fn a_tinta_tracejada_fica_toda_de_um_lado() {
    let formas: [(&str, fn(StrokeSpec) -> VecPath); 3] = [
        ("rect", rect),
        ("rosquinha", rosquinha),
        ("circulo", |s| circulo(0.8, s)),
    ];
    for (nome, forma) in formas {
        for align in [StrokeAlign::Inner, StrokeAlign::Outer] {
            let p = forma(tracejado(align, LineJoin::Miter));
            let t = aligned_stroke(&p).expect("fechado e tracejado alinha");
            let total: f64 = t.iter().map(crate::area).sum();
            let op = match align {
                StrokeAlign::Inner => BoolOp::Subtract,
                _ => BoolOp::Intersect,
            };
            let errado: f64 = t
                .iter()
                .flat_map(|b| apply(b, &nua(&p), op))
                .map(|x| crate::area(&x))
                .sum();
            assert!(
                errado / total < 1e-3,
                "{nome} {align:?}: {:.3}% da tinta atravessou o caminho",
                100.0 * errado / total
            );
            // Os pontos da curva também são discos inteiros (a régua de área não tem eixo).
            for (a, _) in pontos(&t) {
                assert!(
                    (a / DISCO - 1.0).abs() < 0.03,
                    "{nome} {align:?}: um ponto mede {:.3}× o disco",
                    a / DISCO
                );
            }
        }
    }
}

/// ⭐ **O padrão começa onde o do caminho começa, e no mesmo sentido** — na quina inicial
/// `(R0, R1)`, com um TRAÇO a sair para `v1`, nos DOIS sentidos do rectângulo (`+x` e `+y`). O
/// sweep devolve o trilho a partir de um vértice qualquer e no sentido DELE: sem o acerto, o Inner
/// saía noutra fase que o Centre, ou com o traço-ponto-ponto ao contrário.
#[test]
fn o_tracejado_alinhado_comeca_na_quina_do_caminho() {
    for horario in [false, true] {
        for (align, d) in [
            (StrokeAlign::Centre, 0.0),
            (StrokeAlign::Inner, W / 2.0),
            (StrokeAlign::Outer, -W / 2.0),
        ] {
            let mut p = rect(tracejado(align, LineJoin::Miter));
            if horario {
                // v0 fica; o caminho sai para +y.
                p.verts = vec![v(R[0], R[1]), v(R[0], R[3]), v(R[2], R[3]), v(R[2], R[1])];
            }
            let t = tinta(&p);
            // A quina do trilho e a sonda logo à frente dela, sobre o trilho, na direcção de v1.
            let (cx, cy) = (R[0] + d, R[1] + d);
            let (dx, dy) = if horario { (0.0, 1.0) } else { (1.0, 0.0) };
            let sonda = kurbo::Point::new(cx + 2.5 * W * dx, cy + 2.5 * W * dy);
            let nome = format!("{align:?} horario={horario}");
            let peca = t
                .iter()
                .find(|p| crate::to_bez(p).winding(sonda) != 0)
                .unwrap_or_else(|| panic!("{nome}: não há tinta logo depois da quina inicial"));
            assert!(
                crate::area(peca) > 2.0 * W * W,
                "{nome}: depois da quina inicial vem um PONTO — o padrão está noutra fase ou ao contrário"
            );
            let bb = crate::to_bez(peca).bounding_box();
            let inicio = if horario { bb.y0 - cy } else { bb.x0 - cx };
            assert!(
                (inicio + W / 2.0).abs() < 0.1 * W,
                "{nome}: o traço inicial começa a {:.3}w da quina do trilho",
                inicio / W
            );
        }
    }
}

/// ⭐ CONTROLO: o traço SÓLIDO alinhado não mudou — é a banda dupla cortada, ao bit.
#[test]
fn o_solido_alinhado_continua_a_banda_dupla_cortada() {
    for (align, op) in [
        (StrokeAlign::Inner, BoolOp::Intersect),
        (StrokeAlign::Outer, BoolOp::Subtract),
    ] {
        for join in [LineJoin::Miter, LineJoin::Round, LineJoin::Bevel] {
            let mut s = tracejado(align, join);
            s.dash = None;
            let p = rosquinha(s.clone());
            let banda = outline_stroke(&VecPath {
                stroke: Some(StrokeSpec {
                    width: 2.0 * W,
                    ..s.clone()
                }),
                ..p.clone()
            });
            let regiao = VecPath {
                fill: Some(Paint::Solid(s.color())),
                stroke: None,
                ..p.clone()
            };
            let esperado = drop_slivers(banda.iter().flat_map(|b| apply(b, &regiao, op)).collect());
            assert_eq!(aligned_stroke(&p), Some(esperado), "{align:?}/{join:?}");
        }
    }
}

/// Sonda (não gate): peças, pontos e a área de cada um, por alinhamento.
#[test]
#[ignore = "sonda de medição"]
fn sonda_tracejado_alinhado() {
    for align in [StrokeAlign::Centre, StrokeAlign::Inner, StrokeAlign::Outer] {
        let t = tinta(&rect(tracejado(align, LineJoin::Miter)));
        let mut areas: Vec<f64> = t.iter().map(|p| crate::area(p) / (W * W)).collect();
        areas.sort_by(f64::total_cmp);
        let bbs: Vec<String> = t
            .iter()
            .take(4)
            .map(|p| {
                let b = crate::to_bez(p).bounding_box();
                format!("{:.2}w×{:.2}w", b.width() / W, b.height() / W)
            })
            .collect();
        eprintln!(
            "{align:?}: {} peças · área/w² min {:.3} med {:.3} max {:.3} · caixas {bbs:?}",
            t.len(),
            areas[0],
            areas[areas.len() / 2],
            areas[areas.len() - 1]
        );
    }
    for align in [StrokeAlign::Inner, StrokeAlign::Outer] {
        let p = circulo(0.8, tracejado(align, LineJoin::Miter));
        let t = aligned_stroke(&p).expect("alinha");
        eprintln!("circulo {align:?}: {} peças", t.len());
    }
}
