//! O **Expand contra a pilha de tintas** (F45/F47; dono 02/11: *«Expand e suas funções não funcionam
//! corretamente com nosso novo modo de camadas stroke/fill»*). Cada teste é um defeito medido
//! VERMELHO antes da cura: Offset, Outline Stroke e Apply Power Stroke liam só `fill`/`stroke`.

use super::*;
use ph2d_vec_scene::{
    BaseOrder, BlendMode, Opacity, Paint, PaintEntry, Rgba8, StrokeSpec, VecPath, VecVertex,
};

const AZUL: Rgba8 = Rgba8::new(30, 30, 200, 255);
const VERMELHO: Rgba8 = Rgba8::new(200, 30, 30, 255);
const BRANCO: Rgba8 = Rgba8::new(250, 250, 250, 255);
const VERDE: Rgba8 = Rgba8::new(30, 200, 30, 255);

fn square(s: f64) -> VecPath {
    VecPath {
        verts: [[0.0, 0.0], [s, 0.0], [s, s], [0.0, s]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    }
}

/// Quadrado de 10: fill azul de base, traço vermelho de base (2), e uma camada de traço branco largo
/// (6) POR BAIXO do fill — a «etiqueta».
fn etiqueta() -> VecPath {
    let mut p = square(10.0);
    p.fill = Some(Paint::Solid(AZUL));
    p.stroke = Some(StrokeSpec::new(VERMELHO, 2.0));
    p.paints = vec![PaintEntry::stroke(StrokeSpec::new(BRANCO, 6.0))];
    // A camada 0 no fundo, depois o fill de base, depois o traço de base.
    p.base_order = BaseOrder { fill: 1, stroke: 2 };
    p
}

fn bbox(p: &VecPath) -> ([f64; 2], [f64; 2]) {
    let mut lo = [f64::INFINITY; 2];
    let mut hi = [f64::NEG_INFINITY; 2];
    let mut put = |a: [f64; 2]| {
        lo = [lo[0].min(a[0]), lo[1].min(a[1])];
        hi = [hi[0].max(a[0]), hi[1].max(a[1])];
    };
    p.verts.iter().for_each(|v| put(v.anchor));
    p.subpaths
        .iter()
        .flat_map(|s| &s.verts)
        .for_each(|v| put(v.anchor));
    (lo, hi)
}

fn solid(p: &VecPath) -> Option<Rgba8> {
    match &p.fill {
        Some(Paint::Solid(c)) => Some(*c),
        _ => None,
    }
}

fn no_stroke_left(out: &[VecPath]) {
    for p in out {
        assert!(p.stroke.is_none(), "traço de base sobrou: {p:?}");
        assert!(
            p.paints
                .iter()
                .all(|e| !(e.is_active() && e.width().is_some())),
            "camada de traço activa sobrou: {:?}",
            p.paints
        );
    }
}

/// **Apply Offset leva a forma inteira**: as camadas, a ordem delas e a opacidade/mistura do objecto.
#[test]
fn the_offset_keeps_every_paint_layer_and_the_object_look() {
    let mut p = etiqueta();
    p.opacity = Opacity::new(0.5);
    p.blend = BlendMode::Multiply;
    let out = expand_layers(
        &p,
        Expand::Offset {
            join: LineJoin::Miter,
            side: ph2d_vec_scene::OffsetSide::Both,
        },
        2.0,
    );
    assert_eq!(out.len(), 1);
    let r = &out[0];
    assert_eq!(r.paints, p.paints, "as camadas foram-se no offset");
    assert_eq!(r.base_order, p.base_order, "a ordem da pilha foi-se");
    assert_eq!(r.opacity, p.opacity, "a opacidade do objecto foi-se");
    assert_eq!(r.blend, p.blend, "a mistura do objecto foi-se");
    assert_eq!(r.fill, p.fill);
    assert_eq!(r.stroke, p.stroke);
}

/// **Outline Stroke converte CADA traço que se vê, na ordem da pilha**: o branco por baixo do miolo,
/// o vermelho por cima — três objectos de baixo para cima, nenhum traço sobra.
#[test]
fn the_outline_converts_every_visible_stroke_in_stack_order() {
    let out = expand_layers(&etiqueta(), Expand::OutlineStroke, 0.0);
    no_stroke_left(&out);
    let cores: Vec<_> = out.iter().map(solid).collect();
    assert_eq!(
        cores,
        vec![Some(BRANCO), Some(AZUL), Some(VERMELHO)],
        "a ordem de baixo para cima"
    );
    // O branco de 6 sai 3 para lá do quadrado; o vermelho de 2, 1.
    let (lo, hi) = bbox(&out[0]);
    assert!(
        (lo[0] + 3.0).abs() < 0.05 && (hi[0] - 13.0).abs() < 0.05,
        "{lo:?} {hi:?}"
    );
    let (lo, _) = bbox(&out[2]);
    assert!((lo[0] + 1.0).abs() < 0.05, "{lo:?}");
}

/// Uma forma cujo ÚNICO traço é uma camada (a base sem traço): antes o comando não fazia nada.
#[test]
fn a_stroke_that_lives_only_in_a_layer_is_outlined() {
    let mut p = square(10.0);
    p.fill = Some(Paint::Solid(AZUL));
    p.paints = vec![PaintEntry::stroke(StrokeSpec::new(VERDE, 2.0))];
    let out = expand_layers(&p, Expand::OutlineStroke, 0.0);
    no_stroke_left(&out);
    assert_eq!(
        out.iter().map(solid).collect::<Vec<_>>(),
        vec![Some(AZUL), Some(VERDE)]
    );
}

/// O olho manda: o traço de base ESCONDIDO não vira forma (e não se perde — viaja escondido no
/// miolo, como o olho o guardou); uma camada desligada também não.
#[test]
fn a_hidden_or_disabled_stroke_is_not_outlined() {
    let mut p = square(10.0);
    p.fill = Some(Paint::Solid(AZUL));
    p.stroke_hidden = Some(StrokeSpec::new(VERMELHO, 2.0));
    let mut off = PaintEntry::stroke(StrokeSpec::new(BRANCO, 4.0));
    off.enabled = false;
    p.paints = vec![off, PaintEntry::stroke(StrokeSpec::new(VERDE, 2.0))];
    let out = expand_layers(&p, Expand::OutlineStroke, 0.0);
    assert_eq!(
        out.iter().map(solid).collect::<Vec<_>>(),
        vec![Some(AZUL), Some(VERDE)],
        "só o verde, que se vê, vira forma"
    );
    assert_eq!(
        out[0].stroke_hidden, p.stroke_hidden,
        "o escondido viaja no miolo"
    );
    assert!(
        out[0].paints.iter().any(|e| !e.enabled),
        "a camada desligada fica guardada no miolo"
    );
}

/// O fill de base ESCONDIDO não deixa um miolo visível; sem nada que se veja, não há miolo.
#[test]
fn a_hidden_base_fill_leaves_no_visible_core() {
    let mut p = square(10.0);
    p.fill_hidden = Some(Paint::Solid(AZUL));
    p.stroke = Some(StrokeSpec::new(VERMELHO, 2.0));
    let out = expand_layers(&p, Expand::OutlineStroke, 0.0);
    assert_eq!(
        out.iter().map(solid).collect::<Vec<_>>(),
        vec![Some(VERMELHO)]
    );
}

/// O deslocamento X/Y da camada é ONDE ela desenha: o traço convertido nasce lá.
#[test]
fn the_outline_lands_where_the_layer_is_offset() {
    let mut p = square(10.0);
    let mut e = PaintEntry::stroke(StrokeSpec::new(VERDE, 2.0));
    e.offset = [20.0, 5.0];
    p.paints = vec![e];
    let out = expand_layers(&p, Expand::OutlineStroke, 0.0);
    assert_eq!(out.len(), 1);
    let (lo, hi) = bbox(&out[0]);
    assert!(
        (lo[0] - 19.0).abs() < 0.05 && (hi[1] - 16.0).abs() < 0.05,
        "{lo:?} {hi:?}"
    );
}

/// O traço DILATADO (o Offset da camada) converte-se no anel crescido que se vê.
#[test]
fn the_outline_follows_a_dilated_stroke_layer() {
    let mut p = square(10.0);
    let mut e = PaintEntry::stroke(StrokeSpec::new(VERDE, 2.0));
    e.dilate = 4.0;
    e.dilate_join = 0; // Miter: o anel fica quadrado
    p.paints = vec![e];
    let out = expand_layers(&p, Expand::OutlineStroke, 0.0);
    assert_eq!(out.len(), 1);
    let (lo, hi) = bbox(&out[0]);
    assert!(
        (lo[0] + 5.0).abs() < 0.05 && (hi[0] - 15.0).abs() < 0.05,
        "{lo:?} {hi:?}"
    );
}

/// A opacidade e a mistura que se viam continuam: a do objecto em todo pedaço, a da camada no dela.
#[test]
fn the_outline_keeps_the_object_and_layer_look() {
    let mut p = square(10.0);
    p.fill = Some(Paint::Solid(AZUL));
    p.stroke = Some(StrokeSpec::new(VERMELHO, 2.0));
    p.opacity = Opacity::new(0.5);
    let mut e = PaintEntry::stroke(StrokeSpec::new(VERDE, 2.0));
    e.opacity = Opacity::new(0.5);
    e.blend = BlendMode::Screen;
    p.paints = vec![e];
    let out = expand_layers(&p, Expand::OutlineStroke, 0.0);
    assert_eq!(out.len(), 3);
    for (i, o) in out.iter().enumerate() {
        assert_eq!(o.opacity.get(), 0.5, "a opacidade do objecto no pedaço {i}");
    }
    // A da camada compõe-se DENTRO do pedaço dela, como antes dentro da forma.
    let tinta = &out[2].paints;
    assert_eq!(out[2].fill, None);
    assert_eq!(tinta.len(), 1, "{tinta:?}");
    assert_eq!(tinta[0].opacity.get(), 0.5);
    assert_eq!(tinta[0].blend, BlendMode::Screen);
}

/// Apply Power Stroke: o perfil é do traço de BASE; as camadas de traço assam-se como se vêem.
#[test]
fn the_power_stroke_also_bakes_the_stroke_layers() {
    let out = expand_layers(
        &etiqueta(),
        Expand::PowerStroke {
            stops: ph2d_vec_scene::WidthProfile {
                start: 0.2,
                mid: 2.0,
                end: 0.2,
                position: 0.5,
            }
            .to_stops(),
        },
        0.0,
    );
    no_stroke_left(&out);
    assert_eq!(
        out.iter().map(solid).collect::<Vec<_>>(),
        vec![Some(BRANCO), Some(AZUL), Some(VERMELHO)]
    );
}

/// **Um traço convertido é UM objecto** (a foto da cena 50, 02/11: o contorno tracejado virou 536
/// objectos na Hierarchy, um por traço do tracejado). É o *compound path* do Illustrator.
#[test]
fn a_dashed_stroke_outlines_into_one_compound_object() {
    let mut p = square(10.0);
    let mut s = StrokeSpec::new(VERDE, 1.0);
    s.dash = Some(ph2d_vec_scene::DashPattern::single(2.0, 1.0));
    p.paints = vec![PaintEntry::stroke(s.clone())];
    let out = expand_layers(&p, Expand::OutlineStroke, 0.0);
    assert_eq!(out.len(), 1, "um objecto por traço, não um por tracejado");
    assert!(
        out[0].subpaths.len() > 5,
        "os tracejados são contornos do mesmo objecto"
    );
    // O mesmo no traço de base, alinhado por fora.
    let mut q = square(10.0);
    s.align = ph2d_vec_scene::StrokeAlign::Outer;
    q.stroke = Some(s);
    assert_eq!(expand_layers(&q, Expand::OutlineStroke, 0.0).len(), 1);
}

/// **O miolo guarda a ordem de DENTRO dele**: um preenchimento de camada posto por baixo do de base
/// continua por baixo (a prova de mutação 02/11 achou a ordem do pedaço sem gate).
#[test]
fn the_core_keeps_the_order_of_its_own_fills() {
    let mut p = square(10.0);
    p.fill = Some(Paint::Solid(AZUL));
    p.stroke = Some(StrokeSpec::new(VERMELHO, 2.0));
    p.paints = vec![PaintEntry::fill(Paint::Solid(VERDE))];
    // De baixo para cima: o verde, o azul de base, o vermelho de base.
    p.base_order = BaseOrder { fill: 1, stroke: 2 };
    let out = expand_layers(&p, Expand::OutlineStroke, 0.0);
    assert_eq!(out.len(), 2);
    let ordem: Vec<_> = out[0].draw_order().collect();
    assert_eq!(
        ordem[..2],
        [
            ph2d_vec_scene::StackItem::Entry(0),
            ph2d_vec_scene::StackItem::BaseFill
        ],
        "o verde por baixo do azul"
    );
}
