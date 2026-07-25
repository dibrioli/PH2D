//! Gates do Hatch. O oráculo que contém o fenômeno é o BURACO: uma scanline que atravessa um
//! furo tem de sair PARTIDA em dois spans (a regra even-odd) — sem isso a hachura pinta por cima
//! do buraco.

use super::*;
use crate::compound::Contour;
use crate::effect::PathEffect;
use crate::{VecPath, VecVertex};

const R: f64 = 60.0;

/// Um quadrado de lado `2s` centrado na origem, como contorno de 4 quinas.
fn square_verts(s: f64) -> Vec<VecVertex> {
    vec![
        VecVertex::corner([-s, -s]),
        VecVertex::corner([s, -s]),
        VecVertex::corner([s, s]),
        VecVertex::corner([-s, s]),
    ]
}

fn square(s: f64) -> VecPath {
    VecPath {
        verts: square_verts(s),
        closed: true,
        ..VecPath::default()
    }
}

fn ref_of(p: &VecPath) -> f64 {
    crate::effect::FxCtx::of(p).ref_size
}

/// As linhas que o Hatch APENDOU (os subpaths abertos além dos originais).
fn hatch_lines(out: &VecPath, original_subpaths: usize) -> Vec<[[f64; 2]; 2]> {
    out.subpaths
        .iter()
        .skip(original_subpaths)
        .filter(|c| !c.closed && c.verts.len() == 2)
        .map(|c| [c.verts[0].anchor, c.verts[1].anchor])
        .collect()
}

/// **Sem espaçamento (guard), o caminho volta intacto** — o neutro tem de ser byte-idêntico.
#[test]
fn a_neutral_hatch_is_the_path_unchanged() {
    let p = square(R);
    let out = hatch_path(&p, &HatchSpec { spacing: 0.0, ..HatchSpec::default() }, ref_of(&p));
    assert_eq!(out.verts, p.verts);
    assert_eq!(out.subpaths.len(), p.subpaths.len());
}

/// **O Hatch enche o interior com linhas e MANTÉM o outline** — o contorno original fica, as
/// linhas entram como subpaths abertos DENTRO da caixa. A mutação que não emite linha nenhuma
/// (ou some com o outline) sangra.
#[test]
fn hatch_fills_the_interior_and_keeps_the_outline() {
    let p = square(R);
    let spec = HatchSpec { angle: 0.0, spacing: 10.0, cross: false };
    let out = hatch_path(&p, &spec, ref_of(&p));
    assert_eq!(out.verts, p.verts, "o outline não foi mantido");
    let lines = hatch_lines(&out, 0);
    assert!(!lines.is_empty(), "nenhuma linha de hachura");
    // Toda ponta de linha está DENTRO da caixa do quadrado.
    for [a, b] in &lines {
        for pt in [a, b] {
            assert!(
                pt[0] >= -R - 1e-6 && pt[0] <= R + 1e-6 && pt[1] >= -R - 1e-6 && pt[1] <= R + 1e-6,
                "linha fora da forma: {pt:?}"
            );
        }
    }
}

/// **O BURACO parte o span** (even-odd) — uma scanline pela altura do furo sai em DOIS segmentos,
/// com um vão do tamanho do furo. A mutação que ignora os subpaths fechados (só o primário)
/// devolve UM segmento cheio nesse y e sangra.
#[test]
fn a_hole_splits_the_span() {
    // Quadrado externo lado 2R + furo concêntrico lado R (subpath fechado).
    let mut p = square(R);
    p.subpaths.push(Contour {
        verts: square_verts(R / 2.0),
        closed: true,
    });
    let spec = HatchSpec { angle: 0.0, spacing: 10.0, cross: false };
    let out = hatch_path(&p, &spec, ref_of(&p));
    let lines = hatch_lines(&out, 1); // pula o subpath original (o furo)
    // Os segmentos na scanline do CENTRO (y = 0 exato — a grade tem `k*spacing`, e k=0 dá y=0;
    // as vizinhas ficam a `spacing` ⇒ um limiar de 1 unidade isola só a central).
    let mid: Vec<[[f64; 2]; 2]> = lines
        .iter()
        .copied()
        .filter(|[a, b]| ((a[1] + b[1]) * 0.5).abs() < 1.0)
        .collect();
    assert_eq!(
        mid.len(),
        2,
        "a scanline central devia partir em 2 segmentos (esq/dir do furo), veio {}",
        mid.len()
    );
    // Há um vão entre eles ~ a largura do furo (R): o span da direita começa depois do fim do
    // esquerdo, e o vão cobre o furo `[-R/2, R/2]`.
    let xr = |seg: &[[f64; 2]; 2]| -> (f64, f64) { (seg[0][0].min(seg[1][0]), seg[0][0].max(seg[1][0])) };
    let mut spans: Vec<(f64, f64)> = mid.iter().map(xr).collect();
    spans.sort_by(|a, b| a.0.total_cmp(&b.0));
    let gap = spans[1].0 - spans[0].1;
    assert!(gap > R * 0.6, "o vão do furo é pequeno demais ({gap:.2})");
}

/// **Cross-hatch dobra as linhas** — a 2ª família a 90° acrescenta ~o mesmo número. A mutação que
/// ignora `cross` deixa a contagem igual.
#[test]
fn cross_hatch_doubles_the_lines() {
    let p = square(R);
    let rf = ref_of(&p);
    let plain = hatch_lines(
        &hatch_path(&p, &HatchSpec { angle: 0.0, spacing: 10.0, cross: false }, rf),
        0,
    )
    .len();
    let crossed = hatch_lines(
        &hatch_path(&p, &HatchSpec { angle: 0.0, spacing: 10.0, cross: true }, rf),
        0,
    )
    .len();
    assert!(
        crossed as f64 > plain as f64 * 1.5,
        "cross ({crossed}) não é claramente mais que plain ({plain})"
    );
}

/// **Um caminho ABERTO não tem interior** — sem contorno fechado, o Hatch volta intacto.
#[test]
fn an_open_path_is_not_hatched() {
    let mut p = square(R);
    p.closed = false; // uma poligonal aberta, sem região a encher
    let out = hatch_path(&p, &HatchSpec { angle: 0.0, spacing: 10.0, cross: false }, ref_of(&p));
    assert_eq!(out.subpaths.len(), p.subpaths.len(), "hachurou um caminho sem interior");
    assert_eq!(out.verts, p.verts);
}

/// **O Hatch NÃO consome Falloff** — recorta uma região, não tem força por-ponto que um campo
/// module. Um Falloff acima dele é inerte, e o painel diz isso.
#[test]
fn hatch_does_not_take_the_falloff_field() {
    assert!(!PathEffect::Hatch(HatchSpec::default()).takes_falloff());
}

/// Sonda (não-gate) — os números MEDIDOS da cena de smoke (o círculo do MEIO e da DIREITA).
/// `cargo test -p ph2d-vec-scene --lib fx_hatch::tests::probe -- --ignored --nocapture`
#[test]
#[ignore = "measurement probe, not a gate"]
fn probe_hatch_smoke() {
    // O disco do smoke: um círculo de raio 1.2 (quatro cúbicas).
    const K: f64 = 0.552_284_749_830_793_4;
    let r = 1.2;
    let pc = [[r, 0.0], [0.0, r], [-r, 0.0], [0.0, -r]];
    let t = [[0.0, K * r], [-K * r, 0.0], [0.0, -K * r], [K * r, 0.0]];
    let verts = (0..4)
        .map(|i| VecVertex {
            anchor: pc[i],
            in_handle: [pc[i][0] - t[i][0], pc[i][1] - t[i][1]],
            out_handle: [pc[i][0] + t[i][0], pc[i][1] + t[i][1]],
            kind: crate::VertexKind::Smooth,
            corner_radius: 0.0,
        })
        .collect();
    let p = VecPath { verts, closed: true, ..VecPath::default() };
    let rf = ref_of(&p);
    for cross in [false, true] {
        let out = hatch_path(&p, &HatchSpec { angle: 45.0, spacing: 8.0, cross }, rf);
        println!("disc spacing=8% cross={cross} -> {} linhas de hachura", hatch_lines(&out, 0).len());
    }
}
