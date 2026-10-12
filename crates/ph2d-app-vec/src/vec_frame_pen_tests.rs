//! A *Resize Box* com a caneta fixa: a largura não muda, e a caixa VISUAL cai exacta nas duas bordas
//! — no proporcional, numa só direcção, sob escala de mundo, e no bico em esquadria de uma estrela
//! esticada (onde a margem muda com o ângulo). Controlo: o `scale_path` cru desvia.

use super::*;
use ph2d_ecs::VecShape;
use ph2d_vec_scene::{Rgba8, ShapeKind, StrokeSpec, rectangle};

fn cena(path: VecPath, largura: f64) -> (VecScene, VecPathId, VecPath) {
    let mut scene = VecScene::new();
    let id = scene.push_path(path);
    let p = scene.path_mut(id).expect("forma");
    p.stroke = Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), largura));
    let start = p.clone();
    (scene, id, start)
}

fn estrela() -> VecPath {
    crate::shape_live::recook_shape(&VecShape::Param {
        kind: ShapeKind::Star.as_u16(),
        w: 100.0,
        h: 100.0,
        values: ShapeKind::Star.defaults(),
    })
    .expect("estrela")
}

fn visual(scene: &VecScene, id: VecPathId, ws: [f64; 2]) -> ([f64; 2], [f64; 2]) {
    ph2d_vec_render::path_intrinsic_visual_bbox(scene.path(id).expect("forma"), ws).expect("caixa")
}

/// Redimensiona pela caixa visual com o pivô no canto `t` (0 = mínimo, 1 = máximo, 0,5 = centro) e
/// afirma: a largura de partida, e a caixa visual = a pedida, ao `tol`.
fn redimensiona(scene: &mut VecScene, id: VecPathId, ws: [f64; 2], t: f64, s: [f64; 2], w0: f64) {
    let start = scene.path(id).expect("forma").clone();
    let (lo, hi) = visual(scene, id, ws);
    let a = [lo[0] + (hi[0] - lo[0]) * t, lo[1] + (hi[1] - lo[1]) * t];
    resize_keeping_the_pen(scene, id, &start, ws, a, s);
    let (nlo, nhi) = visual(scene, id, ws);
    let tol = 1e-6 * (hi[0] - lo[0]).max(hi[1] - lo[1]);
    for i in 0..2 {
        let (tlo, thi) = (a[i] + (lo[i] - a[i]) * s[i], a[i] + (hi[i] - a[i]) * s[i]);
        assert!(
            (nlo[i] - tlo).abs() < tol && (nhi[i] - thi).abs() < tol,
            "eixo {i}, s {s:?}, pivô {t}: a caixa {nlo:?}–{nhi:?}, pedida {tlo}–{thi}"
        );
    }
    let w = scene
        .path(id)
        .and_then(|p| p.stroke.as_ref())
        .map(|s| s.width);
    assert_eq!(w, Some(w0), "a largura do traço mudou");
}

/// ⭐ O gate do prompt 22 B: 2× pela caixa, duas vezes, do canto — a largura igual e as duas bordas
/// exactas; depois só em X, e do centro.
#[test]
fn resizing_twice_by_the_box_keeps_the_stroke_and_lands_both_visual_edges() {
    let (mut scene, id, _) = cena(rectangle([0.0, 0.0], [100.0, 40.0]), 6.0);
    for _ in 0..2 {
        redimensiona(&mut scene, id, [1.0, 1.0], 0.0, [2.0, 2.0], 6.0);
    }
    redimensiona(&mut scene, id, [1.0, 1.0], 1.0, [2.0, 1.0], 6.0);
    redimensiona(&mut scene, id, [1.0, 1.0], 0.5, [0.5, 0.25], 6.0);
}

/// Sob a escala de MUNDO do objecto (a caneta é redonda: `w·√|det|`) e na estrela esticada só em X
/// (o bico em esquadria muda de ângulo — os passes de acerto fecham-no).
#[test]
fn under_world_scale_and_on_a_stretched_mitre_tip_the_box_is_still_exact() {
    let (mut scene, id, _) = cena(rectangle([0.0, 0.0], [100.0, 40.0]), 6.0);
    redimensiona(&mut scene, id, [3.0, 1.0], 0.0, [2.0, 1.5], 6.0);
    let (mut scene, id, _) = cena(estrela(), 8.0);
    redimensiona(&mut scene, id, [1.0, 1.0], 0.0, [2.0, 1.0], 8.0);
    redimensiona(&mut scene, id, [1.0, 1.0], 0.5, [1.0, 0.6], 8.0);
}

/// Controlo: a MESMA razão pelo `scale_path` cru engrossa o traço e tira a borda do sítio; o
/// `keep_the_pen` devolve a largura de partida.
#[test]
fn the_raw_scale_moves_the_edge_and_keep_the_pen_restores_the_width() {
    let (mut scene, id, start) = cena(rectangle([0.0, 0.0], [100.0, 40.0]), 6.0);
    let (lo, hi) = visual(&scene, id, [1.0, 1.0]);
    scene.scale_path(id, 2.0, 1.0, lo);
    let (nlo, nhi) = visual(&scene, id, [1.0, 1.0]);
    let w = scene
        .path(id)
        .and_then(|p| p.stroke.as_ref())
        .map(|s| s.width);
    assert!(w.is_some_and(|w| w > 6.0), "o cru não engrossou: {w:?}");
    assert!(
        (nhi[0] - (lo[0] + 2.0 * (hi[0] - lo[0]))).abs() > 0.1 || (nlo[0] - lo[0]).abs() > 0.1,
        "o cru caiu exacto — o gate não distingue"
    );
    let p = scene.path_mut(id).expect("forma");
    keep_the_pen(p, &start);
    assert_eq!(p.stroke.as_ref().map(|s| s.width), Some(6.0));
}

/// A pilha também fica: o traço de uma camada, o deslocamento (a sombra dura) e a dilatação — o
/// *Scale Strokes & Effects* desligado leva os EFEITOS junto. Controlo: o `scale_path` mexe-lhes.
#[test]
fn keep_the_pen_keeps_the_layers_width_offset_and_dilation() {
    let (mut scene, id, _) = cena(rectangle([0.0, 0.0], [100.0, 40.0]), 6.0);
    let p = scene.path_mut(id).expect("forma");
    let mut faixa =
        ph2d_vec_scene::PaintEntry::stroke(StrokeSpec::new(Rgba8::new(9, 9, 9, 255), 3.0));
    faixa.offset = [4.0, -2.0];
    faixa.dilate = 1.5;
    p.paints.push(faixa);
    let start = p.clone();
    let lerp = |scene: &VecScene| {
        let e = &scene.path(id).expect("forma").paints[0];
        let w = match &e.kind {
            PaintKind::Stroke(s) => s.width,
            PaintKind::Fill(_) => f64::NAN,
        };
        (w, e.offset, e.dilate)
    };
    scene.scale_path(id, 2.0, 2.0, [0.0, 0.0]);
    assert_ne!(
        lerp(&scene),
        (3.0, [4.0, -2.0], 1.5),
        "controlo: o cru não mexeu"
    );
    if let Some(p) = scene.path_mut(id) {
        *p = start.clone();
    }
    resize_keeping_the_pen(&mut scene, id, &start, [1.0, 1.0], [0.0, 0.0], [2.0, 2.0]);
    assert_eq!(lerp(&scene), (3.0, [4.0, -2.0], 1.5));
}
