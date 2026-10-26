//! ⭐ **A geometria que um efeito entrega aos FILTROS** (F33, achado do dono no smoke de 26/10:
//! Bevel + Flag = vincos). Um filtro não sabe que há efeito: o Bevel lê a DIRECÇÃO do campo de
//! distância da curva cozida, e um laço ou uma quina perdida na curva vira vinco no relevo. As
//! réguas aqui são da curva, por efeito, sobre todo o catálogo — o plano e a medição estão em
//! `docs/UI_New_and_Simple/F33_plano.md`.

use super::*;
use crate::{VecPath, VecVertex, VertexKind};

/// Uma nuvem: topo em arco, base com três vieiras que se encontram em quinas CÔNCAVAS — a forma
/// da foto do dono. Seis quinas, dois lados curvos.
fn nuvem() -> VecPath {
    let k = 0.5523;
    let (y0, r) = (60.0, 20.0);
    let xs = [0.0, 40.0, 80.0, 120.0];
    let quina = |a: [f64; 2], h_in: [f64; 2], h_out: [f64; 2]| VecVertex {
        in_handle: h_in,
        out_handle: h_out,
        ..VecVertex::corner(a)
    };
    let mut v = vec![
        quina([0.0, 0.0], [-5.0, 30.0], [40.0, -20.0]),
        quina([120.0, 0.0], [80.0, -20.0], [125.0, 30.0]),
        quina([120.0, y0], [125.0, y0 - 30.0], [120.0, y0]),
    ];
    for i in (0..3).rev() {
        let (a, b) = (xs[i + 1], xs[i]);
        let m = (a + b) / 2.0;
        v.last_mut().expect("há sempre a quina anterior").out_handle = [a, y0 + r * k];
        v.push(VecVertex::smooth(
            [m, y0 + r],
            [m + r * k, y0 + r],
            [m - r * k, y0 + r],
        ));
        v.push(quina([b, y0], [b, y0 + r * k], [b, y0]));
    }
    // A última quina, (0, y0), fecha pelo lado esquerdo até (0, 0).
    v.last_mut().expect("a quina final").out_handle = [-5.0, y0 - 30.0];
    VecPath {
        verts: v,
        closed: true,
        ..VecPath::default()
    }
}

/// ⭐ A Nuvem DO APP (`ShapeKind::Cloud`, valores de fábrica) — a forma da foto do dono.
fn nuvem_do_app() -> VecPath {
    let k = crate::ShapeKind::Cloud;
    crate::cook(k, [0.0, 0.0], [120.0, 80.0], &k.defaults())
}

fn circulo() -> VecPath {
    let (c, r) = ([60.0, 40.0], 40.0);
    let k = 0.5523 * r;
    VecPath {
        verts: vec![
            VecVertex::smooth([c[0] + r, c[1]], [c[0] + r, c[1] - k], [c[0] + r, c[1] + k]),
            VecVertex::smooth([c[0], c[1] + r], [c[0] + k, c[1] + r], [c[0] - k, c[1] + r]),
            VecVertex::smooth([c[0] - r, c[1]], [c[0] - r, c[1] + k], [c[0] - r, c[1] - k]),
            VecVertex::smooth([c[0], c[1] - r], [c[0] - k, c[1] - r], [c[0] + k, c[1] - r]),
        ],
        closed: true,
        ..VecPath::default()
    }
}

/// Os parâmetros típicos de cada efeito — no Flag, os da foto do dono.
fn ajustado(kind: usize) -> Option<PathEffect> {
    let mut e = PathEffect::from_kind(kind)?;
    let nomes: Vec<&str> = e.params().iter().map(|p| p.name).collect();
    for (i, n) in nomes.iter().enumerate() {
        let v = match *n {
            "Start" => 0.1,
            "End" => 0.8,
            "Offset" => 0.2,
            "Size" => 5.0,
            "Ridges" => 10.0,
            "Smooth" => 1.0,
            "Copies X" => 3.0,
            "Move X" => 50.0,
            "Amount" => 40.0,
            "Bend" => 17.45,
            "Horizontal" => 12.21,
            "Vertical" => -11.66,
            "Angle" => 90.0,
            "Gap" => 5.0,
            "Passes" => 2.0,
            "Roughness" => 3.0,
            "Detail" => 8.0,
            "Seed" => 3.0,
            "Spacing" => 5.0,
            _ => continue,
        };
        e.set(i, v);
    }
    Some(e)
}

fn contornos(p: &VecPath) -> Vec<(&[VecVertex], bool)> {
    std::iter::once((p.verts.as_slice(), p.closed))
        .chain(p.subpaths.iter().map(|c| (c.verts.as_slice(), c.closed)))
        .collect()
}

fn cubica(v: &[VecVertex], i: usize) -> [[f64; 2]; 4] {
    let (a, b) = (&v[i], &v[(i + 1) % v.len()]);
    [a.anchor, a.out_handle, b.in_handle, b.anchor]
}

/// Quantos segmentos RECUAM — a derivada aponta contra a corda nalgum `t`: um laço ou uma cúspide
/// DENTRO do segmento, que o Bevel desenha como vinco.
fn recuos(p: &VecPath) -> usize {
    let mut n = 0;
    for (v, closed) in contornos(p) {
        let segs = if closed {
            v.len()
        } else {
            v.len().saturating_sub(1)
        };
        for i in 0..segs {
            let c = cubica(v, i);
            let corda = [c[3][0] - c[0][0], c[3][1] - c[0][1]];
            if corda[0].hypot(corda[1]) < 1e-9 {
                continue;
            }
            let recua = (0..=64).any(|s| {
                let t = f64::from(s) / 64.0;
                let u = 1.0 - t;
                let d: [f64; 2] = std::array::from_fn(|k| {
                    3.0 * u * u * (c[1][k] - c[0][k])
                        + 6.0 * u * t * (c[2][k] - c[1][k])
                        + 3.0 * t * t * (c[3][k] - c[2][k])
                });
                d[0].mul_add(corda[0], d[1] * corda[1]) < 0.0
            });
            n += usize::from(recua);
        }
    }
    n
}

/// Uma quina é QUALQUER quebra de tangente: um campo liso leva uma quina a uma quina, mas o ângulo
/// muda (o Twist a 90° leva a de 9,5° da nuvem a 3,5°). Um vértice liso sai com viragem ~`1e-16`.
const QUEBRA: f64 = 1e-6;

fn quinas(p: &VecPath) -> usize {
    contornos(p)
        .into_iter()
        .filter_map(|(v, closed)| crate::arc_path::ArcPath::from_contour(v, closed))
        .map(|ap| ap.corner_arcs(QUEBRA).len())
        .sum()
}

fn cozido(forma: &VecPath, pilha: &[PathEffect]) -> VecPath {
    let pilha: Vec<FxEntry> = pilha.iter().cloned().map(FxEntry::new).collect();
    run_stack(forma, &pilha).unwrap_or_else(|| forma.clone())
}

fn e_bloat(e: &PathEffect) -> bool {
    matches!(e, PathEffect::Bloat(_))
}

/// O Zig Zag e o Sketch deslocam pela NORMAL; num vale côncavo da Nuvem do app os pontos deslocados
/// cruzam-se (medido: as alças seguem a tangente da fonte e a corda volta para trás, 124°). É a
/// lei do deslocamento, a mesma antes e depois da reamostragem — aberto em F33_plano §6, não aqui.
fn desloca_no_vale(forma: &str, e: &PathEffect) -> bool {
    forma == "nuvem do app" && matches!(e, PathEffect::ZigZag(_) | PathEffect::Sketch(_))
}

/// ⭐ **G1 — nenhum efeito dobra um segmento para trás.** Pucker & Bloat fica de fora: a pétala
/// com bico É o efeito (a lei do Illustrator), não um laço de amostragem.
#[test]
fn no_effect_folds_a_segment_back() {
    let mut falhas = Vec::new();
    for (nome, forma) in [
        ("nuvem", nuvem()),
        ("nuvem do app", nuvem_do_app()),
        ("círculo", circulo()),
    ] {
        assert_eq!(recuos(&forma), 0, "a fonte ({nome}) não recua");
        for k in 0..PathEffect::KINDS.len() {
            let Some(e) = ajustado(k) else { continue };
            if matches!(e, PathEffect::Falloff(_)) || e_bloat(&e) || desloca_no_vale(nome, &e) {
                continue;
            }
            let r = recuos(&cozido(&forma, &[e]));
            if r > 0 {
                falhas.push(format!("{nome}/{}: {r}", PathEffect::KINDS[k]));
            }
        }
    }
    assert!(falhas.is_empty(), "segmentos que recuam: {falhas:?}");
}

/// ⭐ **G2 — um deformador CONTÍNUO não come quinas.** Um campo liso (Warp, Twist) leva uma quina a
/// uma quina; arredondá-la é o vinco que o Bevel mostra nas reentrâncias. Também sob um Falloff.
#[test]
fn a_continuous_deformer_keeps_every_corner() {
    let forma = nuvem();
    let fonte = quinas(&forma);
    assert_eq!(fonte, 6, "a fixtura tem as seis quinas");
    let mut falhas = Vec::new();
    for k in 0..PathEffect::KINDS.len() {
        let Some(e) = ajustado(k) else { continue };
        if !matches!(
            e,
            PathEffect::Warp(_)
                | PathEffect::Twist(_)
                | PathEffect::Repeat(_)
                | PathEffect::Knot(_)
        ) {
            continue;
        }
        let copias = if matches!(e, PathEffect::Repeat(_)) {
            3
        } else {
            1
        };
        let q = quinas(&cozido(&forma, &[e]));
        if q < fonte * copias {
            falhas.push(format!("{}: {q}/{}", PathEffect::KINDS[k], fonte * copias));
        }
    }
    let flag = PathEffect::KINDS
        .iter()
        .position(|n| *n == "Flag")
        .and_then(ajustado);
    let radial = PathEffect::KINDS
        .iter()
        .position(|n| *n == "Falloff Radial")
        .and_then(PathEffect::from_kind);
    let sob_falloff = cozido(
        &forma,
        &[radial.expect("Falloff Radial"), flag.expect("Flag")],
    );
    if quinas(&sob_falloff) < fonte {
        falhas.push(format!(
            "Falloff Radial + Flag: {}/{fonte}",
            quinas(&sob_falloff)
        ));
    }
    let app = nuvem_do_app();
    let fonte_app = quinas(&app);
    assert!(fonte_app > 0, "a Nuvem do app tem vales em quina");
    for nome in ["Flag", "Wave", "Twist"] {
        let e = PathEffect::KINDS
            .iter()
            .position(|n| *n == nome)
            .and_then(ajustado);
        let q = quinas(&cozido(&app, &[e.expect("efeito")]));
        if q < fonte_app {
            falhas.push(format!("Nuvem do app + {nome}: {q}/{fonte_app}"));
        }
    }
    assert!(falhas.is_empty(), "quinas perdidas: {falhas:?}");
}

/// **G3 — um vértice `Smooth` que um efeito emite é G1** (alças colineares). Um `Smooth` com
/// tangente partida é uma quina que o painel e o Edit tratam como lisa.
#[test]
fn a_smooth_vertex_out_of_an_effect_is_g1() {
    let mut falhas = Vec::new();
    for (nome, forma) in [
        ("nuvem", nuvem()),
        ("nuvem do app", nuvem_do_app()),
        ("círculo", circulo()),
    ] {
        for k in 0..PathEffect::KINDS.len() {
            let Some(e) = ajustado(k) else { continue };
            if matches!(e, PathEffect::Falloff(_)) || e_bloat(&e) {
                continue;
            }
            let out = cozido(&forma, &[e]);
            let partidos = contornos(&out)
                .into_iter()
                .flat_map(|(v, _)| v.iter())
                .filter(|x| x.kind == VertexKind::Smooth)
                .filter(|x| {
                    let a = [x.anchor[0] - x.in_handle[0], x.anchor[1] - x.in_handle[1]];
                    let b = [x.out_handle[0] - x.anchor[0], x.out_handle[1] - x.anchor[1]];
                    let (la, lb) = (a[0].hypot(a[1]), b[0].hypot(b[1]));
                    la > 1e-9
                        && lb > 1e-9
                        && (a[0].mul_add(b[1], -(a[1] * b[0])) / (la * lb)).abs() > 1e-6
                })
                .count();
            if partidos > 0 {
                falhas.push(format!("{nome}/{}: {partidos}", PathEffect::KINDS[k]));
            }
        }
    }
    assert!(falhas.is_empty(), "Smooth com tangente partida: {falhas:?}");
}
