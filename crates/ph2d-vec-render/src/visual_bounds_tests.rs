//! Gates da caixa VISUAL ([`super::path_visual_bounds`]) — o que a caixa de transformação, a ④ e o
//! enquadrar desenham (doc 17 §2).

use super::{path_intrinsic_visual_bbox, path_visual_bounds};
use ph2d_vec_scene::{
    LineJoin, Paint, PaintEntry, Rgba8, StrokeAlign, StrokeSpec, VecPath, VecVertex,
};
use ph2d_vector::Affine;

/// Um rectângulo 100 × 50 centrado na origem, com traço `w` (junta `join`).
fn caixa(w: f64, join: LineJoin, align: StrokeAlign) -> VecPath {
    let mut s = StrokeSpec::new(Rgba8::new(0, 0, 0, 255), w);
    s.join = join;
    s.align = align;
    VecPath {
        verts: [[-50.0, -25.0], [50.0, -25.0], [50.0, 25.0], [-50.0, 25.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(10, 20, 30, 255))),
        stroke: Some(s),
        ..VecPath::default()
    }
}

fn perto(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol
}

/// ⭐ O traço CENTRADO de 10 cresce a caixa 5 de cada lado — exactamente, com a junta MITER (a
/// caixa conservadora dos recortes inflaria `4 × 5` = 20: a «caixa que não abraça»).
#[test]
fn a_centred_stroke_grows_the_box_by_half_its_width_exactly() {
    let r = path_visual_bounds(
        &caixa(10.0, LineJoin::Miter, StrokeAlign::Centre),
        Affine::IDENTITY,
    )
    .expect("caixa");
    for (got, want) in [(r.x0, -55.0), (r.x1, 55.0), (r.y0, -30.0), (r.y1, 30.0)] {
        assert!(perto(got, want, 1e-2), "{r:?}");
    }
}

/// O Outer chega `w` para fora; o Inner fica dentro da forma (a caixa é a geometria).
#[test]
fn outer_reaches_the_whole_width_and_inner_stays_inside() {
    let fora = path_visual_bounds(
        &caixa(10.0, LineJoin::Miter, StrokeAlign::Outer),
        Affine::IDENTITY,
    )
    .expect("caixa");
    assert!(
        perto(fora.x1, 60.0, 1e-2) && perto(fora.y0, -35.0, 1e-2),
        "{fora:?}"
    );
    let dentro = path_visual_bounds(
        &caixa(10.0, LineJoin::Miter, StrokeAlign::Inner),
        Affine::IDENTITY,
    )
    .expect("caixa");
    assert!(
        perto(dentro.x1, 50.0, 1e-9) && perto(dentro.y0, -25.0, 1e-9),
        "{dentro:?}"
    );
}

/// ⭐ O bico do MITER numa ponta de estrela passa a meia largura (a ponta a 36° chega
/// `½w / sin 18°` ≈ 3,24 × ½w); a junta redonda chega SÓ a meia largura.
#[test]
fn a_miter_tip_reaches_past_half_the_width_and_a_round_join_does_not() {
    let mut star = ph2d_vec_scene::star([0.0, 0.0], 100.0, 100.0, 5, 0.382);
    let tip = star
        .verts
        .iter()
        .map(|v| v.anchor[1])
        .fold(f64::MIN, f64::max);
    let mut s = StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 10.0);
    s.join = LineJoin::Round;
    star.stroke = Some(s.clone());
    let redonda = path_visual_bounds(&star, Affine::IDENTITY).expect("caixa");
    s.join = LineJoin::Miter;
    star.stroke = Some(s);
    let bico = path_visual_bounds(&star, Affine::IDENTITY).expect("caixa");
    assert!(
        perto(redonda.y1, tip + 5.0, 1e-2),
        "{} contra {}",
        redonda.y1,
        tip + 5.0
    );
    assert!(
        bico.y1 > tip + 5.0 * 3.0,
        "o bico: {} sobre a ponta {tip}",
        bico.y1
    );
}

/// ⭐ Sob escala desigual a caneta é REDONDA (`√|sx·sy|`): a caixa intrínseca cresce `5·1/2` no x
/// (`sx = 2`) e `5·1/0,5` no y — a desenhada (× escala) fica com 5 de cada lado nos dois eixos.
#[test]
fn under_unequal_scale_the_pen_stays_round() {
    let p = caixa(10.0, LineJoin::Round, StrokeAlign::Centre);
    let (lo, hi) = path_intrinsic_visual_bbox(&p, [2.0, 0.5]).expect("caixa");
    assert!(perto(hi[0], 50.0 + 2.5, 1e-2), "{hi:?}");
    assert!(perto(hi[1], 25.0 + 10.0, 1e-2), "{hi:?}");
    assert!(perto(lo[0], -hi[0], 1e-9) && perto(lo[1], -hi[1], 1e-9));
}

/// A pilha conta: um traço extra de 20 por baixo do de 2 manda; uma camada deslocada leva a caixa.
#[test]
fn the_stack_and_the_offset_layer_count() {
    let mut p = caixa(2.0, LineJoin::Round, StrokeAlign::Centre);
    p.paints = vec![PaintEntry::stroke(StrokeSpec::new(
        Rgba8::new(9, 9, 9, 255),
        20.0,
    ))];
    let r = path_visual_bounds(&p, Affine::IDENTITY).expect("caixa");
    assert!(perto(r.x1, 60.0, 1e-2), "{r:?}");
    p.paints[0].offset = [30.0, 0.0];
    let r = path_visual_bounds(&p, Affine::IDENTITY).expect("caixa");
    assert!(perto(r.x1, 90.0, 1e-2) && perto(r.x0, -51.0, 1e-2), "{r:?}");
}

/// O CONTROLO: sem traço a caixa é a da geometria, ao bit da `bounding_box` da curva.
#[test]
fn without_a_stroke_the_box_is_the_geometry() {
    let mut p = caixa(10.0, LineJoin::Miter, StrokeAlign::Centre);
    p.stroke = None;
    let r = path_visual_bounds(&p, Affine::IDENTITY).expect("caixa");
    assert_eq!((r.x0, r.y0, r.x1, r.y1), (-50.0, -25.0, 50.0, 25.0));
}
