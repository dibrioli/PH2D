//! **O «look» viaja com a geometria refeita** (dono 02/11, auditoria do painel Vector contra a pilha
//! F45): booleana, Offset, faca e pathfinder copiavam só `fill`/`stroke` e as camadas, a ordem, o olho,
//! a opacidade e a mistura do objecto ficavam para trás.

use super::*;
use ph2d_vec_scene::{
    BaseOrder, BlendMode, LineJoin, OffsetSide, Opacity, Paint, PaintEntry, Rgba8, StrokeSpec,
    VecVertex,
};

fn quadrado(x: f64, s: f64) -> VecPath {
    VecPath {
        verts: [[x, 0.0], [x + s, 0.0], [x + s, s], [x, s]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    }
}

/// Um quadrado com tudo o que a pilha guarda.
fn vestido(x: f64) -> VecPath {
    let mut p = quadrado(x, 10.0);
    p.fill = Some(Paint::Solid(Rgba8::new(30, 30, 200, 255)));
    p.stroke_hidden = Some(StrokeSpec::new(Rgba8::new(200, 30, 30, 255), 2.0));
    p.paints = vec![
        PaintEntry::stroke(StrokeSpec::new(Rgba8::new(250, 250, 250, 255), 6.0)),
        PaintEntry::fill(Paint::Solid(Rgba8::new(30, 200, 30, 128))),
    ];
    p.base_order = BaseOrder { fill: 1, stroke: 3 };
    p.opacity = Opacity::new(0.5);
    p.blend = BlendMode::Multiply;
    p
}

fn mesmo_look(out: &[VecPath], src: &VecPath, quem: &str) {
    assert!(!out.is_empty(), "{quem}: nada saiu");
    for (i, r) in out.iter().enumerate() {
        assert_eq!(r.paints, src.paints, "{quem}[{i}]: as camadas");
        assert_eq!(r.base_order, src.base_order, "{quem}[{i}]: a ordem");
        assert_eq!(r.stroke_hidden, src.stroke_hidden, "{quem}[{i}]: o olho");
        assert_eq!(r.opacity, src.opacity, "{quem}[{i}]: a opacidade");
        assert_eq!(r.blend, src.blend, "{quem}[{i}]: a mistura");
        assert_eq!(r.fill, src.fill, "{quem}[{i}]: o preenchimento");
    }
}

#[test]
fn the_offset_carries_the_look() {
    let p = vestido(0.0);
    let out = offset_path(&p, 2.0, LineJoin::Miter, OffsetSide::Both);
    mesmo_look(&out, &p, "offset");
}

#[test]
fn a_boolean_carries_the_look_of_the_top_shape() {
    let (a, b) = (quadrado(-5.0, 10.0), vestido(0.0));
    mesmo_look(&apply(&a, &b, BoolOp::Union), &b, "union");
}

#[test]
fn the_pathfinder_carries_the_look() {
    let (a, b) = (quadrado(-5.0, 10.0), vestido(0.0));
    let out = pathfinder(&[&a, &b], PathfinderOp::Intersect).expect("sweep");
    mesmo_look(&out, &b, "pathfinder");
}

#[test]
fn the_knife_carries_the_look() {
    let p = vestido(0.0);
    let line = VecPath {
        verts: vec![
            VecVertex::corner([5.0, -5.0]),
            VecVertex::corner([5.0, 15.0]),
        ],
        closed: false,
        ..VecPath::default()
    };
    let out = cut_with_line(&p, &line).expect("corta");
    mesmo_look(&out, &p, "faca");
}
