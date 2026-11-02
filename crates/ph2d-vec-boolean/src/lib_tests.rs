use super::*;
use ph2d_vec_scene::{Paint, Rgba8};

/// ⭐ A forma que uma booleana devolve leva a tinta de base ESCONDIDA pelo olho (o olho devolve-a
/// depois) — antes ela sumia e o olho não tinha o que mostrar (auditoria do fecho UI/UX).
#[test]
fn a_compound_keeps_the_hidden_base_paint() {
    let mut style = square(0.0, 0.0, 1.0);
    style.fill_hidden = Some(Paint::solid(Rgba8::new(1, 2, 3, 255)));
    let bp = to_bez(&style);
    let out = compound_from(&[bp], &style).expect("forma");
    assert_eq!(out.fill_hidden, style.fill_hidden);
}

fn square(cx: f64, cy: f64, r: f64) -> VecPath {
    let v = |x: f64, y: f64| VecVertex::corner([x, y]);
    VecPath {
        verts: vec![
            v(cx - r, cy - r),
            v(cx + r, cy - r),
            v(cx + r, cy + r),
            v(cx - r, cy + r),
        ],
        closed: true,
        ..VecPath::default()
    }
}

#[test]
fn union_of_overlapping_squares_is_nonempty() {
    let a = square(0.0, 0.0, 2.0);
    let b = square(2.0, 2.0, 2.0);
    let r = apply(&a, &b, BoolOp::Union);
    assert!(!r.is_empty());
    assert!(r[0].closed);
    assert!(r[0].verts.len() >= 4);
}

#[test]
fn intersect_overlapping_is_nonempty_disjoint_is_empty() {
    let a = square(0.0, 0.0, 2.0);
    let b = square(2.0, 2.0, 2.0);
    assert!(!apply(&a, &b, BoolOp::Intersect).is_empty());
    let far = square(100.0, 100.0, 2.0);
    assert!(apply(&a, &far, BoolOp::Intersect).is_empty());
}

#[test]
fn subtract_overlapping_is_nonempty() {
    let a = square(0.0, 0.0, 3.0);
    let b = square(2.0, 2.0, 2.0);
    assert!(!apply(&a, &b, BoolOp::Subtract).is_empty());
}

#[test]
fn exclude_overlapping_is_nonempty_identical_is_empty() {
    let a = square(0.0, 0.0, 3.0);
    let b = square(2.0, 2.0, 3.0);
    // XOR of two overlapping squares leaves the non-shared regions.
    assert!(!apply(&a, &b, BoolOp::Exclude).is_empty());
    // XOR of a region with itself is empty (everything is shared).
    assert!(apply(&a, &a, BoolOp::Exclude).is_empty());
}

/// O caso que motivou o compound path: um quadrado menor DENTRO de um maior,
/// subtraído, tem de virar UM path com buraco — não dois paths sólidos.
#[test]
fn subtract_inner_from_outer_yields_one_compound_with_a_hole() {
    let outer = square(0.0, 0.0, 4.0);
    let inner = square(0.0, 0.0, 2.0);
    let r = apply(&outer, &inner, BoolOp::Subtract);
    assert_eq!(r.len(), 1, "a rosquinha é UM path, não dois");
    assert!(r[0].is_compound(), "o buraco vive em subpaths");
    assert_eq!(r[0].subpaths.len(), 1);
    assert_eq!(r[0].fill_rule, FillRule::EvenOdd);
    // Buraco de verdade: o centro está FORA da região.
    let mut scene = ph2d_vec_scene::VecScene::new();
    let id = scene.push_path(r[0].clone());
    assert!(
        !scene.path_contains_point(id, [0.0, 0.0]),
        "centro é vazado"
    );
    assert!(scene.path_contains_point(id, [3.0, 0.0]), "anel é sólido");
}

/// Union de 3 discos: o fold N-ário roda em uma chamada, e o estilo vem do topo.
#[test]
fn union_many_folds_and_takes_the_top_style() {
    let a = square(0.0, 0.0, 2.0);
    let mut b = square(2.0, 0.0, 2.0);
    let mut c = square(4.0, 0.0, 2.0);
    b.fill = Some(Paint::solid(Rgba8::new(1, 2, 3, 255)));
    c.fill = Some(Paint::solid(Rgba8::new(9, 8, 7, 255)));
    let r = apply_many(&[&a, &b, &c], BoolOp::Union);
    assert_eq!(r.len(), 1, "os três se tocam → uma região só");
    assert_eq!(r[0].fill, c.fill, "estilo do topo (Illustrator)");
}

/// `Subtract` N-ário preserva o de trás: `a − (b ∪ c)`.
#[test]
fn subtract_many_removes_every_path_above_the_base() {
    let base = square(0.0, 0.0, 6.0);
    let b = square(-4.0, 0.0, 2.5);
    let c = square(4.0, 0.0, 2.5);
    let r = apply_many(&[&base, &b, &c], BoolOp::Subtract);
    assert!(!r.is_empty());
    let mut scene = ph2d_vec_scene::VecScene::new();
    let id = scene.push_path(r[0].clone());
    // Os dois mordidos saíram; o miolo ficou.
    assert!(scene.path_contains_point(id, [0.0, 0.0]));
    assert!(!scene.path_contains_point(id, [-5.0, 0.0]));
    assert!(!scene.path_contains_point(id, [5.0, 0.0]));
}
/// A regressão: 3 quadrados enfileirados unem num RETÂNGULO. O sweep insere um
/// vértice em cada aresta onde os operandos se tocam, e todos caem no meio de
/// uma aresta reta que segue reta — 10 pontos para uma forma de 4.
#[test]
fn union_of_collinear_squares_prunes_down_to_the_rectangle() {
    let r = apply_many(
        &[
            &square(0.0, 0.0, 2.0),
            &square(2.0, 0.0, 2.0),
            &square(4.0, 0.0, 2.0),
        ],
        BoolOp::Union,
    );
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].verts.len(), 4, "um retângulo tem 4 pontos");
    // E é o retângulo certo: [-2,6] × [-2,2].
    let mut scene = ph2d_vec_scene::VecScene::new();
    let id = scene.push_path(r[0].clone());
    let (lo, hi) = scene.path_bbox(id).unwrap();
    assert_eq!((lo, hi), ([-2.0, -2.0], [6.0, 2.0]));
    assert!(scene.path_contains_point(id, [5.0, 0.0]));
    assert!(!scene.path_contains_point(id, [7.0, 0.0]));
}

/// Uma aresta reta tem de voltar reta: handles COLAPSADOS na âncora e vértice
/// `Corner`. Antes, `to_bez` mandava a reta como cúbica degenerada, o sweep a
/// cortava, e as metades voltavam com pontos de controle a 1/3 e 2/3 da aresta —
/// cada quina virava um `Smooth` com dois handles pendurados no meio do nada.
#[test]
fn straight_edges_come_back_as_corners_with_collapsed_handles() {
    let r = apply(
        &square(0.0, 0.0, 2.0),
        &square(2.0, 2.0, 2.0),
        BoolOp::Union,
    );
    let p = &r[0];
    assert_eq!(p.verts.len(), 8, "união diagonal = octógono");
    for v in &p.verts {
        assert_eq!(v.kind, VertexKind::Corner, "quina, não Smooth");
        assert_eq!(v.in_handle, v.anchor, "handle colapsado");
        assert_eq!(v.out_handle, v.anchor);
    }
}

/// Podar não pode comer curva: um vértice com QUALQUER handle não-degenerado é
/// preservado, e a forma continua exata.
#[test]
fn curved_contours_keep_every_control_point() {
    let circle = |cx: f64, cy: f64, rad: f64| ph2d_vec_scene::ellipse([cx, cy], rad, rad);
    let r = apply(
        &circle(0.0, 0.0, 4.0),
        &circle(0.0, 0.0, 2.0),
        BoolOp::Subtract,
    );
    assert_eq!(r.len(), 1);
    let p = &r[0];
    // Anel: 4 pontos por contorno, todos suaves (o círculo-Bézier canônico).
    assert_eq!((p.verts.len(), p.subpaths[0].verts.len()), (4, 4));
    assert!(p.verts_all().all(|v| v.kind == VertexKind::Smooth));
    assert!(
        p.verts_all()
            .all(|v| v.in_handle != v.anchor && v.out_handle != v.anchor)
    );

    let mut scene = ph2d_vec_scene::VecScene::new();
    let id = scene.push_path(p.clone());
    assert!(scene.path_contains_point(id, [3.0, 0.0]), "o anel é sólido");
    assert!(
        !scene.path_contains_point(id, [0.0, 0.0]),
        "o furo é vazado"
    );
    assert!(!scene.path_contains_point(id, [4.5, 0.0]), "fora do disco");
}

/// Mistura: quadrado menos círculo = contorno externo de quinas + furo curvo.
/// Cada contorno é classificado por conta própria.
#[test]
fn a_mixed_result_keeps_corners_and_curves_apart() {
    let circle = ph2d_vec_scene::ellipse([0.0, 0.0], 2.0, 2.0);
    let r = apply(&square(0.0, 0.0, 4.0), &circle, BoolOp::Subtract);
    let p = &r[0];
    assert!(p.verts.iter().all(|v| v.kind == VertexKind::Corner));
    assert_eq!(p.verts.len(), 4);
    assert!(
        p.subpaths[0]
            .verts
            .iter()
            .all(|v| v.kind == VertexKind::Smooth)
    );
}

/// Um bico (a curva volta sobre si) NÃO é colinear-redundante: `dot > 0` exige
/// que as arestas sigam no mesmo sentido, então a ponta sobrevive à poda.
#[test]
fn prune_keeps_a_spike_and_drops_a_zero_length_edge() {
    use ph2d_vec_scene::VecVertex as V;
    // Bico: (0,0) → (4,0) → volta a (2,0) → (2,4). O (4,0) é a ponta.
    let spike = prune_contour(vec![
        V::corner([0.0, 0.0]),
        V::corner([4.0, 0.0]),
        V::corner([2.0, 0.0]),
        V::corner([2.0, 4.0]),
    ]);
    assert_eq!(spike.len(), 4, "a ponta do bico é forma, não ruído");
    // Aresta de comprimento zero + colinear no meio: some.
    let flat = prune_contour(vec![
        V::corner([0.0, 0.0]),
        V::corner([2.0, 0.0]), // colinear
        V::corner([4.0, 0.0]),
        V::corner([4.0, 0.0]), // duplicado
        V::corner([4.0, 4.0]),
        V::corner([0.0, 4.0]),
    ]);
    assert_eq!(flat.len(), 4);
    assert_eq!(flat[0].anchor, [0.0, 0.0]);
    assert_eq!(flat[1].anchor, [4.0, 0.0]);
}
