//! Gates do Sketch. O oráculo central é o que separa "à mão" de "errado": as passadas TÊM de
//! DIFERIR entre si (uma linha só perturbada lê como erro; ≥2 distintas leem como esboço).

use super::*;
use crate::VecPath;
use crate::effect::{FxCtx, PathEffect};

const R: f64 = 60.0;

/// Um círculo em quatro cúbicas (a mesma construção dos gates do Zig Zag).
fn circle() -> Vec<VecVertex> {
    const K: f64 = 0.552_284_749_830_793_4;
    let p = [[R, 0.0], [0.0, R], [-R, 0.0], [0.0, -R]];
    let tang = [[0.0, K * R], [-K * R, 0.0], [0.0, -K * R], [K * R, 0.0]];
    (0..4)
        .map(|i| VecVertex {
            anchor: p[i],
            in_handle: [p[i][0] - tang[i][0], p[i][1] - tang[i][1]],
            out_handle: [p[i][0] + tang[i][0], p[i][1] + tang[i][1]],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        })
        .collect()
}

fn path(verts: Vec<VecVertex>) -> VecPath {
    VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    }
}

fn ref_of(p: &VecPath) -> f64 {
    FxCtx::of(p).ref_size
}

/// **Sem tremor, o caminho volta intacto** — o neutro tem de ser byte-idêntico, senão a pilha não
/// pode saltá-lo e o `Cow::Borrowed` do `cooked()` morre.
#[test]
fn a_neutral_sketch_is_the_path_unchanged() {
    let p = path(circle());
    let out = sketch_path(
        &p,
        &SketchSpec {
            roughness: 0.0,
            ..SketchSpec::default()
        },
        ref_of(&p),
        None,
    );
    assert_eq!(out.verts, p.verts, "neutro mudou o contorno");
    assert_eq!(out.subpaths.len(), p.subpaths.len(), "neutro criou passadas");
}

/// **O Sketch produz o número de passadas pedido** — cada contorno vira `passes` cópias (a 1ª é o
/// primário, o resto são subpaths). A mutação que ignora `passes` (sempre 1) deixa 0 subpaths.
#[test]
fn sketch_produces_the_requested_passes() {
    let p = path(circle());
    let spec = SketchSpec {
        passes: 3.0,
        roughness: 4.0,
        ..SketchSpec::default()
    };
    let out = sketch_path(&p, &spec, ref_of(&p), None);
    // 1 primário + 2 subpaths = 3 contornos.
    assert!(!out.verts.is_empty(), "o primário ficou vazio");
    assert_eq!(out.subpaths.len(), 2, "esperava 3 passadas (1 + 2 subpaths)");
}

/// **As passadas DIFEREM entre si** — é a propriedade "à mão". Cada passada tem seed própria
/// (`seed ^ pass`), então as âncoras de mesmo índice não coincidem. A mutação que usa a MESMA
/// seed em toda passada faz as cópias colarem (delta 0) e sangra aqui.
#[test]
fn the_passes_differ_from_each_other() {
    let p = path(circle());
    let spec = SketchSpec {
        passes: 2.0,
        roughness: 5.0,
        ..SketchSpec::default()
    };
    let out = sketch_path(&p, &spec, ref_of(&p), None);
    assert_eq!(out.subpaths.len(), 1, "esperava 2 passadas");
    let pass0 = &out.verts;
    let pass1 = &out.subpaths[0].verts;
    assert_eq!(pass0.len(), pass1.len(), "passadas com contagens diferentes");
    let max_delta = pass0
        .iter()
        .zip(pass1.iter())
        .map(|(a, b)| (a.anchor[0] - b.anchor[0]).hypot(a.anchor[1] - b.anchor[1]))
        .fold(0.0_f64, f64::max);
    assert!(
        max_delta > 0.5,
        "as passadas são quase idênticas (max delta {max_delta:.4}) — não lê como à mão"
    );
}

/// **O tremor cresce com a Roughness** — mais roughness, mais desvio da curva original. Mede o
/// maior afastamento do raio `R` no círculo. A mutação que ignora `roughness` (ou o normaliza)
/// achata a diferença.
#[test]
fn the_wobble_scales_with_roughness() {
    let p = path(circle());
    let rf = ref_of(&p);
    let dev = |rough: f64| -> f64 {
        let out = sketch_path(
            &p,
            &SketchSpec {
                passes: 1.0,
                roughness: rough,
                ..SketchSpec::default()
            },
            rf,
            None,
        );
        out.verts
            .iter()
            .map(|v| (v.anchor[0].hypot(v.anchor[1]) - R).abs())
            .fold(0.0_f64, f64::max)
    };
    let small = dev(2.0);
    let big = dev(8.0);
    assert!(
        big > small * 1.5,
        "roughness 8 ({big:.3}) não treme claramente mais que roughness 2 ({small:.3})"
    );
}

/// **O Sketch CONSOME o campo de Falloff** (escala o tremor por-amostra, como o Zig Zag). A
/// composição em si é gateada pela pilha + `fx_falloff`; aqui só se pina o canal.
#[test]
fn sketch_takes_the_falloff_field() {
    assert!(
        PathEffect::Sketch(SketchSpec::default()).takes_falloff(),
        "o Sketch tem de consumir o Falloff"
    );
}

/// Sonda (não-gate) — os números MEDIDOS da cena de smoke (a estrela do MEIO e da DIREITA).
/// `cargo test -p ph2d-vec-scene --lib fx_sketch::tests::probe -- --ignored --nocapture`
#[test]
#[ignore = "measurement probe, not a gate"]
fn probe_sketch_smoke() {
    // A estrela de 5 pontas do smoke (10 verts), raio 1.3.
    let verts: Vec<VecVertex> = (0..10)
        .map(|k| {
            let rr = if k % 2 == 0 { 1.3 } else { 1.3 * 0.42 };
            let a = (90.0 + f64::from(k) * 36.0).to_radians();
            VecVertex::corner([rr * a.cos(), rr * a.sin()])
        })
        .collect();
    let p = path(verts);
    let rf = ref_of(&p);
    for (passes, rough, detail, seed) in [(2.0, 4.0, 6.0, 1u64), (3.0, 7.0, 8.0, 3)] {
        let out = sketch_path(
            &p,
            &SketchSpec { passes, roughness: rough, detail, seed },
            rf,
            None,
        );
        let contours = 1 + out.subpaths.len();
        let verts_total = out.verts.len() + out.subpaths.iter().map(|c| c.verts.len()).sum::<usize>();
        println!(
            "star passes={passes} rough={rough}% -> {contours} passadas, {verts_total} verts totais"
        );
    }
}
