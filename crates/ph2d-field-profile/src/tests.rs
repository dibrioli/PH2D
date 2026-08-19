//! Os gates da costura.
//!
//! ⚠️ Uma conversão entre dois documentos falha **em silêncio**: o sólido sai *quase* como o
//! desenho, e ninguém sabe dizer qual das duas pontas mentiu. Cada gate aqui afirma uma coisa que,
//! quebrada, produz exatamente esse "quase".

use super::*;
use ph2d_vec_scene::VertexKind;

/// A constante de Bézier que aproxima um quarto de círculo — `4/3·(√2 − 1)`.
const KAPPA: f64 = 0.552_284_749_830_793_4;

fn path_of(verts: Vec<VecVertex>) -> VecPath {
    VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    }
}

fn square(a: f64) -> VecPath {
    path_of(
        [[-a, -a], [a, -a], [a, a], [-a, a]]
            .into_iter()
            .map(VecVertex::corner)
            .collect(),
    )
}

/// Círculo de raio `r` em quatro arcos cúbicos, anti-horário.
fn circle(r: f64) -> VecPath {
    let k = r * KAPPA;
    path_of(vec![
        VecVertex {
            anchor: [r, 0.0],
            in_handle: [r, -k],
            out_handle: [r, k],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        },
        VecVertex {
            anchor: [0.0, r],
            in_handle: [k, r],
            out_handle: [-k, r],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        },
        VecVertex {
            anchor: [-r, 0.0],
            in_handle: [-r, k],
            out_handle: [-r, -k],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        },
        VecVertex {
            anchor: [0.0, -r],
            in_handle: [-k, -r],
            out_handle: [k, -r],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        },
    ])
}

/// ⭐ **Um quadrado coze em QUATRO pontos** — nem cinco, nem dezasseis.
///
/// Cinco seria o primeiro ponto repetido no fim (uma aresta de comprimento zero, que é uma divisão
/// por zero na distância ponto-segmento). Dezasseis seria mandar a reta ao achatador como cúbica
/// degenerada e ele decidir subdividir. Os dois são invisíveis na forma e caríssimos no traçado.
#[test]
fn a_square_cooks_to_exactly_four_points() {
    let p = cook_path(&square(1.0), 1e-3).expect("quadrado é perfil válido");
    assert_eq!(p.contours().len(), 1);
    assert_eq!(
        p.segment_count(),
        4,
        "um quadrado tem quatro arestas; saiu com {} — {:?}",
        p.segment_count(),
        p.contours()[0]
    );
}

/// Distância de um ponto ao segmento `a—b`.
fn point_to_segment(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let e = [b[0] - a[0], b[1] - a[1]];
    let w = [p[0] - a[0], p[1] - a[1]];
    let ee = e[0] * e[0] + e[1] * e[1];
    let t = if ee > 0.0 {
        ((w[0] * e[0] + w[1] * e[1]) / ee).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (w[0] - t * e[0]).hypot(w[1] - t * e[1])
}

/// **O achatamento entrega a tolerância que declara.**
///
/// Não se confia na promessa da biblioteca: mede-se — a distância de cada ponto da **curva** à
/// polilinha que a substitui.
///
/// # ⚠️ O oráculo não é o círculo, e essa distinção custou um vermelho
///
/// A primeira versão deste gate media contra o **círculo verdadeiro** e reprovava a 10⁻⁴ com uma
/// flecha de 1,86·10⁻⁴. Não era o achatador: um círculo feito de quatro cúbicas com `κ = 0,5523`
/// **já é ~2,7·10⁻⁴ diferente do círculo** por construção, e abaixo dessa ordem o erro medido era
/// quase todo da fonte, não da conversão. *Um oráculo que aproxima o que mede deixa de ser oráculo
/// exatamente quando a tolerância desce até ele.*
///
/// O que o achatamento promete — e portanto o que se afirma aqui — é ficar a menos de `ε` **da
/// curva que lhe deram**.
#[test]
fn the_flattening_honours_the_tolerance_it_declares() {
    let r = 1.0_f64;
    // A MESMA curva, montada aqui de forma independente: um gate que pedisse a curva ao código sob
    // teste estaria a comparar a conversão consigo própria.
    let k = r * KAPPA;
    let mut source = BezPath::new();
    source.move_to(Point::new(r, 0.0));
    source.curve_to(Point::new(r, k), Point::new(k, r), Point::new(0.0, r));
    source.curve_to(Point::new(-k, r), Point::new(-r, k), Point::new(-r, 0.0));
    source.curve_to(Point::new(-r, -k), Point::new(-k, -r), Point::new(0.0, -r));
    source.curve_to(Point::new(k, -r), Point::new(r, -k), Point::new(r, 0.0));
    source.close_path();
    let arcs: Vec<kurbo::CubicBez> = source
        .segments()
        .filter_map(|s| match s {
            kurbo::PathSeg::Cubic(c) => Some(c),
            _ => None,
        })
        .collect();
    assert_eq!(arcs.len(), 4, "o círculo de referência tem quatro arcos");

    for tol in [1e-2_f64, 1e-3, 1e-4] {
        let p = cook_path(&circle(r), tol).expect("círculo é perfil válido");
        let c: Vec<[f64; 2]> = p.contours()[0]
            .iter()
            .map(|q| [f64::from(q[0]), f64::from(q[1])])
            .collect();

        let mut worst = 0.0_f64;
        for arc in &arcs {
            for i in 0..=400 {
                let t = f64::from(i) / 400.0;
                let pt = kurbo::ParamCurve::eval(arc, t);
                let d = (0..c.len())
                    .map(|j| point_to_segment([pt.x, pt.y], c[j], c[(j + 1) % c.len()]))
                    .fold(f64::INFINITY, f64::min);
                worst = worst.max(d);
            }
        }
        assert!(
            worst <= tol,
            "tolerância {tol:e}: a curva afasta-se {worst:e} da polilinha, em {} arestas",
            c.len()
        );
        // E não está a subdividir muito mais do que precisa: a conta fechada para um arco de raio
        // `R` é `n ≈ 2,22·√(R/ε)`, e o dobro disso é folga generosa.
        let expected = 2.22 * (r / tol).sqrt();
        assert!(
            (c.len() as f64) < 2.0 * expected,
            "tolerância {tol:e}: {} arestas contra as ~{expected:.0} que a geometria pede",
            c.len()
        );
    }
}

/// ⭐ **O raio vivo de quina do editor vetorial CHEGA ao sólido.**
///
/// É a prova de que o cozimento parte de `cooked()` e não da fonte. Se alguém trocar por
/// `path.verts`, o quadrado volta a ter quatro pontos e a quina arredondada desaparece do sólido —
/// sem erro nenhum, e com o editor a mostrar a quina redonda na tela.
#[test]
fn a_live_corner_radius_reaches_the_profile() {
    let mut p = square(1.0);
    for v in &mut p.verts {
        v.corner_radius = 0.4;
    }
    let cooked = cook_path(&p, 1e-3).expect("quadrado com quina viva");
    assert!(
        cooked.segment_count() > 4,
        "a quina viva tem de virar arco no perfil; saiu com {} pontos",
        cooked.segment_count()
    );
    // E o vértice afiado deixou de existir: nada fica a menos de ~0,1 da quina (1, 1).
    let nearest = cooked.contours()[0]
        .iter()
        .map(|q| (f64::from(q[0]) - 1.0).hypot(f64::from(q[1]) - 1.0))
        .fold(f64::INFINITY, f64::min);
    assert!(
        nearest > 0.1,
        "a quina afiada sobreviveu ao cozimento: há ponto a {nearest:.3} de (1, 1)"
    );
}

/// ⚠️ Um contorno **aberto** é recusado alto.
///
/// Ignorá-lo em silêncio daria um sólido que é quase o desenho — e a diferença apareceria como uma
/// parede que não fechou, três waves depois de alguém desenhar um path aberto sem reparar.
#[test]
fn an_open_contour_is_refused_not_skipped() {
    let mut p = square(1.0);
    p.closed = false;
    assert_eq!(
        cook_path(&p, 1e-3),
        Err(CookError::OpenContour { contour: 0 }),
        "contorno aberto tem de ser recusado"
    );
}

/// ⚠️ **A conversão não espelha o Y.** É um pino, não uma descoberta: se um dia alguém precisar do
/// espelho, ele pertence à ferramenta que escolhe o plano de desenho — e este gate é onde a decisão
/// fica escrita em vez de virar um `-y` perdido numa linha.
#[test]
fn the_cook_does_not_flip_the_y_axis() {
    // Triângulo com a base em y = 0 e o topo em y = 2 — assimétrico de propósito.
    let tri = path_of(
        [[-1.0, 0.0], [1.0, 0.0], [0.0, 2.0]]
            .into_iter()
            .map(VecVertex::corner)
            .collect(),
    );
    let p = cook_path(&tri, 1e-3).expect("triângulo");
    let (min, max) = p.bounds();
    assert!(
        (min[1] - 0.0).abs() < 1e-6 && (max[1] - 2.0).abs() < 1e-6,
        "o topo tem de continuar em +2 e a base em 0; saiu ({}, {})",
        min[1],
        max[1]
    );
}

/// ⭐ **A tolerância automática é uma FRAÇÃO, e é isso que faz a mesma forma sair igual em qualquer
/// unidade.**
///
/// A mesma peça desenhada em milímetros e em metros tem de dar o mesmo número de arestas. Com uma
/// tolerância absoluta, mudar a unidade do documento mudaria a suavidade da forma **e** o custo do
/// traçado — 1000× em cada direção.
#[test]
fn the_automatic_tolerance_follows_the_size_of_the_drawing() {
    let small = cook_path_auto(&circle(0.01)).expect("círculo pequeno");
    let big = cook_path_auto(&circle(100.0)).expect("círculo grande");
    assert_eq!(
        small.segment_count(),
        big.segment_count(),
        "a mesma forma em escalas 10.000× diferentes tem de dar o mesmo nº de arestas: {} vs {}",
        small.segment_count(),
        big.segment_count()
    );
    // E o número está no orçamento medido (~64 arestas por perfil a 640×480).
    assert!(
        (24..=80).contains(&small.segment_count()),
        "um círculo na tolerância automática tem de cair no orçamento medido; deu {}",
        small.segment_count()
    );
}

/// A regra de preenchimento atravessa a costura — um compound path com buraco continua com buraco.
#[test]
fn the_fill_rule_crosses_the_seam() {
    let mut p = square(1.0);
    p.fill_rule = ph2d_vec_scene::FillRule::EvenOdd;
    p.subpaths.push(ph2d_vec_scene::Contour::new_closed(
        [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]]
            .into_iter()
            .map(VecVertex::corner)
            .collect(),
    ));
    let cooked = cook_path(&p, 1e-3).expect("compound");
    assert_eq!(cooked.fill(), ph2d_field::FillRule::EvenOdd);
    assert_eq!(
        cooked.contours().len(),
        2,
        "o buraco é um contorno a mais, e tem de chegar"
    );
}

/// A tolerância viaja **dentro** do perfil — é ela que responde "este perfil está bom?" sem
/// adivinhação, e é ela que um re-cozimento tem de igualar para não mudar a forma em silêncio.
#[test]
fn the_profile_remembers_the_tolerance_it_was_cooked_at() {
    let p = cook_path(&circle(1.0), 5e-4).expect("círculo");
    assert!((f64::from(p.tolerance()) - 5e-4).abs() < 1e-9);
}
