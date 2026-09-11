//! **O gate que faltava:** o arranjo sobre as formas que o PRODUTO produz.
//!
//! Os 16 gates da 1ª versão do Shape Builder usaram quadrados eixo-alinhados construídos à
//! mão, na identidade. Nenhum deles viu uma forma do catálogo (curva), nenhum viu um
//! `Transform` (ADR-0111: a geometria é LOCAL, centrada no 0, e a pose mora na entidade), e
//! todos ficaram VERDES enquanto o Enio via um véu que não batia com forma nenhuma.
//!
//! Aqui a fixture é a do print dele: **pentágono + estrela + retângulo arredondado**, cada um
//! **centrado no local 0** com a pose num `Xform` — exatamente como a Shape tool os deixa —
//! e assados no mundo pelo mesmo `bake_xform` que a sessão usa.
//!
//! ## O oráculo sai da DEFINIÇÃO, não do código
//!
//! Uma face de pertinência `m` é, por definição, o conjunto dos pontos que estão **dentro de
//! toda forma em `m`** e **fora de toda forma fora de `m`**. Então:
//!
//! 1. **Partição:** todo ponto coberto por alguma forma pertence a **exatamente uma** face, e
//!    a geometria dessa face o contém. (Um ponto coberto que nenhuma face contém é um BURACO
//!    — foi o que o Enio fotografou.)
//! 2. **Pertinência:** todo ponto DENTRO da geometria de uma face `m` está dentro de cada
//!    fonte de `m` e fora de cada fonte fora dela. (Uma face que vaza para fora das suas
//!    formas é o "véu grande que não bate" — a outra metade do print.)
//!
//! Nenhuma das duas olha para `compute_region`. Elas olham para o que o artista vê.

use ph2d_vec_boolean::Arrangement;
use ph2d_vec_scene::{ShapeKind, VecPath, Xform, bake_xform, contains_point, cook};

/// Uma forma do catálogo como a Shape tool a deixa: **geometria LOCAL centrada no 0**, pose
/// num `Xform` (ADR-0111). O `bake_xform` é o mesmo que `BuildSession::open` chama.
fn placed(kind: ShapeKind, half: [f64; 2], params: &[f64], xf: Xform) -> VecPath {
    let mut p = cook(
        kind,
        [-half[0], -half[1]],
        [half[0], half[1]], // centrada no local 0
        params,
    );
    bake_xform(&mut p, &xf);
    p
}

/// Translação pura (o caso comum: desenhou e arrastou).
fn at(x: f64, y: f64) -> Xform {
    Xform([1.0, 0.0, 0.0, 1.0, x, y])
}

/// Rotação + escala + translação (o caso que o gizmo produz num par de cliques).
fn posed(deg: f64, s: f64, x: f64, y: f64) -> Xform {
    let (sin, cos) = deg.to_radians().sin_cos();
    Xform([s * cos, s * sin, -s * sin, s * cos, x, y])
}

/// A cena do print do Enio: três formas sobrepostas, todas com pose.
fn enios_scene() -> Vec<VecPath> {
    vec![
        // Retângulo arredondado grande, no fundo (raio base 40, sem desvios, sem smoothing).
        placed(
            ShapeKind::RoundRect,
            [160.0, 110.0],
            &[40.0, 0.0, 0.0, 0.0, 0.0],
            at(40.0, 20.0),
        ),
        // Pentágono, girado — o gizmo gira.
        placed(
            ShapeKind::Polygon,
            [90.0, 90.0],
            &[5.0, 0.0],
            posed(17.0, 1.0, -40.0, 0.0),
        ),
        // Estrela de 5 pontas (razão interna 0.45), no topo.
        placed(
            ShapeKind::Star,
            [100.0, 100.0],
            &[5.0, 0.45, 0.0],
            at(80.0, 30.0),
        ),
    ]
}

/// A bbox de mundo de um punhado de formas (pelas âncoras + handles — folgada de propósito:
/// varrer FORA do necessário só torna o gate mais rigoroso).
fn bbox(paths: &[VecPath]) -> ([f64; 2], [f64; 2]) {
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for p in paths {
        for v in p.verts_all() {
            for q in [v.anchor, v.in_handle, v.out_handle] {
                lo = [lo[0].min(q[0]), lo[1].min(q[1])];
                hi = [hi[0].max(q[0]), hi[1].max(q[1])];
            }
        }
    }
    (lo, hi)
}

/// **Invariante 1 — a partição.** Todo ponto coberto por alguma forma cai em EXATAMENTE uma
/// face, e a geometria dessa face o contém.
///
/// O ponto coberto que nenhuma face contém é o buraco preto do print: o cursor passa por cima
/// dele e o Shape Builder não tem nada para pintar.
#[test]
fn every_covered_point_lands_in_exactly_one_face_whose_geometry_contains_it() {
    let shapes = enios_scene();
    let (lo, hi) = bbox(&shapes);
    let mut arr = Arrangement::new(shapes.clone());

    let step = 4.0;
    let (mut covered, mut orphan) = (0usize, Vec::new());
    let mut y = lo[1];
    while y <= hi[1] {
        let mut x = lo[0];
        while x <= hi[0] {
            let p = [x, y];
            let m = arr.membership_at(p);
            if m != 0 {
                covered += 1;
                match arr.face_at(p) {
                    Some(f) => {
                        assert_eq!(f.membership, m, "a face de {p:?} mudou de pertinência");
                        let hits = arr
                            .region(m)
                            .iter()
                            .filter(|c| contains_point(c, p))
                            .count();
                        assert_eq!(hits, 1, "o ponto {p:?} cai em {hits} componentes de {m:b}");
                    }
                    // O ponto está DENTRO de alguma forma e o arranjo não tem face para
                    // ele: a geometria da região `m` não o cobre. É um buraco.
                    None => orphan.push((p, m)),
                }
            }
            x += step;
        }
        y += step;
    }
    assert!(
        covered > 1000,
        "a varredura tem de ver a arte (viu {covered})"
    );
    assert!(
        orphan.is_empty(),
        "{} de {covered} pontos cobertos não têm face — buraco no arranjo. \
         Primeiros: {:?}",
        orphan.len(),
        &orphan[..orphan.len().min(6)]
    );
}

/// **Invariante 2 — a pertinência.** Todo ponto dentro da geometria de uma face está dentro
/// de cada fonte da pertinência dela, e FORA de cada fonte que não está.
///
/// É o gate do "véu grande que não bate com as formas": uma face que vaza para fora dos seus
/// operandos pinta a tela inteira, e nenhuma asserção de bbox pega isso.
#[test]
fn a_face_lies_inside_every_shape_it_belongs_to_and_outside_every_other() {
    let shapes = enios_scene();
    let (lo, hi) = bbox(&shapes);
    let mut arr = Arrangement::new(shapes.clone());

    // Todas as pertinências que a arte realmente produz (varre e coleta).
    let mut classes: Vec<u32> = Vec::new();
    let step = 4.0;
    let mut y = lo[1];
    while y <= hi[1] {
        let mut x = lo[0];
        while x <= hi[0] {
            let m = arr.membership_at([x, y]);
            if m != 0 && !classes.contains(&m) {
                classes.push(m);
            }
            x += step;
        }
        y += step;
    }
    assert!(classes.len() >= 5, "a cena tem de ter faces variadas");

    let mut leaks: Vec<([f64; 2], u32)> = Vec::new();
    for m in classes {
        let region: Vec<VecPath> = arr.region(m).to_vec();
        for comp in &region {
            // Varre a bbox da FACE: todo ponto dentro dela tem de ter a pertinência `m`.
            let (flo, fhi) = bbox(std::slice::from_ref(comp));
            let mut y = flo[1];
            while y <= fhi[1] {
                let mut x = flo[0];
                while x <= fhi[0] {
                    let p = [x, y];
                    if contains_point(comp, p) && arr.membership_at(p) != m {
                        leaks.push((p, m));
                    }
                    x += 2.0;
                }
                y += 2.0;
            }
        }
    }
    assert!(
        leaks.is_empty(),
        "{} pontos estão dentro da geometria de uma face mas NÃO na pertinência dela \
         (a face vaza para fora das formas que a definem). Primeiros: {:?}",
        leaks.len(),
        &leaks[..leaks.len().min(6)]
    );
}
